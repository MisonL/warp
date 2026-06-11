---
name: onboarding-verification-skill
description: 启动两个启用 computer use 的并行 Oz cloud agents，下载并安装最新 stable Linux Warp build，分别在登出态和登录态走查首次 onboarding 并截图，然后针对初始探索者提出的不同 onboarding 分支选择性派发后续 cloud agents。当用户要求在 cloud Linux 环境中测试、记录、截图或走查 Warp 首次安装/onboarding 体验时使用本技能。
---

# Onboarding 验证技能

使用本技能在 Linux 上验证首次 Warp 安装与 onboarding flow，并获得比单条线性 walkthrough 更广的分支覆盖。

parent agent 不应在本地执行 walkthrough。应启动两个启用 computer use 的并行 Oz cloud agents。两个初始 children 都安装适合其平台的最新 stable Warp Linux package，并在每个可见 onboarding step 截图，直到 Warp 到达可用的 terminal session。一个 child 验证 login-free flow，另一个 child 使用托管 secret `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 验证 logged-in flow。

这两个 baseline explorers 还负责发现有意义的 alternate onboarding branches，并返回后续 cloud agents 的具体计划。parent agent 应综合这些计划，去重重叠建议，并启动有界的第二波 targeted follow-up agents，以提升真实用户可能遇到路径的覆盖率。

## Parent workflow

1. 在单个并行 `run_agents` batch 中启动恰好两个 remote Oz cloud agents，并启用 computer use。
2. 除非用户提供了 environment，否则不要使用 environment-specific assumptions。如果未提供 environment，省略 environment ID，让 Warp 选择默认 remote environment。
3. 给两个 baseline child agents 提供下面的 shared child prompt，以及合适的 flow-specific prompt。
4. 等待两个 baseline agents 的报告。每份报告必须包含：
   - 已完成的 baseline walkthrough result 和 artifacts。
   - 已观察到的 UI quality issues、suspected bugs、error states 或 rough edges 的简短列表；可见时附截图。
   - 一个按优先级排列的 follow-up coverage plan，描述值得用额外 cloud agents 探索的不同 onboarding paths。
5. 如果 `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 缺失或认证不成功，将 authenticated baseline child 视为 blocked。
6. 从两份 baseline reports 构建合并后的 coverage map。去重到达同一 visible state 或覆盖同一 decision surface 的建议。
7. 为最有价值的 follow-up onboarding branches 启动第二个启用 computer use 的 `run_agents` batch：
   - 优先选择会实质改变 visible UI、available controls、downstream screens、auth state 或 setup outcomes 的分支。
   - 偏向可能暴露 correctness、polish、layout、truncation、loading 或 validation problems 的路径。
   - 默认总共最多启动四个 follow-up agents，除非用户明确要求 exhaustive coverage，或 baseline reports 显示超过四个明显不同且高价值的 branches。
   - 当 baseline agents 没有观察到具体 branch point 时，不要启动 speculative follow-ups；改为报告覆盖在 baseline pass 后停止。
8. 给每个 follow-up child 提供 shared child prompt、下面的 follow-up flow prompt、与其 assigned auth state 匹配的 logged-out 或 logged-in flow prompt，以及一项从 baseline reports 综合出的 branch assignment。
9. 等待所有 follow-up reports，然后再总结 coverage、issues、artifacts，以及任何值得后续运行但尚未探索的 branches。

## 托管 FTUE auth secret

- `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 是供 cloud agents 使用的 internal-team managed secret，不是 repo file 或 prompt literal。
- 该 secret 应认证为专用的非员工、非 `warp.dev` FTUE test user。
- 使用 `oz-dev secret update --team --value-file <private-token-file> ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 轮换该 secret。
- 将 private token file 仅视为本地 scratch material。不要将其读入 chat、打印、stage、commit、upload，或包含在 artifacts 中。managed secret 更新后删除它。
- Children 只能通过注入 remote run 的托管 environment variable 接收该 secret。

初始 `run_agents` call 使用如下形态：

```text
summary: 启动两个启用 computer use 的 baseline cloud agents，对比登出态和登录态 Warp onboarding 截图，并提出后续覆盖分支。
remote.computer_use_enabled: true
agent_run_configs:
- name: "warp-onboarding-logged-out"
  prompt: 下面的 logged-out flow prompt
- name: "warp-onboarding-logged-in"
  prompt: 下面的 logged-in flow prompt
base_prompt: 下面的 shared child prompt
```

当 baseline reports 识别出具体 follow-up branches 时，使用如下形态的第二个 `run_agents` call：

```text
summary: 启动 targeted cloud follow-up agents，探索 baseline onboarding explorers 识别出的不同 onboarding branches。
remote.computer_use_enabled: true
agent_run_configs:
- name: "warp-onboarding-followup-theme-choice"
  prompt: 下面的 follow-up flow prompt、下面的 logged-out flow prompt，以及一项综合出的 logged-out branch assignment
- name: "warp-onboarding-followup-model-choice"
  prompt: 下面的 follow-up flow prompt、下面的 logged-in flow prompt，以及一项综合出的 logged-in branch assignment
base_prompt: 下面的 shared child prompt
```

## Shared child prompt

给两个 cloud agents 提供这些共享指令：

```text
你正在验证 Linux 上的首次 Warp 安装和 onboarding 体验。

目标：
- 下载并安装适合此 cloud environment 的 distro 和 CPU architecture 的最新 stable Warp Linux build。
- 以全新的 first-run state 启动 Warp。
- 在每个可见 onboarding step 截图。
- 继续执行，直到 Warp 到达可用的 terminal session；如果 assigned flow 无法继续，则停止并报告 blocker。
- 注意会通向明显不同 screens、states 或 outcomes 的 alternate onboarding decisions，并为 parent orchestrator 返回具体 follow-up cloud-agent plans。
- 将 visual polish、missing assets、misalignment、overlapping content、clipped text、poor contrast、broken loading states、unexpected errors 和 confusing controls 视为 verification findings，而不是忽略它们。

安装要求：
- 只使用官方 stable Warp downloads。
- 不要使用 Warp Preview、Alpha、source builds 或 repository development build。
- 用 `uname -m` 检测 CPU architecture。
- 选择 package format 前，先检测 package manager 或 distro。
- 优先使用 native packages，而不是 AppImage，因为前者会正常安装 dependencies 并注册 app。

Stable Linux package 映射：
- Debian/Ubuntu with amd64 or x86_64: https://app.warp.dev/download?package=deb
- Debian/Ubuntu with arm64 or aarch64: https://app.warp.dev/download?package=deb_arm64
- Fedora/RHEL/CentOS/openSUSE with amd64 or x86_64: https://app.warp.dev/download?package=rpm
- Fedora/RHEL/CentOS/openSUSE with arm64 or aarch64: https://app.warp.dev/download?package=rpm_arm64
- Arch with amd64 or x86_64: https://app.warp.dev/download?package=pacman
- Arch with arm64 or aarch64: https://app.warp.dev/download?package=pacman_arm64
- 如果没有可用的 native package path，使用 AppImage fallback：
  - amd64 or x86_64: https://app.warp.dev/download?package=appimage
  - arm64 or aarch64: https://app.warp.dev/download?package=appimage_arm64

启动前：
- 创建 flow-specific artifact directory，例如 `~/warp-onboarding-logged-out` 或 `~/warp-onboarding-logged-in`。
- 确保 run 从全新的 Warp first-run state 开始。只移除 test user 的 Warp-specific config/data/cache/state directories，例如存在时的 `~/.config/warp-terminal`、`~/.local/share/warp-terminal`、`~/.local/state/warp-terminal` 和 `~/.cache/warp-terminal`。
- 不要删除无关 user files 或 system directories。

Screenshot workflow：
- 在与第一个可见 Warp window 交互前，先拍第一张 screenshot。
- 在每次 user action 前拍一张 screenshot。
- 如果 UI 发生变化，每次 action 后再拍一张 screenshot。
- 使用带 flow prefix 的顺序文件名，例如 `01-logged-out-initial-window.png` 或 `01-logged-in-initial-window.png`。
- 如果任何内容看起来不对，额外拍一张 issue-focused screenshot，尽可能清楚捕捉问题状态。
- 在 artifact directory 中维护 manifest file，并为每张 screenshot 记录：
  - filename
  - timestamp
  - 可见内容
  - 即将发生或刚发生的 action
- 对 issue-focused screenshots，添加 suspected issue category 以及问题出现的 screen 或 step。
- 不要在 manifest、logs、shell history、screenshots 或 final report 中包含 secret values、refresh tokens、ID tokens、auth redirect URLs 或 Authorization headers。

Onboarding behavior：
- 除非 flow-specific prompt 另有说明，baseline children 在每一步选择默认或最保守选项，同时记录值得单独 follow-up coverage 的 branch points。
- Follow-up children 走具体 assigned alternate branch；除非 branch assignment 另有说明，其他无关决策使用默认或最保守选项。
- 如果出现 telemetry、shell、theme、editor-import 或 agent integration choices，使用 default path，并在 manifest 中记录选择。
- 继续执行，直到可见并可使用正常 terminal prompt。

UI quality review：
- 留意 visually broken、明显未完成、misaligned、truncated、clipped、crowded、low-contrast、unexpectedly blank、stuck loading，或与相邻步骤不一致的 screens。
- 留意正常 flow exploration 中出现的 actionable errors 或 validation states，包括 auth failures、failed button transitions、controls 不响应、duplicated overlays、missing images，或 broken post-selection states。
- 对每个可疑状态：
  - 捕获一张 screenshot。
  - 记录 screen、导致它的 action、看起来不对的内容，以及它是否 blocked progress。
  - 如实描述问题。如果 expected behavior 不确定，应说明它看起来 suspicious，而不是声称是 confirmed bug。

Terminal verification：
- terminal session 可见后，运行一个无害的 flow-specific command：
  - logged-out flow: `echo warp-onboarding-logged-out-ready`
  - logged-in flow: `echo warp-onboarding-logged-in-ready`
- 捕获一张 final screenshot，展示可用 terminal 和 command output。

Report back：
- 你是 baseline explorer 还是 follow-up branch explorer。
- 你运行的是哪个 flow：logged-out 或 logged-in。
- 检测到的 OS 和 distro。
- 检测到的 CPU architecture。
- 使用的 Package URL 和 install method。
- 使用的 Launch command。
- walkthrough 是否到达可用 terminal session。
- 带简短描述的有序 screenshot list。
- Artifact directory path。
- 如果 harness 支持 artifact upload，报告任何内置 artifact IDs 或 attachment names。
- 任何 visual polish concern、suspected bug、error state，或 unpolished/misaligned screen，包括：
  - screenshot filename
  - screen 或 step
  - 它出现前刚执行的 action
  - 简短 observed behavior
  - 它是否 blocked progress
- 任何 blocker、crash、missing dependency、display problem、auth failure，或需要 judgment 的 step。
- 对 baseline explorers，包含一个 `Follow-up coverage plan` section，列出零个或多个 proposed child-agent branches。每个 proposal 必须包含：
  - suggested agent name
  - logged-out 或 logged-in flow
  - alternate branch 开始的 onboarding screen 或 decision point
  - 要探索的 exact alternate choice 或 action sequence
  - 它为什么与 baseline path 实质不同
  - 它可能揭示的 user-visible state、setup outcome 或 failure mode
  - 任何 secret、auth 或 environment dependency
  - priority: high, medium, or low
- 对 follow-up explorers，包含 assigned branch 是否 reachable 并 completed。如果沿 assigned path 走时出现新的 branch point，将其记录为 later-run suggestion，而不是自行递归扩展 run。

不要将 screenshots 或 logs 上传到公开 external services。如果 harness 提供内置 artifact 或 screenshot attachment mechanism，使用它。否则，将文件留在 artifact directory，并报告路径。
```

## Logged-out flow prompt

将此 prompt 附加到 logged-out child 的 shared child prompt 后：

```text
你负责 logged-out onboarding flow。

Flow-specific goal：
- 不要创建 account、登录，或使用真实 user identity。
- 只通过 login-free 或 account-free paths 继续执行，直到 Warp 到达可用 terminal session。
- 如果 flow 要求登录或创建 account，且没有 skip/continue-without-account option，停止并报告 blocker。

Flow-specific onboarding behavior：
- 如果有 skip、"continue without account"、"not now"、"login later" 或等效 option，使用它。
- 不要输入 email address、连接 OAuth、粘贴 auth token，或创建 credentials。
- 特别留意 logged-out branch points，例如选择 terminal-only 还是 agentic experiences、customization/layout options、third-party integration toggles，以及 terminal theme selection。如果出现这些分支，提出覆盖实质不同选择的 follow-up branches，而不是在当前流程中尝试所有 alternates。
- 使用 artifact directory `~/warp-onboarding-logged-out`。
```

## Logged-in flow prompt

将此 prompt 附加到 logged-in child 的 shared child prompt 后：

```text
你负责 logged-in onboarding flow。

Flow-specific goal：
- 使用 managed secret environment variable `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN`，认证为专用的非员工、非 `warp.dev` FTUE test user。
- 覆盖 already-authenticated user 可见的 onboarding screens。
- 沿 authenticated onboarding path 继续，直到 Warp 到达可用 terminal session。

Secret handling requirements：
- 做 auth work 前，验证 `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 存在且非空，但不要打印它。
- 绝不要 echo、log、screenshot、upload 或 report secret value。
- 避免 shell tracing（`set -x`），并避免编写会把 raw token 放入 shell history 或 process lists 的命令。
- 将每个包含 refresh token 的 auth redirect URL 都视为 secret-bearing material，即使经过 URL-encoding 之后也是如此。
- 不要将 token-bearing redirect URL 传给 shell command、desktop URI handler、browser address bar、process argument、log、artifact 或 report。尤其不要将 `xdg-open`、`gio open`、`open` 或等效命令与 redirect URL 搭配使用。
- 如果需要构造 auth redirect URL，只能将其保存在 clipboard value 或具有 user-only permissions 的 private temporary file 中，通过 Warp 可见的 Paste Auth Token flow 粘贴，然后在使用后立即删除 temporary file。

Secure Paste Auth Token process：
1. 验证 `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 存在且非空，但不要打印它。
2. 启动 Warp 的正常 login flow，并从 Warp 生成的 login URL 中派生当前 run 的 `state`。
3. 私下 normalize managed secret：
   - Trim surrounding whitespace；如果存在一对包裹在外的 single 或 double quotes，也去掉它们。
   - 如果 secret 可解析为带 `refresh_token` query parameter 的 URL，提取该 `refresh_token` value，并忽略 secret 中任何 stale `state`。
   - 否则，将 trimmed secret 视为 raw refresh token。
4. 将提取出的 refresh token 和当前 run 的 `state` 分别作为 query parameter values 进行 URL-encode。
5. 只在 clipboard value 或具有 user-only permissions 的 private temporary file 中构造 redirect URL。
6. 返回 Warp，并使用可见的 Paste Auth Token path：
   - 点击 Warp 显示的 `Click here to paste your token from the browser` link、`Paste Auth Token` button，或等效 pasted-token control。
   - 聚焦出现的 auth token text input。
   - 将准备好的 redirect URL 粘贴到该 input 中，并通过 Warp UI submit，让 Warp 解析并验证它。
7. 使用后立即删除任何 private temporary files；如果 environment 支持安全清理，也清空 clipboard。
8. 如果无法安全到达或自动化 Paste Auth Token UI，停止并报告 auth blocker；不要代替 Warp 解析 redirect，也不要使用 desktop URI handler、browser address bar，或带 token-bearing URL 的 shell command。

Preferred authenticated path：
- 在全新的 first-run state 中启动 Warp，并从 onboarding 中选择 login/sign-in path。
- 使用 Warp 内置 Paste Auth Token flow，而不是访问真实 OAuth providers、调用 desktop URI handler，或要求 agent 自行解析/验证 redirect URI。
- 如果 UI 暴露 copied login URL 或打开 browser，从 Warp 生成的 login URL 中派生 `<state>`。如果经过合理努力后 UI 仍不暴露 state，将其报告为 auth blocker，而不是绕过 state validation。
- 将 token 交给 Warp 前，不要用 Firebase Secure Token 进行 preflight。Warp 的 desktop redirect handler 只需要 `refresh_token` 和 `state`；`user_uid` 是可选的，`deleted_anonymous_user=true` 会处理 anonymous-user override case。
- 将 `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 视为以下 secret shapes 之一：
  - raw Firebase refresh token，或
  - 包含 `refresh_token` query parameter 的完整 Warp desktop auth redirect URL。
- 在不打印的情况下，将 secret normalize 成 current-run redirect URL：
  - Trim surrounding whitespace；如果存在一对包裹在外的 single 或 double quotes，也去掉它们。
  - 如果 secret 可解析为带 `refresh_token` query parameter 的 URL，提取该 `refresh_token` value，并忽略 secret 中任何 stale `state`。
  - 否则，将 trimmed secret 视为 raw refresh token。
  - 将提取出的 refresh token 和当前 run 的 `state` 分别作为 query parameter values 进行 URL-encode。
  - 构造 `warp://auth/desktop_redirect?refresh_token=<url-encoded-normalized-refresh-token>&deleted_anonymous_user=true&state=<url-encoded-current-state>`。
  - 除非所提供 desktop redirect URL 中已经存在 `user_uid`，否则不要包含 `user_uid`；此 flow 不需要它。
- 只在 clipboard value 或 private temporary file 中构造 normalized redirect URL，然后通过 Paste Auth Token UI 交给 Warp。不要在 Warp 之外解析、验证或路由 redirect。
- 如果无法安全到达或自动化 Paste Auth Token flow，停止并报告 auth blocker；不要使用 desktop URI handler 或任何包含 token-bearing URL 的 shell command。

Fallback authenticated path：
- 如果 Warp 拒绝 normalized redirect，报告 non-sensitive user-visible error，并在不报告任何 token contents 的情况下，分类 secret 看起来是 raw token 还是 desktop redirect URL。
- 如果 Paste Auth Token flow 被 UI automation issues 阻塞，报告 blocker，并包含 automation 失败处的 exact non-sensitive step。
- 不要为此 child 切换到 logged-out path。

Flow-specific onboarding behavior：
- 出现 auth choice 时，选择 login/sign-in，而不是 skip/login-later。
- auth 成功后，使用默认或保守选项继续通过剩余 onboarding screens。
- 特别留意 logged-in branch points，例如 model selection、account-aware onboarding screens、AI/agent setup、workspace 或 project setup，以及任何会改变 available product capability 的 decision。如果出现这些分支，提出覆盖实质不同选择的 follow-up branches，而不是在当前流程中尝试所有 alternates。
- terminal verification 成功后，点击右上角 avatar/account control，从该 menu 打开 Settings，并额外 capture 一张 screenshot，清楚显示 Warp settings 或 account/profile settings 中已登录用户的 email address。
- 在 manifest 和 final report 中包含 account/settings email screenshot。email address 本身可以在 screenshot 中可见，但除非用户明确要求，不要将该 email 复制到 logs、shell output 或 final text report 中。
- 使用 artifact directory `~/warp-onboarding-logged-in`。
```

## Follow-up flow prompt

对每个 second-wave child，将此 prompt 附加到 shared child prompt 后，随后附加匹配的 logged-out 或 logged-in flow prompt，以及一项从 baseline reports 综合出的 branch assignment：

```text
你负责 parent orchestrator 从先前 baseline exploration report 中选出的一条 follow-up onboarding branch。

Follow-up branch behavior：
- 从全新的 first-run Warp state 开始，并使用 shared instructions 安装同一个 latest stable Linux build。
- 遵守 assigned auth state：logged-out assignments 保持登出；logged-in assignments 使用 managed authenticated flow。
- 遵循 branch assignment 中的 exact alternate onboarding choice 或 action sequence。
- 在每个 assigned branch decision 前后 capture screenshots；如果 path 允许，随后继续到可用 terminal session。
- 应用与 baseline explorers 相同的 UI quality review standard，并指出任何看起来 broken、rough、misaligned、confusing 或 unexpectedly error-prone 的内容。
- 如果 assigned branch 不可 reachable，capture 最接近的相关 screen，报告它为什么不可 reachable，并且不要静默替换为其他 branch。
- 如果 assigned branch 揭示了另一条有趣的 alternate path，将其记录为 later-run suggestion，而不是自行递归启动更多 agents。

Final report additions：
- 用简洁且 non-sensitive 的表述复述你尝试的 exact branch assignment。
- 说明它是 reachable、completed、blocked 还是 not applicable。
- 当 UI 中可见该比较时，将 branch outcome 与可能的 baseline behavior 进行对比。
```

## 成功标准

满足以下条件时，run 视为成功：

- 针对检测到的 architecture，从官方 Linux package 或 AppImage 安装了 Warp stable。
- 为每个 onboarding screen 和最终可用 terminal 捕获了 screenshots。
- logged-out child 在没有登录、创建 account 或使用真实 user identity 的情况下到达可用 terminal。
- logged-in child 使用 `ONBOARDING_AGENT_FTUE_REFRESH_TOKEN` 完成认证，并在 authenticated FTUE path 中到达可用 terminal。
- logged-in child 从 avatar/settings flow 捕获了额外 post-login screenshot，展示已登录用户的 email address。
- 每个 terminal session 都可用到足以运行其 flow-specific `echo` command。
- 两个 baseline explorers 都返回了具体 follow-up coverage proposals，或明确说明没有观察到有意义的额外 branch points。
- parent orchestrator 针对最高价值的具体 branch proposals 启动了 targeted second-wave agents，除非不存在此类 proposals，或 prerequisite blocker 使其不可行。
- 每个报告的 visual polish concern、suspected bug 或 error state，只要问题在屏幕上可见，都包含 screenshot reference。

## 常见失败处理

- 如果 package manager 提示确认，使用该 package manager 支持的 non-interactive confirmation flag。
- 如果因为 display setup 导致 `warp-terminal` 启动失败，检查 cloud environment 的 display variables；如果 computer use 提供 desktop/app launcher，尝试从那里启动。
- 如果 logged-out flow 在登录处阻塞且没有 skip path，停在该 screen，capture screenshot，并将其报告为 logged-out flow 的 terminal point。
- 如果 logged-in flow 因 secret 缺失、无效、过期、被撤销，或无法通过 Warp auth redirect flow 路由而无法认证，停在该 screen，capture screenshot，并报告 non-sensitive blocker。
- 如果由于 dependencies 不可用导致 native package 无法安装，fallback 到匹配的 AppImage，并清楚报告该 fallback。
