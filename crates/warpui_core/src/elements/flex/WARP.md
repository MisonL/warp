# Flex Element 调试指南

本指南帮助诊断和修复 WarpUI 中常见的 Flex layout panic。

## 快速参考：错误消息 -> 修复

### Error: `flex contains flexible children but has an infinite constraint along the flex axis`

**原因**：带有 `MainAxisSize::Min`（默认值）的 `Flex` 包含 `Expanded` 或 `Shrinkable` child，但在主轴方向没有 max constraint。

**修复**（按优先顺序）：
1. 如果不需要增长，从 child 中移除 `Expanded`/`Shrinkable`
2. 使用 `ConstrainedBox` 给 `Flex` 或某个 ancestor 添加 max constraint：
   - 对 `Flex::row()`：添加 `max_width`
   - 对 `Flex::column()`：添加 `max_height`
3. 如果该 `Flex` 位于另一个 `Flex` 内，确保 parent 向下传递 bounded constraint

### Error: `A flex that should expand to a max space can't be rendered in an infinite max constraint`

**原因**：某个 `Flex` 使用 `MainAxisSize::Max`，但没有 ancestor 提供最大尺寸约束。

**修复**：
1. 如果该 Flex 不需要填满 parent，请移除 `.with_main_axis_size(MainAxisSize::Max)`（使用默认 `MainAxisSize::Min`）
2. 使用带 max constraint 的 `ConstrainedBox` 约束该 `Flex` 或某个 ancestor

## 关键概念

### 两类 Child

- **Flexible child**（`Expanded`、`Shrinkable`）：尺寸通过按 flex ratio 划分剩余空间来计算
- **Non-flexible child**：使用其 intrinsic size 布局，但仍遵守 parent `Flex` 传来的 max constraint

### 重要行为

1. **`Expanded` 只能作为 `Flex` 的直接 child 工作** - 包在 `Container`/`ConstrainedBox` 中会破坏它

2. **`Expanded` 不强制增长** - 与 CSS `flex-grow` 不同，它只授予增长的*能力*。像 `Text` 这样的 element 默认不会扩展；如有需要请包在 `Align` 中

3. **`MainAxisSize::Min` + `Expanded` = 实际上是 `MainAxisSize::Max`** - `Expanded` child 仍会增长以填充可用空间

4. **嵌套 `MainAxisSize::Max` 的 `Flex`** - 将一个带 `MainAxisSize::Max` 的 `Flex` 放入另一个带 `MainAxisSize::Max` 的 `Flex` 内，且两者都没有从 ancestor 接收 max constraint 时，会导致 layout panic

## 常见模式

### 水平居中（需要 bounded parent）

```rust
Flex::row()
    .with_main_axis_size(MainAxisSize::Max)
    .with_main_axis_alignment(MainAxisAlignment::Center)
    .with_child(element)
    .finish()
```

### 垂直居中（需要 bounded parent）

```rust
Flex::row()
    .with_cross_axis_alignment(CrossAxisAlignment::Center)
    .with_child(element)
    .finish()
```

### 分隔两组内容（例如左右对齐项）

```rust
Flex::row()
    .with_child(left_element)
    .with_child(Expanded::new(1.0, Empty::new()))  // spacer
    .with_child(right_element)
    .finish()
```

## 调试技巧

1. 使用 `RUST_BACKTRACE=full` 运行，以识别导致 panic 的 element
2. 检查 element hierarchy 中是否存在 unbounded `Flex` container
3. 从 root 到失败 element 跟踪 constraint，找到 max constraint 丢失的位置
4. 避免不必要的 `MainAxisSize::Max`，只有当 `Flex` *必须*填满 parent 时才使用
