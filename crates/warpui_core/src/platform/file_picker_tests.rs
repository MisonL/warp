use super::*;

#[test]
fn yaml_file_type_accepts_both_yaml_and_yml() {
    assert_eq!(FileType::Yaml.extensions(), &["yaml", "yml"]);
}

#[test]
fn markdown_file_type_accepts_md_and_markdown() {
    assert_eq!(FileType::Markdown.extensions(), &["md", "markdown"]);
}

#[test]
fn open_file_picker_title_uses_legacy_defaults() {
    assert_eq!(
        FilePickerConfiguration::new()
            .folders_only()
            .title_or_default(),
        "Choose directory..."
    );
    assert_eq!(
        FilePickerConfiguration::new().title_or_default(),
        "Choose file..."
    );
}

#[test]
fn file_picker_titles_preserve_explicit_titles() {
    assert_eq!(
        FilePickerConfiguration::new()
            .with_title("Pick a thing".into())
            .title_or_default(),
        "Pick a thing"
    );
    assert_eq!(
        SaveFilePickerConfiguration::new()
            .with_title("Write it".into())
            .title_or_default(),
        "Write it"
    );
}

#[test]
fn save_file_picker_title_uses_legacy_default() {
    assert_eq!(
        SaveFilePickerConfiguration::new().title_or_default(),
        "Save file as..."
    );
}
