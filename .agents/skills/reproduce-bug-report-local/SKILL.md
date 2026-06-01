---
name: reproduce-bug-report-local
specializes: reproduce-bug-report
description: Warp 仓库专用的 bug 复现指南。针对登出态 Warp UI 复现、报告者精确版本安装和免登录 onboarding 特化核心 reproduce-bug-report 技能。
---

# `warp` 仓库专用 bug 复现指南

本文件是核心 `reproduce-bug-report` 技能的配套说明。它不会重新定义共享的 Oz computer-use 编排、artifact 处理、安全规则或报告格式。它只针对 Warp bug 报告特化范围和设置。

## 范围

- 仅将此 workflow 用于 app 保持登出状态时可触发的 Warp bug。
- 适用于 UI 可见的 Warp bug、交互 bug、rendering/layout bug、登出态 onboarding bug、settings bug、editor/display bug、terminal-display bug，以及其他截图或录屏有帮助的视觉或交互问题。
- 不要将它用于 authenticated-user flows、account-specific state、cloud-synced state、logged-in onboarding，或需要登录的 AI behaviors。
- 如果报告需要 authentication、account state、cloud sync 或其他 logged-in-only capability，不要使用此本地特化启动 repro agent；应报告它超出当前登出态 Warp workflow 的范围。

## Warp 版本与安装策略

- 优先使用用户报告的精确 Warp version/build 和 channel 进行复现。
- 默认不要从源码构建 Warp。改为安装与报告者 version/channel 匹配的 Linux package 或 binary release。
- 如果 bug 报告给出的是 macOS 或 Windows build，且存在匹配的 Linux artifact，则使用相同 version/channel 对应的 Linux build，并说明这是报告者平台的 Linux proxy。
- 使用环境中可用的仓库工具或 Warp release tooling/docs，查找并安装精确版本的 Linux artifact。当精确匹配版本可安装时，不要静默替换为最新 stable build。
- 如果无法找到或安装精确 version/build，应清楚报告这一点，说明已尝试的操作；只有在有助于继续调查时，才使用最接近且有理由的 fallback。
- 在 manifest 和 final report 中记录报告者请求的 Warp version、已安装的 Linux version、已安装 artifact 的来源，以及任何 fallback decision。

## 登出态 Warp baseline

- 在整个复现尝试期间保持 Warp 登出。不要创建账号、登录、粘贴 auth tokens，或使用真实用户凭据。
- 启动 Warp，并完成 login-free / continue-without-account onboarding 路径，直到可使用正常的登出态 terminal session。
- 在尝试复现具体 bug 前，先 capture 一张 post-onboarding baseline screenshot。
- 如果进入正常登出态 Warp session 后仍无法触发分配的 bug，应停止并报告 blocker，而不是临时改走 authenticated flow。

## 本地 prompt 补充

将核心技能应用到 Warp 时，确保 parent prompt 和 child prompts 包含：

- Reporter Warp version/build/channel：报告中的精确值，或 `unknown`。
- Build/app target：要安装的精确版本 Linux Warp package/binary；如果没有精确 artifact，则填写有理由的 fallback。
- Assigned Warp state：first-run logged-out state、completed logged-out onboarding、terminal/session/layout/settings state，或目标 code-path hypothesis。
- 提醒 Warp 必须保持登出，并说明 logged-in-only reports 会被此特化阻塞。

## 本地复现优先级

- 在扩大搜索空间前，先匹配报告者的 Warp version/build/channel。
- 先严格遵循 issue 的原始步骤，然后最多测试两个由 issue 或 code-path hypothesis 支持的目标变体。
- 相比宽泛探索式点击，应优先使用从 Warp UI strings、settings names、feature names、telemetry names、route names 和相关 components 推导出的目标假设。
- 在 final report 中包含：
  - 报告者请求的 Warp version/build/channel
  - 已安装的 Linux Warp version/build/channel
  - package 或 binary source
  - 是否使用了 fallback
  - 该测试是否是 macOS 或 Windows 报告的 Linux proxy
