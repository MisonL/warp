use std::io::{self, Write};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::{
    ClipboardCopy, copy_to_clipboard_with, osc52_sequences, tmux_passthrough, write_osc52_sequences,
};

#[test]
fn local_copy_reports_copied() {
    assert_eq!(
        copy_to_clipboard_with("conversation", false, |_| Ok(()), |_| Ok(())).unwrap(),
        ClipboardCopy::Copied
    );
}

#[test]
fn ssh_session_reports_sent_to_terminal() {
    let mut copied = None;

    assert_eq!(
        copy_to_clipboard_with(
            "conversation",
            true,
            |_| panic!("remote copies must not use the native clipboard"),
            |text| {
                copied = Some(text.to_owned());
                Ok(())
            },
        )
        .unwrap(),
        ClipboardCopy::SentToTerminal
    );
    assert_eq!(copied.as_deref(), Some("conversation"));
}

#[test]
fn native_failure_falls_back_to_osc52() {
    let mut copied = None;

    assert_eq!(
        copy_to_clipboard_with(
            "conversation",
            false,
            |_| anyhow::bail!("clipboard unavailable"),
            |text| {
                copied = Some(text.to_owned());
                Ok(())
            },
        )
        .unwrap(),
        ClipboardCopy::SentToTerminal
    );
    assert_eq!(copied.as_deref(), Some("conversation"));
}

#[test]
fn osc52_encodes_utf8_for_clipboard_and_primary() {
    let text = "hello 日🙂";
    let payload = STANDARD.encode(text.as_bytes());
    assert_eq!(
        osc52_sequences(text, false),
        format!("\x1b]52;c;{payload}\x07\x1b]52;p;{payload}\x07")
    );
}

#[test]
fn tmux_passthrough_wraps_and_doubles_escape_bytes() {
    assert_eq!(
        tmux_passthrough("\x1b]52;c;abc\x07"),
        "\x1bPtmux;\x1b\x1b]52;c;abc\x07\x1b\\"
    );
    let wrapped = osc52_sequences("x", true);
    assert_eq!(wrapped.matches("\x1bPtmux;").count(), 2);
    assert_eq!(wrapped.matches("\x1b\x1b]52;").count(), 2);
}

#[test]
fn clipboard_writer_emits_exact_markdown_payload() {
    let markdown = "# Conversation\n\nHello";
    let mut output = Vec::new();

    write_osc52_sequences(markdown, false, &mut output).unwrap();

    assert_eq!(output, osc52_sequences(markdown, false).into_bytes());
}

#[test]
fn clipboard_writer_propagates_output_errors() {
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    assert_eq!(
        write_osc52_sequences("conversation", false, &mut FailingWriter)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Other
    );
}

#[test]
fn hard_failure_reports_err() {
    assert!(
        copy_to_clipboard_with(
            "conversation",
            true,
            |_| panic!("remote copies must not use the native clipboard"),
            |_| anyhow::bail!("terminal write failed"),
        )
        .is_err()
    );
}
