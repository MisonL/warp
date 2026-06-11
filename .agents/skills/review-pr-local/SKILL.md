---
name: review-pr-local
specializes: review-pr
description: warp 仓库专用的 PR 审查指南。这里只能特化核心 review-pr 技能声明为可覆盖的类别。
---

# `warp` 仓库专用 PR 审查指南

本文件是核心 `review-pr` 技能的配套说明。它不会重新定义审查输出 schema、严重程度标签、安全规则或证据规则。它只特化核心技能标记为可覆盖的类别。

## 仓库专用风格与常见审查模式

- 如果现有测试已经覆盖有意义的行为，不要建议只改变 constructor 输入或 struct 字段的测试用例。只有在新测试会覆盖不同代码路径或边界场景时，才建议补充测试。
- 当 PR 明显是 V0 或初始实现时，应将 timeout、retry、lifecycle management 等健壮性建议表述为可选的未来工作，而不是阻塞问题；除非它们会影响正确性、安全性、数据丢失，或导致持续 UI 卡死。
- 对 Rust 变更，应用 `AGENTS.md` 中的仓库约定：避免不必要的类型标注，优先使用 import 而不是很长的路径限定符，将 context 参数命名为 `ctx` 并放在最后，删除未使用参数而不是给它们加 `_` 前缀，并优先在宏中使用 inline format arguments。
- 当 enum 可以合理穷尽匹配时，避免使用通配 `_` match arm；优先使用穷尽匹配，让未来新增 variant 能在审查期间暴露出来。
- 对新增或修改的 feature flag，除非没有编译期 gate 就无法编译，否则应优先使用 `FeatureFlag::YourFlag.is_enabled()` 这类高层运行时检查，而不是 `#[cfg(...)]`。
- 当调用栈可能已经持有 model lock 时，标记嵌套或冗余的 `TerminalModel` locking。优先把已锁定引用向下传递，并保持 lock scope 尽可能短。
- 在 WarpUI 代码中，标记 render 或 event handling 期间内联使用 `MouseStateHandle::default()` 的情况。Mouse state handle 应在构造期间创建，然后在需要处 clone/reference。
- 对面向用户的 UI 变更，只有当缺失验证与具体风险相关，或 PR 修改了应通过视觉方式验证的行为时，才提及缺少验证。

## 行为或 UI 影响变更需要视觉证据

- 如果 PR 修改了任何用户可见内容（UI component、layout、styling、用户可见表面的 copy、terminal/Warp app 视觉效果，或用户能感知的其他行为），应同时分析 `pr_description.txt` 和 workflow context 中可用的 PR comments，查找能端到端展示变更的截图、GIF 或视频。
  - 将 markdown 图片/视频嵌入（`![...](...)`、`<img ...>`、`<video ...>`）、GitHub user-attachment 链接（例如 `https://github.com/user-attachments/...`、`https://user-images.githubusercontent.com/...`）、Loom 链接和类似托管媒体视为有效证据。
  - `.github/pull_request_template.md` 中的 `Screenshots / Videos` section 存在但为空，不算作证据。
  - Unit tests、integration tests、`git diff --check`、代码路径说明和其他文本解释可以补充视觉证据，但对于用户可见行为不能替代视觉证据。
- 如果变更影响行为或 UI，并且描述或 comments 中没有附加截图或视频，应添加 inline 或 summary-level comment 要求补充。可使用类似措辞："For this user-facing change, please include screenshots or a screen recording demonstrating it working end to end."
- 当可手动测试的行为或 UI 影响变更缺少必需的视觉证据时，即使没有发现其他阻塞问题，也要将顶层 `body` 的 `## Verdict` section 中的最终建议设为 `Request changes`。顶层 `verdict` field 必须对应为 `"REJECT"`。
- 作者环境限制（例如 headless runner、没有 desktop、环境无法 capture）不能豁免 UI 影响变更的视觉证据要求。建议从本地 desktop run，或从具备 desktop/computer-use 支持的远程环境 capture 录屏（例如启用了 [computer use](https://docs.warp.dev/agent-platform/warps-agent/capabilities-overview/computer-use) 的 Oz coding agent）。回复可使用类似内容：_"This change is user-facing, so a screenshot or short recording is still required. If a local desktop isn't available, you can capture it from a coding agent that supports computer use (Oz is one option - see [Warp's computer use docs](https://docs.warp.dev/agent-platform/warps-agent/capabilities-overview/computer-use)) and attach it here."_ 将 verdict 设为 `Request changes`。
- 只有当用户可见行为确实无法通过视觉方式进行有意义展示时（例如只影响 screen reader 或非视觉 side effect 的变更），才豁免视觉证据。若如此，应简短说明截图或录屏为什么没有意义。绝不要基于作者环境限制进行豁免。
- 如果 PR 完全不是用户可见变更（例如纯 refactor、internal tools、build scripts、backend-only code、tests 或 documentation），不要要求截图或视频。

## 面向用户的字符串

- 标记在运行时读起来不自然，或用错误 casing 拼接句子片段的 interpolated text。
- Link text 应具有描述性，而不是裸 URL 或泛泛的 "click here" label。
- 确认同一个 PR 中相关 UI、comments、workflow messages 和 errors 的产品术语保持一致。

## Graceful degradation 与可观测性

- 当 URL、session links、workflow links、issue numbers 或 metadata 等可选动态数据可能缺失时，优先省略该元素或显示简短 fallback，而不是渲染空白或损坏输出。
- 不要建议从 error paths 中移除 session links、workflow URLs 或 diagnostic context。这些链接对调试失败的 automation 和用户报告很重要。
- 在用户可见表面优先使用通用且对用户安全的 error text，但要保留足够的 structured logging 或 diagnostic context，供维护者调查失败原因。
