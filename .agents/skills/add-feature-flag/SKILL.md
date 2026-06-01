---
name: add-feature-flag
description: 在 Warp 代码库中新增 feature flag，用于对代码变更进行门控。
---

# add-feature-flag

在 Warp 代码库中新增 feature flag，用于对代码变更进行门控。

## 概述

Warp 中的 feature flag 是编译期开关，允许针对不同 channel（例如 Dev、Stable）选择性启用功能。它们使用一层小型运行时管线来检查 flag 是否已启用。

## 步骤

### 1. 添加到 Cargo.toml

将该 feature 添加到 `app/Cargo.toml` 的 `[features]` section 下，但**不要**添加到嵌套的 `default` stanza 中：

```toml
[features]
your_feature_name = []
```

### 2. 添加到 FeatureFlag enum

在 `warp_core/src/features.rs` 的 `FeatureFlag` enum 中添加一个新 variant：

```rust
#[derive(Sequence)]
pub enum FeatureFlag {
    YourFeatureName,
}
```

### 3. 添加条件编译指令

将该 feature 添加到 `app/src/lib.rs`，并配上对应的 `#[cfg(feature = "...")]` attribute，以确保只有启用时才包含它：

```rust
#[cfg(feature = "your_feature_name")]
YourFeatureName,
```

### 4. 使用运行时检查门控代码

在代码中使用运行时检查，有条件地执行 feature-gated code：

```rust
if FeatureFlag::YourFeatureName.is_enabled() {
    // feature-gated behavior
}
```

### 5.（可选）为 dogfood 构建启用

要在 Dev/dogfood 构建中默认启用该 feature，请将它添加到 `features.rs` 中的 `DOGFOOD_FLAGS` 数组：

```rust
pub const DOGFOOD_FLAGS: &[FeatureFlag] = &[
    FeatureFlag::YourFeatureName,
];
```

### 6. 带 feature flag 运行

要在本地启用该 feature 进行测试：

```bash
cargo run --features your_feature_name

# Multiple features:
cargo run --features your_feature_name,another_feature
```

## 带 Feature Flag 的 Keybinding

如果要添加属于 gated feature 的 `EditableBinding` 或 `FixedBinding`，请包含一个检查 feature flag 的 enabled predicate。这样可以避免 feature 禁用时，该 keybinding 出现在键盘设置中。

示例：

```rust
EditableBinding::new(
    "action:name",
    "Action description",
    YourAction::Variant
)
.with_enabled(|| FeatureFlag::YourFeatureName.is_enabled())
.with_key_binding("cmdorctrl-key")
```

## 推广到 Stable

当准备为所有 Warp Stable 用户启用该 feature 时，将它添加到 `app/Cargo.toml` 的 `default` 数组：

```toml
[features]
default = [
    "your_feature_name",
    # other default features...
]
```

## 最佳实践

- **优先使用运行时检查而不是 cfg 指令**：尽可能使用 `FeatureFlag::YourFeatureName.is_enabled()`，而不是 `#[cfg(...)]`，这样 flag 无需重新编译即可切换，后续也更容易清理
- 仅当没有该 flag 代码就无法编译时（例如平台相关代码或缺失依赖），才使用 `#[cfg(...)]`
- 保持 flag 处于高层级并面向产品，不要按调用点创建 flag
- 发布稳定后移除 flag 和 dead branch
