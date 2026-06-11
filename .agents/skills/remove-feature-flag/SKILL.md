---
name: remove-feature-flag
description: 在 Warp 代码库中，当 feature flag 已经 rollout 并稳定后移除它。
---

# remove-feature-flag

在 Warp 代码库中，当 feature flag 已经 rollout 并稳定后移除它。

## 概述

当某个 feature flag 已为所有用户启用，并在 production 中稳定后，应移除该 flag，以减少技术债并简化代码库。这包括移除 flag 定义和所有条件检查。

## 何时移除

在以下情况下移除 feature flag：
- 该 feature 已在 `app/Cargo.toml` 的 `default` features 中启用
- 该 feature 已在 production 中稳定运行合理时间
- 没有计划禁用该 feature 或提供配置选项
- 团队同意该 feature 是永久性的

## 步骤

### 1. 从 app/Cargo.toml 移除

同时从 `[features]` section 和 `default` 数组中移除该 feature：

```toml
[features]
default = [
    # Remove "your_feature_name" from here
]

# Remove this line:
# your_feature_name = []
```

### 2. 从 FeatureFlag enum 移除

从 `warp_core/src/features.rs` 的 `FeatureFlag` enum 中移除该 variant：

```rust
#[derive(Sequence)]
pub enum FeatureFlag {
    // Remove YourFeatureName,
}
```

### 3. 从 app/src/lib.rs 移除

移除条件编译指令：

```rust
// Remove these lines:
// #[cfg(feature = "your_feature_name")]
// YourFeatureName,
```

### 4. 从 DOGFOOD_FLAGS/PREVIEW_FLAGS/RELEASE_FLAGS 移除

如果该 flag 列在 `features.rs` 的任何这些数组中，请移除它：

```rust
pub const DOGFOOD_FLAGS: &[FeatureFlag] = &[
    // Remove FeatureFlag::YourFeatureName,
];
```

### 5. 移除所有运行时检查和 dead code

在整个代码库中查找并移除所有 `FeatureFlag::YourFeatureName.is_enabled()` 检查：

**Before:**
```rust
if FeatureFlag::YourFeatureName.is_enabled() {
    // new behavior
} else {
    // old behavior (dead code)
}
```

**After:**
```rust
// new behavior (unconditionally enabled)
```

使用 ripgrep 查找所有出现位置：

```bash
rg "YourFeatureName" app/ warp_core/
```

### 6. 移除 keybinding predicate

如果 feature flag 用于 keybinding enabled predicate，请移除该 predicate：

**Before:**
```rust
EditableBinding::new(
    "action:name",
    "Action description",
    YourAction::Variant
)
.with_enabled(|| FeatureFlag::YourFeatureName.is_enabled())
.with_key_binding("cmdorctrl-key")
```

**After:**
```rust
EditableBinding::new(
    "action:name",
    "Action description",
    YourAction::Variant
)
.with_key_binding("cmdorctrl-key")
```

### 7. 清理 dead code branch

移除任何只在 feature 禁用时执行的代码路径（feature check 中的 `else` branch）。这些现在都是 dead code。

### 8. 运行测试和验证

移除 flag 后：

```bash
# Format and lint
./script/format
cargo clippy --workspace --all-targets --all-features --tests -- -D warnings

# Run tests
cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2

# Build the app
cargo run
```

## 最佳实践

- 不再需要 feature flag 后，应及时移除以减少技术债
- 移除 flag 时，移除所有相关代码（检查、dead branch、keybinding predicate）
- 使用 grep/ripgrep 确保已找到所有出现位置
- 移除后充分测试，确保没有 regression
- 考虑用单独 PR 移除 flag，便于审查

## 示例搜索命令

```bash
# Find all occurrences of the flag name
rg "YourFeatureName" app/ warp_core/

# Find feature flag checks
rg "FeatureFlag::YourFeatureName" app/

# Find cfg attributes
rg 'cfg\(feature = "your_feature_name"\)' app/
```
