---
name: promote-feature
description: 在 Warp 代码库中，将 feature-flagged feature 推广到 Dogfood、Preview 或 Stable。当 FeatureFlag 背后的 feature 准备向更大受众 rollout 时使用，包括接好编译期/运行时桥接，并安全推迟 flag 清理。
---

# promote-feature

指导将 gated `FeatureFlag` variant 分阶段推广到 Dogfood、Preview 或 Stable，并安排后续清理。

## 概述

Feature flag 有两个相互作用的层：
- **Runtime**（`warp_core/src/features.rs`）：`DOGFOOD_FLAGS`、`PREVIEW_FLAGS`、`RELEASE_FLAGS`，启动时按 channel 启用。
- **Compile-time**（`app/Cargo.toml` + `app/src/lib.rs`）：`[features]` 中的 Cargo feature。`default = [...]` 数组会为所有 build 启用某个 feature。`app/src/lib.rs` 中的 `enabled_features()` 通过 `#[cfg(feature = "...")]` 将每个 Cargo feature 桥接到它的 `FeatureFlag` variant。

**推广到 Stable 后不要立即移除 flag。** 至少保留 1-2 个 release cycle，这样 rollback 可以是一行 PR（从 `default` 中移除该条目）。稍后清理时使用 `remove-feature-flag` skill。

## 推广到 Dogfood

将 flag 添加到 `warp_core/src/features.rs` 的 `DOGFOOD_FLAGS`：

```rust
pub const DOGFOOD_FLAGS: &[FeatureFlag] = &[
    // ...
    FeatureFlag::YourFeature,
];
```

不需要修改其他文件。

## 推广到 Preview

1. 添加到 `warp_core/src/features.rs` 的 `PREVIEW_FLAGS`。
2. 如果存在于 `DOGFOOD_FLAGS` 中，请移除。Preview flag 会自动包含在 Dogfood build 中。

```rust
pub const PREVIEW_FLAGS: &[FeatureFlag] = &[
    // ...
    FeatureFlag::YourFeature,
];
```

## 推广到 Stable

这需要修改**三个文件**。

### 1. `app/Cargo.toml`：添加到 `default`

将 snake_case feature name 添加到 `default = [...]` 数组：

```toml
default = [
    # ...
    "your_feature_name",
]
```

相比添加到 `RELEASE_FLAGS`，更推荐这种方式（见 `warp_core/src/features.rs:787-790` 附近注释）。它会把 feature 编译进所有 build，并支持一行 rollback。

### 2. `app/src/lib.rs`：添加到 `enabled_features()` bridge

按照现有模式，在 `enabled_features()` 的 `flags.extend([...])` block 中添加一个 `#[cfg(...)]` entry：

```rust
#[cfg(feature = "your_feature_name")]
FeatureFlag::YourFeature,
```

将它放在逻辑相关的 entry 附近。

### 3. `warp_core/src/features.rs`：从 `PREVIEW_FLAGS` / `DOGFOOD_FLAGS` 移除

从当前包含该 variant 的数组中移除它：

```rust
pub const PREVIEW_FLAGS: &[FeatureFlag] = &[
    // Remove FeatureFlag::YourFeature,
];
```

### 验证

```bash
./script/format
cargo clippy --workspace --all-targets --all-features --tests -- -D warnings
```

### 创建 follow-up Linear issue

PR 落地后，创建一个 Linear issue，提醒团队移除该 flag。使用 Linear MCP tool：

```
save_issue(
  title: "Remove FeatureFlag::YourFeature after stabilization",
  team: <your team>,
  assignee: "me",
  description: "FeatureFlag::YourFeature was promoted to Stable in <PR link>. Remove the flag and dead code branches after 1–2 release cycles. Follow the `remove-feature-flag` skill.",
  labels: ["tech-debt"],
  priority: 4  // Low
)
```
