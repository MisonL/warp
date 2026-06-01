## 描述
<!-- 如果包含任何 UI 变更，请记得把你的设计搭档加入 PR 审查。 -->

## 关联 Issue
<!--
链接此 PR 处理的 GitHub issue。打开此 PR 前，请确认：
-->
- [ ] 关联 issue 已标记为 `ready-to-spec` 或 `ready-to-implement`。
- [ ] 适当时，下面已包含实现的截图或短视频（尤其是用户可见或 UI 变更）。

## 测试
<!--
你如何测试了此变更？添加了哪些自动化测试？如果没有添加新测试，理由是什么？

凡是可以手动测试的变更都必须手动测试，而几乎所有变更都可以手动测试。如果你的变更可以手动测试，请包含能展示端到端工作效果的截图或屏幕录制。

你可以使用 `./script/run` 在本地运行应用。环境准备详情见 WARP.md。
-->

- [ ] 我已使用 `./script/run` 在本地手动测试这些变更

### 截图 / 视频
<!-- 适当时附上展示此变更的截图或短视频。如果此部分与你的 PR 无关，请移除。 -->

## Agent Mode
- [ ] Warp Agent Mode - 此 PR 通过 Warp 的 AI Agent Mode 创建

<!--
## Stable 的 Changelog 条目

以下条目会用于构建 stable release changelog 的软拷贝。如果 stable changelog 不需要条目，请留空或移除这些行。条目应与前缀位于同一行，不包含 `{{` `}}` 括号。你可以使用多行，甚至使用同一类型的多行。有效后缀如下：

- NEW-FEATURE：用于新的、相对较大的功能。这里列出的功能很可能关联文档、社交媒体发布或市场发布，因此请谨慎使用。
- IMPROVEMENT：用于现有功能的新能力。
- BUG-FIX：用于与已知 bug 或回归相关的修复。
- IMAGE：URL 指定的图片（托管在 GCP）会被添加到 Dev 和 Preview release。Stable release 请参考 #release Slack 频道中的置顶文档。
- OZ：Oz 相关更新。使用 `CHANGELOG-OZ`。每次 release 最多在应用内展示 4 条 Oz 更新。
- NONE：显式选择不纳入 changelog。对永远不应出现在 changelog 中的 PR（例如重构、内部工具、CI 变更），请使用 `CHANGELOG-NONE`。这会防止 changelog agent 推断出条目。

CHANGELOG-NEW-FEATURE: {{此处填写文本...}}
CHANGELOG-IMPROVEMENT: {{此处填写文本...}}
CHANGELOG-BUG-FIX: {{此处填写文本...}}
CHANGELOG-BUG-FIX: {{此处填写更多文本...}}
CHANGELOG-IMAGE: {{此处填写 GCP 托管 URL...}}
CHANGELOG-OZ: {{此处填写文本...}}
CHANGELOG-NONE
-->
