---
name: classify-changelog-pr
description: 用于判断未标记 PR 是否应出现在 changelog 中以及归入哪个类别的参考指南。由 changelog-draft skill 内联使用，不作为单独 agent 调度。
---

# Classify Changelog PR

本文档提供对缺少显式 `CHANGELOG-*` marker 的 PR 进行分类的规则。changelog-draft agent 在决定是否包含未标记 PR 时，会内联遵循这些规则。

## 类别

- **NEW-FEATURE** - 实质性的新用户可见能力。保留给值得文档、市场或社交媒体关注的功能。
- **IMPROVEMENT** - 以用户可感知方式增强现有功能（性能、UX、新选项）。
- **BUG-FIX** - 修复用户可见 bug 或 regression。
- **OZ** - Oz / AI agent 能力变更。Stable changelog 中每个 release 最多 4 条。
- **NONE** - 显式选择不纳入 changelog。由上游 `fetch_prs.py` marker extraction 处理。

## 决策规则

### 始终排除

- 带显式 `CHANGELOG-NONE` marker 的 PR（贡献者选择排除）
- 已知 bot 作者的 PR（dependabot、renovate、github-actions、codecov）
- 只修改 CI workflow（`.github/workflows/`）、测试文件或开发工具的 PR
- 只更新内部文档、注释或 README 文件的 PR
- 没有用户可见行为变化的 dependency bump
- 没有可观察行为变化的 refactor（代码移动、重命名、格式化）

### 始终包含

- 带显式 `CHANGELOG-*` marker 的 PR（在适用本指南前已处理）
- 修复 crash、数据丢失或安全问题的 PR，即使没有 marker

### 按 channel 条件判断

- **Stable channel：** 只包含对所有用户 live 的变更。排除由 `DOGFOOD_FLAGS` 或 `PREVIEW_FLAGS` 门控的 PR。
- **Preview channel：** 包含由 `PREVIEW_FLAGS` 门控的 PR。仍排除仅 `DOGFOOD_FLAGS` 的变更。
- **Dev channel：** 包含所有用户可见变更，不受 flag gate 限制。

### Feature-flagged PR

如果 PR 在 diff 或标题中提到 `FeatureFlag` variant：
1. 检查它属于哪个 flag list（`RELEASE_FLAGS`、`PREVIEW_FLAGS`、`DOGFOOD_FLAGS`）。
2. 应用上面的 channel 规则。
3. 如果 flag 位于 `RELEASE_FLAGS` 或在 `app/Cargo.toml` 中默认启用，将其视为 live。
4. 在分类输出中将 `feature_flag` 设置为 flag name。

### Confidence level

- **high** - 清晰的用户可见变更，类别明显。
- **medium** - 可能用户可见，但类别或范围略有歧义。
- **low** - 不清楚用户是否会注意到；或 PR 同时触及内部和用户可见代码。设置 `needs_review: true`。

## 编写 changelog 文案

- 从用户视角编写，并保持 `Feature.Action.Result` 这类命名模式只用于内部 telemetry，不进入面向用户的文案；例如："Added X"、"Fixed Y"、"Improved Z"。
- 保持一句话，不超过 120 个字符。
- 不要引用内部实现细节、文件路径或函数名。
- 不要以 "PR" 或 PR 编号开头，这些会作为 metadata 添加。
- 对新功能使用主动语态和现在时（"Adds dark mode"），对修复使用过去时（"Fixed crash on startup"）。
