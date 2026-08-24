#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose;
use futures_lite::future::block_on;
use warp_localization::LocaleId;
use warpui_core::clipboard::{ClipboardContent, ImageData};

use super::{
    ClipboardPasteContent, ImageFileReadError, MAX_IMAGE_SIZE_BYTES, attachment_error_for_locale,
    attachment_path_error_for_locale, classify_clipboard_content, open_image_file,
    parse_image_paths, process_clipboard_content, process_clipboard_content_for_locale,
    process_paths_for_locale, read_image_file_at_most, read_image_file_with_limit,
    split_windows_image_path_tokens,
};

const ONE_PIXEL_PNG: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";

#[test]
fn parses_single_and_quoted_image_paths() {
    let cwd = Path::new("/workspace");
    assert_eq!(
        parse_image_paths("image.png", cwd).unwrap(),
        vec![cwd.join("image.png")]
    );
    assert_eq!(
        parse_image_paths("'screenshots/image one.webp'", cwd).unwrap(),
        vec![cwd.join("screenshots/image one.webp")]
    );
}

#[test]
fn parses_multiple_image_paths_in_order() {
    let cwd = Path::new("/workspace");
    assert_eq!(
        parse_image_paths("one.png two.jpg", cwd).unwrap(),
        vec![cwd.join("one.png"), cwd.join("two.jpg")]
    );
}

#[test]
fn preserves_windows_path_separators_when_tokenizing() {
    assert_eq!(
        split_windows_image_path_tokens(r#"C:\Users\Alice\Pictures\shot.png"#),
        Some(vec![r#"C:\Users\Alice\Pictures\shot.png"#.to_owned()])
    );
    assert_eq!(
        split_windows_image_path_tokens(r#""C:\Users\Alice\Pictures\shot one.png""#),
        Some(vec![r#"C:\Users\Alice\Pictures\shot one.png"#.to_owned()])
    );
}

#[test]
fn decodes_percent_encoded_file_urls() {
    let cwd = Path::new("/workspace");
    assert_eq!(
        parse_image_paths("file:///workspace/image%20one.png", cwd),
        Some(vec![cwd.join("image one.png")])
    );
    assert!(parse_image_paths("file:///workspace/image%ZZ.png", cwd).is_none());
    assert!(parse_image_paths("file://remote/workspace/image.png", cwd).is_none());
}

#[test]
fn rejects_mixed_or_non_image_pastes() {
    let cwd = Path::new("/workspace");
    assert!(parse_image_paths("one.png notes.txt", cwd).is_none());
    assert!(parse_image_paths("ordinary prompt text", cwd).is_none());
}

#[test]
fn classifies_plain_clipboard_text_as_text() {
    let content = ClipboardContent::plain_text("ordinary prompt text".to_owned());

    let ClipboardPasteContent::Text(text) =
        classify_clipboard_content(content, Path::new("/workspace"))
    else {
        panic!("plain clipboard text should remain text");
    };

    assert_eq!(text, "ordinary prompt text");
}

#[test]
fn classifies_clipboard_image_paths_without_reparsing_spaces() {
    let content = ClipboardContent {
        paths: Some(vec!["/workspace/image one.png".to_owned()]),
        ..Default::default()
    };

    let ClipboardPasteContent::ImagePaths {
        paths,
        original_text,
    } = classify_clipboard_content(content, Path::new("/other"))
    else {
        panic!("an image file path should be classified as an attachment");
    };

    assert_eq!(paths, [Path::new("/workspace/image one.png")]);
    assert_eq!(original_text, "/workspace/image one.png");
}

#[test]
fn classifies_clipboard_image_data_before_text() {
    let content = ClipboardContent {
        plain_text: "image fallback".to_owned(),
        images: Some(vec![ImageData {
            data: vec![1, 2, 3],
            mime_type: "image/png".to_owned(),
            filename: None,
        }]),
        ..Default::default()
    };

    assert!(matches!(
        classify_clipboard_content(content, Path::new("/workspace")),
        ClipboardPasteContent::Image(_)
    ));
}

#[test]
fn processes_valid_images_in_paste_order() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.png");
    let second = directory.path().join("second.png");
    let png = general_purpose::STANDARD.decode(ONE_PIXEL_PNG).unwrap();
    std::fs::write(&first, &png).unwrap();
    std::fs::write(&second, &png).unwrap();

    let images = block_on(process_paths_for_locale(
        vec![first, second],
        LocaleId::EnUs,
    ))
    .unwrap();

    assert_eq!(
        images
            .iter()
            .map(|image| image.file_name.as_str())
            .collect::<Vec<_>>(),
        ["first.png", "second.png"]
    );
    assert!(images.iter().all(|image| !image.data.is_empty()));
}

#[test]
fn processing_is_all_or_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let valid = directory.path().join("valid.png");
    let invalid = directory.path().join("invalid.png");
    let png = general_purpose::STANDARD.decode(ONE_PIXEL_PNG).unwrap();
    std::fs::write(&valid, png).unwrap();
    std::fs::write(&invalid, b"not an image").unwrap();

    assert!(
        block_on(process_paths_for_locale(
            vec![valid, invalid],
            LocaleId::EnUs,
        ))
        .is_err()
    );
}

#[test]
fn rejects_oversized_image_before_reading_it() {
    let directory = tempfile::tempdir().unwrap();
    let oversized = directory.path().join("oversized.png");
    let file = std::fs::File::create(&oversized).unwrap();
    file.set_len(u64::try_from(MAX_IMAGE_SIZE_BYTES).unwrap() + 1)
        .unwrap();

    assert_eq!(
        block_on(process_paths_for_locale(
            vec![oversized.clone()],
            LocaleId::EnUs,
        ))
        .unwrap_err(),
        format!("Image is too large: {}.", oversized.display())
    );
}

#[test]
fn reports_directories_as_non_image_files() {
    let directory = tempfile::tempdir().unwrap();
    let image_directory = directory.path().join("image.png");
    std::fs::create_dir(&image_directory).unwrap();

    assert_eq!(
        block_on(process_paths_for_locale(
            vec![image_directory.clone()],
            LocaleId::EnUs,
        ))
        .unwrap_err(),
        format!("Image path is not a file: {}.", image_directory.display())
    );
}

#[test]
fn rejects_a_file_that_grows_past_the_limit_after_metadata_check() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("image.png");
    std::fs::write(&image, b"123").unwrap();
    let mut file = block_on(open_image_file(&image)).unwrap();
    std::fs::write(&image, b"12345").unwrap();

    assert_eq!(
        block_on(read_image_file_with_limit(&image, 3)),
        Err(ImageFileReadError::TooLarge)
    );
    assert_eq!(
        block_on(read_image_file_at_most(&mut file, 3)).unwrap(),
        b"1234"
    );
}

#[test]
fn reads_at_most_one_byte_past_the_image_limit() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("image.png");
    std::fs::write(&image, b"12345").unwrap();
    let mut file = block_on(open_image_file(&image)).unwrap();

    assert_eq!(
        block_on(read_image_file_at_most(&mut file, 3)).unwrap(),
        b"1234"
    );
}

#[cfg(unix)]
#[test]
fn reads_the_opened_file_after_its_path_is_replaced() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("image.png");
    let replacement = directory.path().join("replacement.png");
    std::fs::write(&image, b"original").unwrap();
    std::fs::write(&replacement, b"replacement").unwrap();

    let mut file = block_on(open_image_file(&image)).unwrap();
    assert!(block_on(file.metadata()).unwrap().is_file());
    std::fs::rename(replacement, image).unwrap();

    assert_eq!(
        block_on(read_image_file_at_most(&mut file, 32)).unwrap(),
        b"original"
    );
}

#[cfg(unix)]
#[test]
fn rejects_a_fifo_without_waiting_for_a_writer() {
    let directory = tempfile::tempdir().unwrap();
    let image = directory.path().join("image.png");
    let path = CString::new(image.as_os_str().as_bytes()).unwrap();
    // SAFETY: `path` is NUL-terminated and remains valid throughout the call.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);

    assert!(block_on(process_paths_for_locale(vec![image], LocaleId::EnUs)).is_err());
}

#[test]
fn processes_clipboard_image_content() {
    let png = general_purpose::STANDARD.decode(ONE_PIXEL_PNG).unwrap();
    let content = ClipboardContent {
        images: Some(vec![ImageData {
            data: png,
            mime_type: "image/png".to_owned(),
            filename: Some("clipboard.png".to_owned()),
        }]),
        ..Default::default()
    };

    let context = process_clipboard_content_for_locale(content, LocaleId::EnUs).unwrap();

    assert_eq!(context.mime_type, "image/png");
    assert_eq!(context.file_name, "clipboard.png");
    assert!(!context.data.is_empty());
}

#[test]
fn reports_unavailable_clipboard_image_data() {
    assert_eq!(
        process_clipboard_content(ClipboardContent::plain_text("text".to_owned())).unwrap_err(),
        "The clipboard does not contain an image."
    );
}

#[test]
fn attachment_errors_are_localized_for_simplified_chinese() {
    let path = Path::new("/tmp/example.png");

    assert_eq!(
        attachment_path_error_for_locale(
            LocaleId::ZhCn,
            "tui.attachments.error.unsupported_image_type",
            path,
        ),
        "不支持 /tmp/example.png 的图像类型。请使用 PNG、JPG、GIF 或 WebP。"
    );
    assert_eq!(
        attachment_error_for_locale(
            LocaleId::ZhCn,
            "tui.attachments.error.clipboard_no_image",
            &[],
        ),
        "剪贴板中不包含图像。"
    );
}
