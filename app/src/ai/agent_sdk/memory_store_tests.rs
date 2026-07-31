use super::*;

fn header_text<T: TableFormat>() -> Vec<String> {
    T::header_for_locale(LocaleId::ZhCn)
        .into_iter()
        .map(|cell| cell.content().to_owned())
        .collect()
}

#[test]
fn memory_store_table_headers_use_simplified_chinese_catalog() {
    assert_eq!(
        header_text::<MemoryStoreItem>(),
        [
            "UID",
            "所有者类型",
            "所有者 UID",
            "描述",
            "创建时间",
            "更新时间"
        ]
    );
    assert_eq!(
        header_text::<MemoryVersionItem>(),
        ["UID", "版本", "内容", "原因", "创建时间"]
    );
    assert_eq!(
        header_text::<AgentAttachmentItem>(),
        ["UID", "名称", "访问权限", "说明"]
    );
    assert_eq!(
        header_text::<MemoryItem>(),
        ["UID", "版本", "来源", "内容", "创建时间", "更新时间"]
    );
    assert_eq!(header_text::<CreateMemoryOutput>(), ["记忆 ID", "版本 ID"]);
    assert_eq!(header_text::<UpdateMemoryOutput>(), ["记忆 ID", "版本 ID"]);
}

#[test]
fn memory_store_status_messages_use_simplified_chinese_catalog() {
    assert_eq!(
        localization::text_for_locale(LocaleId::ZhCn, "agent_sdk.memory_store.output.no_stores"),
        "未找到记忆库。"
    );
    assert_eq!(
        localization::text_for_locale_with_args(
            LocaleId::ZhCn,
            "agent_sdk.memory_store.output.updated_memory",
            &[("uid", "memory_123")],
        ),
        "已更新记忆 memory_123。"
    );
}

#[test]
fn memory_response_json_keeps_canonical_field_names() {
    let output = CreateMemoryOutput {
        memory_id: "memory_123".to_owned(),
        version_id: "version_456".to_owned(),
    };

    assert_eq!(
        serde_json::to_value(output).expect("memory response should serialize"),
        serde_json::json!({
            "memory_id": "memory_123",
            "version_id": "version_456"
        })
    );
}
