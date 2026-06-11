# Release 配置

本 README 记录本目录中 `release_configurations.json` 文件的格式。该文件定义 Warp 的各个 release channel，并为运行 `create_new_releases.yml` GitHub workflow 所需的各类变量提供取值。

未来某个时间点，我们可能会用 JSON schema 文件替换本文档；该 schema 可作为 PR presubmit 的一部分，用于验证配置正确性。

## 字段

* **channel**：该 channel 的唯一标识符。
* **type**：release 节奏。目前有效值为 "nightly" 或 "weekly"。
* **is_prerelease**：如果为 true，该 channel 的 GitHub release 会被标记为 prerelease。
* **is_autopush**：如果为 true，该 channel 会使用 `channel_versions.json` 中的 "latest" 关键字自动部署新的 release candidate。非 autopush channel 需要手动变更才能部署。
* **release_base_name**：为该 channel 创建的 GitHub release 的基础名称。
* **release_body_text**：为该 channel 创建的 GitHub release 的正文文本。
* **sentry_project**：哪个 Sentry project 应接收该 channel 的崩溃和错误报告。
* **sentry_environment**：与该 channel 对应的 Sentry environment。
* **changelog_slack_channel**：每当切出新的 release candidate 时发布新 changelog 的 Slack channel。
* **gcs_cache_control_value**：release DMG 的 cache-control 响应头取值。
  - **重要**：cache-control header 的值_必须_全小写；Cloud CDN 不会遵循大写值。
