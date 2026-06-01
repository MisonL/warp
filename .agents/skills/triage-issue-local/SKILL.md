---
name: triage-issue-local
specializes: triage-issue
description: warp 仓库专用的 issue triage 指南。这里只能特化核心 triage-issue 技能声明为可覆盖的类别。
---

# `warp` 仓库专用 issue triage 指南

本文件是核心 `triage-issue` 技能的配套说明。它不会重新定义 triage 输出 schema、安全规则或后续问题契约。它只特化核心技能标记为可覆盖的类别。

## 启发式规则

- `warp` 是面向公众的 Warp desktop client 仓库。应将公开 issue 报告视为可能不完整，并避免在公开 issue thread 中要求提供 secrets、tokens、private workspace names、private repository names 或 account identifiers。
- 区分用户实际观察到的 Warp 行为，以及他们对 Rust modules、UI components、server behavior、feature flags 或 product intent 的猜测。
- 对提到其他 terminal、editor、shell 或 CLI tool 的 issue 报告，在分配 Warp ownership 前，应先判断问题是 Warp-specific，还是在 Warp 之外也普遍可复现。
- 当 issue 包含 screenshots、videos、logs、stack traces 或 command output 时，将它们作为主要证据；只针对无法从这些证据推断出的缺失细节询问后续问题。
- 在提出任何后续问题前，检查 Warp documentation 和仓库现有 feature set，判断报告者描述的期望行为是否已经被支持。如果已有 feature、setting 或 workflow 可以满足请求，应向报告者推荐它，而不是将 issue 视为 bug 或 feature gap。
- 如果报告涉及 billing（pricing、plans、subscriptions、payments、refunds、invoices、AI request quotas、charges）或 appeals（account suspensions、bans、takedowns、abuse decisions 或其他 account-status disputes），不要尝试将其 triage 为可操作的 bug 或 feature request。应改为告知报告者这些请求必须通过 Warp support channels（https://docs.warp.dev/support-and-community/troubleshooting-and-support/sending-us-feedback）处理，并引导其前往解决。视情况应用相关的 `area:billing` 或 `area:auth` label，确保 issue 仍能正确路由。

## 后续问题限制

每次 triage response **最多询问 2 个后续问题**。每个问题都必须具有高价值：回答后应能实质改变 label assignment、owner routing 或 reproduction confidence。不要询问能从现有证据推断出答案的问题，也不要把多个子问题打包进同一个 bullet。如果未知项超过 2 个，优先选择最可能解除 triage 阻塞的两个。

## Label taxonomy

本仓库的 label taxonomy 由 `.github/issue-triage/config.json` 管理。优先使用该配置中的 labels，尤其是 `area:*`、`os:*`、`repro:*`、`accessibility`、`needs-info`、`duplicate` 和主要 issue-type labels。除非 prompt 明确允许，否则不要发明新 label。

在 triage 期间评估 `ready-to-implement`，而不是依赖 issue-template defaults。对 bug reports，只有当 issue 可由已提供证据或直接本地验证复现，并且可能的修复范围足够窄，无需 product spec、design mocks 或大量调查即可实现时，才应用 `ready-to-implement`。如果 bug 不可复现、缺少清晰修复路径、需要 product/design decisions，或需要更深入的技术发现，则省略 `ready-to-implement`，并优先使用 `needs-info`、`ready-to-spec`、`needs-mocks` 或适当的 `repro:*` label。

基于用户报告的 surface 使用 area labels：

- `area:shell-terminal` 用于 terminal output、block rendering、shell integration、prompt rendering、command execution display 和 terminal-emulation behavior。
- `area:terminal-input` 用于 command-line input editing、cursor movement、key handling 和 typed text behavior。
- `area:window-tabs-panes` 用于 window、tab、pane、split、layout 和 focus behavior。
- `area:editor-notebooks` 用于 editors、notebooks、markdown rendering、LSP 和 code display。
- `area:agent` 用于 agent conversations、agent mode、cloud/local agent execution、prompts 和 AI-specific UI。
- `area:code-review` 用于 git diff views、review UI、review comments 和 PR-focused agent flows。
- `area:mcp` 用于 MCP server connection、tool/resource discovery、OAuth 和 integration issues。
- `area:settings-keybindings` 用于 settings UI、preferences、keyboard shortcuts 和 keybinding configuration。
- `area:warp-drive` 用于 Warp Drive objects、sync、sharing、workflows、notebooks、tab configs 和 persisted artifacts。
- 当报告包含 CPU、memory、GPU、startup、rendering、latency 或 responsiveness 症状时，使用 `area:performance:*`。当证据指向具体资源时，添加更具体的 CPU、memory 或 GPU label。

## 提出后续问题前要检查的信息

在向报告者索要更多信息前，检查 issue body、comments、attachments、logs、labels 和 repository context 中是否已有：

- Warp channel 和 version/build number，尤其是报告针对 Dev、Canary、Preview、Beta 还是 Stable。
- OS 和 version、architecture、display setup、Linux 上的 window manager 或 desktop environment，以及该问题是否 platform-specific。
- Shell 和 terminal context：shell name/version、prompt framework、shell integration status、正在运行的 command、terminal mode、本地还是 SSH/remote/tmux，以及该行为是否能在 fresh session 中复现。
- 清晰的 reproduction steps、expected behavior、actual behavior、frequency、regression timing，以及用户是否能在 Warp 之外复现。
- UI、rendering、layout、font、cursor、focus、window、pane、tab 和 accessibility 问题的视觉证据。当症状是视觉问题时，优先要求 screenshot 或 short recording。
- crashes、hangs、startup failures、update failures、authentication failures、MCP failures 和 agent execution failures 的 logs 与 diagnostics。只有当报告缺少可操作证据时，才要求 redacted logs。
- 对 AI/agent reports：agent 是 local 还是 cloud、已知 model、相关 conversation/session link、repository context、涉及的 tool 或 MCP server，以及触发失败的精确用户操作。
- 对 performance reports：大致 project/session size、command output size、CPU/memory/GPU observations、已提供的 profile 或 diagnostics，以及问题是否出现在 long-running sessions 之后。
- 对 keyboard 或 input reports：keyboard layout、custom keybindings、IME usage、冲突的 OS shortcuts、focused surface，以及相同按键是否在其他 app 中工作。
- 对 account、billing 或 auth reports：仅当用户已经提供时，使用 account tier 或 authentication method。不要在公开场合要求 private identifiers；当需要 private account details 时，引导用户联系 support。特别是 billing 或 appeals reports，不要在公开 thread 中继续追问 triage 问题，而应按上面的启发式规则将报告者重定向到 Warp support channels。

## 常见后续问题模式

- 没有媒体的 Visual UI/rendering issue：先要求 screenshot 或 short screen recording。
- Environment-sensitive terminal issue：询问 Warp version/channel、OS/version、shell，以及是否能在 fresh local session 中复现。
- SSH/tmux/remote issue：询问 local OS、remote OS、shell、是否涉及 tmux，以及复现问题的最小 command 或 workflow。
- Agent/MCP issue：询问 failing workflow、local vs cloud execution、相关 session link、MCP server/tool name，以及任何 redacted error text。
- Performance issue：询问大致规模、Warp 已运行多久、哪个 action 触发 spike 或 hang，以及是否有 logs 或 profile。

## Owner inference 提示

owner inference 优先使用 `.github/STAKEHOLDERS`。当不存在 path-level match 时，使用 label 和 issue surface 选择可能 owner，而不是默认归到宽泛的 app ownership。
