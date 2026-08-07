//! Image path parsing and clipboard/file processing for TUI attachments.

use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(unix)]
use async_fs::unix::OpenOptionsExt as _;
use base64::Engine as _;
use base64::engine::general_purpose;
use futures_lite::future::FutureExt as _;
use futures_lite::io::AsyncReadExt as _;
use url::Url;
use warp::tui_export::{
    ImageContext, MAX_IMAGE_SIZE_BYTES, MIME_SNIFF_BYTES, ProcessImageResult, infer_mime_type,
    is_supported_image_mime_type, process_image_for_agent,
};
use warp_localization::LocaleId;
use warpui_core::r#async::Timer;
use warpui_core::clipboard::{ClipboardContent, ImageData};
use warpui_core::clipboard_utils::CLIPBOARD_IMAGE_MIME_TYPES;

pub(super) enum ClipboardPasteContent {
    Image(ClipboardContent),
    ImagePaths {
        paths: Vec<PathBuf>,
        original_text: String,
    },
    Text(String),
    Empty,
}

pub(super) fn parse_image_paths(text: &str, cwd: &Path) -> Option<Vec<PathBuf>> {
    let tokens = split_image_path_tokens(text)?;
    if tokens.is_empty() {
        return None;
    }
    tokens
        .into_iter()
        .map(|token| resolve_image_path(&token, cwd))
        .collect()
}

pub(super) fn classify_clipboard_content(
    content: ClipboardContent,
    cwd: &Path,
) -> ClipboardPasteContent {
    if content.has_image_data() {
        return ClipboardPasteContent::Image(content);
    }

    let original_text = if content.plain_text.is_empty() {
        content
            .paths
            .as_ref()
            .map(|paths| paths.join("\n"))
            .unwrap_or_default()
    } else {
        content.plain_text.clone()
    };
    if let Some(paths) = content.paths.as_ref()
        && !paths.is_empty()
        && let Some(paths) = paths
            .iter()
            .map(|path| resolve_image_path(path, cwd))
            .collect()
    {
        return ClipboardPasteContent::ImagePaths {
            paths,
            original_text,
        };
    }
    if let Some(paths) = parse_image_paths(&content.plain_text, cwd) {
        return ClipboardPasteContent::ImagePaths {
            paths,
            original_text,
        };
    }
    if original_text.is_empty() {
        ClipboardPasteContent::Empty
    } else {
        ClipboardPasteContent::Text(original_text)
    }
}

fn resolve_image_path(token: &str, cwd: &Path) -> Option<PathBuf> {
    let path = if token.starts_with("file://") {
        let url = Url::parse(token).ok()?;
        file_url_path(&url)?
    } else if token == "~" {
        dirs::home_dir()?
    } else if let Some(rest) = token.strip_prefix("~/") {
        dirs::home_dir()?.join(rest)
    } else {
        PathBuf::from(token)
    };
    let path = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp").then_some(path)
}

fn file_url_path(url: &Url) -> Option<PathBuf> {
    if url.host_str().is_some() {
        return None;
    }
    validate_percent_encoding(url.path())?;
    url.to_file_path()
        .ok()
        .or_else(|| percent_decode_path(url.path()).map(PathBuf::from))
}

fn validate_percent_encoding(value: &str) -> Option<()> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            hex_value(bytes.get(index + 1).copied()?)?;
            hex_value(bytes.get(index + 2).copied()?)?;
            index += 3;
        } else {
            index += 1;
        }
    }
    Some(())
}

fn percent_decode_path(value: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte != b'%' {
            decoded.push(byte);
            continue;
        }
        let high = bytes.next().and_then(hex_value)?;
        let low = bytes.next().and_then(hex_value)?;
        decoded.push((high << 4) | low);
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(super) async fn process_paths(paths: Vec<PathBuf>) -> Result<Vec<ImageContext>, String> {
    process_paths_for_locale(paths, localization::current_locale()).await
}

async fn process_paths_for_locale(
    paths: Vec<PathBuf>,
    locale: LocaleId,
) -> Result<Vec<ImageContext>, String> {
    let mut images = Vec::with_capacity(paths.len());
    for path in paths {
        let bytes = match read_image_file_with_limit(&path, MAX_IMAGE_SIZE_BYTES).await {
            Ok(bytes) => bytes,
            Err(ImageFileReadError::NotFile) => {
                return Err(attachment_path_error_for_locale(
                    locale,
                    "tui.attachments.error.path_not_file",
                    &path,
                ));
            }
            Err(ImageFileReadError::TooLarge) => {
                return Err(attachment_path_error_for_locale(
                    locale,
                    "tui.attachments.error.image_too_large",
                    &path,
                ));
            }
            Err(ImageFileReadError::Io) => {
                return Err(attachment_path_error_for_locale(
                    locale,
                    "tui.attachments.error.read_image",
                    &path,
                ));
            }
        };
        let mime_type = infer_mime_type(&path, &bytes[..bytes.len().min(MIME_SNIFF_BYTES)]);
        if !is_supported_image_mime_type(&mime_type) {
            return Err(attachment_path_error_for_locale(
                locale,
                "tui.attachments.error.unsupported_image_type",
                &path,
            ));
        }
        let data = match process_image_for_agent(&bytes) {
            ProcessImageResult::Success { data } => data,
            ProcessImageResult::TooLarge => {
                return Err(attachment_path_error_for_locale(
                    locale,
                    "tui.attachments.error.image_too_large",
                    &path,
                ));
            }
            ProcessImageResult::Error(_) => {
                return Err(attachment_path_error_for_locale(
                    locale,
                    "tui.attachments.error.process_image",
                    &path,
                ));
            }
        };
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err(attachment_path_error_for_locale(
                locale,
                "tui.attachments.error.invalid_filename",
                &path,
            ));
        };
        images.push(ImageContext {
            data: general_purpose::STANDARD.encode(data),
            mime_type,
            file_name: file_name.to_owned(),
            is_figma: false,
        });
    }
    Ok(images)
}

pub(super) async fn read_clipboard_content() -> Result<ClipboardContent, String> {
    blocking::unblock(|| -> Result<ClipboardContent, String> {
        let mut clipboard = warpui::platform::create_system_clipboard()
            .map_err(|_| "The system clipboard is unavailable.".to_owned())?;
        Ok(clipboard.read())
    })
    .await
}

pub(super) fn process_clipboard_content(content: ClipboardContent) -> Result<ImageContext, String> {
    let images = content
        .images
        .ok_or_else(|| "Clipboard image data is unavailable.".to_owned())?;
    let image = CLIPBOARD_IMAGE_MIME_TYPES
        .iter()
        .find_map(|mime_type| {
            images
                .iter()
                .find(|image| image.mime_type == *mime_type)
                .cloned()
        })
        .ok_or_else(|| {
            attachment_error_for_locale(
                locale,
                "tui.attachments.error.clipboard_no_supported_image",
                &[],
            )
        })?;
    process_clipboard_image_data_for_locale(image, locale)
}

fn process_clipboard_image_data_for_locale(
    image: ImageData,
    locale: LocaleId,
) -> Result<ImageContext, String> {
    let data = match process_image_for_agent(&image.data) {
        ProcessImageResult::Success { data } => data,
        ProcessImageResult::TooLarge => {
            return Err(attachment_error_for_locale(
                locale,
                "tui.attachments.error.clipboard_image_too_large",
                &[],
            ));
        }
        ProcessImageResult::Error(_) => {
            return Err(attachment_error_for_locale(
                locale,
                "tui.attachments.error.clipboard_image_processing",
                &[],
            ));
        }
    };
    Ok(ImageContext {
        data: general_purpose::STANDARD.encode(data),
        mime_type: image.mime_type,
        file_name: image
            .filename
            .unwrap_or_else(|| default_clipboard_image_file_name_for_locale(locale)),
        is_figma: false,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImageFileReadError {
    Io,
    NotFile,
    TooLarge,
}

async fn read_image_file_with_limit(
    path: &Path,
    max_bytes: usize,
) -> Result<Vec<u8>, ImageFileReadError> {
    // Reject special paths before opening them; Windows can otherwise report a
    // generic I/O error or block while opening a named pipe.
    let path_metadata = async_fs::metadata(path)
        .await
        .map_err(|_| ImageFileReadError::Io)?;
    if !path_metadata.is_file() {
        return Err(ImageFileReadError::NotFile);
    }
    if path_metadata.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX) {
        return Err(ImageFileReadError::TooLarge);
    }

    let mut file = open_image_file(path)
        .await
        .map_err(|_| ImageFileReadError::Io)?;
    let file_metadata = file.metadata().await.map_err(|_| ImageFileReadError::Io)?;
    if !file_metadata.is_file() {
        return Err(ImageFileReadError::NotFile);
    }
    if file_metadata.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX) {
        return Err(ImageFileReadError::TooLarge);
    }

    let bytes = read_image_file_at_most(&mut file, max_bytes)
        .await
        .map_err(|_| ImageFileReadError::Io)?;
    if bytes.len() > max_bytes {
        return Err(ImageFileReadError::TooLarge);
    }
    Ok(bytes)
}

async fn open_image_file(path: &Path) -> std::io::Result<async_fs::File> {
    let mut options = async_fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK);
    options.open(path).await
}

async fn read_image_file_at_most(
    file: &mut async_fs::File,
    max_bytes: usize,
) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.take(
        u64::try_from(max_bytes)
            .unwrap_or(u64::MAX)
            .saturating_add(1),
    )
    .read_to_end(&mut bytes)
    .await?;
    Ok(bytes)
}

pub(super) fn default_clipboard_image_file_name() -> String {
    default_clipboard_image_file_name_for_locale(localization::current_locale())
}

fn default_clipboard_image_file_name_for_locale(locale: LocaleId) -> String {
    format!(
        "{}.png",
        localization::text_for_locale(locale, "tui.attachments.default_image_name")
    )
}

fn attachment_error_for_locale(locale: LocaleId, key: &str, args: &[(&str, &str)]) -> String {
    localization::text_with_args_for_locale(locale, key, args)
}

fn attachment_path_error_for_locale(locale: LocaleId, key: &str, path: &Path) -> String {
    let path = path.display().to_string();
    attachment_error_for_locale(locale, key, &[("path", &path)])
}

#[cfg(test)]
#[path = "image_processing_tests.rs"]
mod tests;
