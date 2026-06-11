---
name: dedupe-issue-local
specializes: dedupe-issue
description: warp 仓库专用的重复 issue 判断指南。这里只能特化核心 dedupe-issue 技能声明为可覆盖的类别。
---

# `warp` 仓库专用重复 issue 判断指南

本文件是核心 `dedupe-issue` 技能的配套说明。它不会重新定义重复检测算法、相似度阈值或输出契约。它只特化核心技能标记为可覆盖的类别。

## 仓库专用归一化规则

- 比较标题前，去除低信号标题前缀，例如 `Bug:`、`Feature:`、`Request:`、`[Bug]`、`[Feature]`、`Warp:`，以及 `[macOS]`、`[Linux]`、`[Windows]` 等平台标签。
- 当核心症状和复现路径相同，app channel/version、OS version 和 shell name 应作为支持证据，而不是阻止判定重复的条件。
- 不要仅因为不同 Warp 表面共享 "agent"、"terminal"、"MCP"、"settings"、"search" 或 "sync" 等词，就把它们合并为重复。必须要求实际失败行为或请求能力存在重叠。
- 对 terminal 问题，在将两份报告判为重复之前，应比较 shell/session 上下文、命令输出行为、prompt 渲染、输入行为，以及是否涉及 remote/tmux。
- 对 agent 或 MCP 问题，在将两份报告判为重复之前，应比较触发路径、本地执行还是云端执行、MCP server/tool、可见错误和预期 workflow。
- 对 UI/rendering 问题，应比较受影响表面和可见症状。当标题含糊时，相似截图或录屏是很强的重复证据。

## 已知重复分组

本仓库尚未记录已知重复分组。当维护者反复将 issue 关闭为同一个规范线程的重复项时，每周 `update-dedupe` 循环会逐步建议在此处补充。
