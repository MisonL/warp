# 输入框

## 1. 输入框 API

输入框是一个名为 `EditorView` 的子 view，在 `TerminalView` 中实例化。

如果 editor view 要与 terminal view 通信，应让 editor view 发送 action，并在父 view 中注册该 action。

如果 terminal view 要与 editor view 通信，terminal view 可以通过其 struct 中的字段直接访问 editor view。例如，为了输入换行，它会调用：

```
fn input_newline(&mut self, _: &(), ctx: &mut ViewContext<Self>) {
...
    self.input.update(ctx, |input, ctx| input.insert(&'\n'.into(), ctx));
...
```

## 2. Editor

### Buffer

`EditorView` 中包含文本的数据结构是 `Buffer`。

`Buffer` 最常用的 API 是 `chars_at()`，它会返回一个 iterator，可用于从某个位置开始遍历字符。

底层实现中，buffer 是一个 `SumTree<Fragment>`。每个 `Fragment` 都有一个 `Text`。`Text` 由 `text` 和 `runs` 组成，其中 `text` 是 `Arc<str>`，`runs` 是 `SumTree<Run>`。run 描述 fragment 占用的空间。

`Buffer` 的另一个有用 API 是 `line_len(row_number)`，它会给出某一行号对应行的长度。

### 索引 Buffer

我们使用 `Point` 和 `Offset` 来索引 buffer。`Point` 是 `Buffer` 内带有 `row` 和 `column` 的二维位置。`Offset` 是 `Buffer` 内的一维 `usize`。你可以通过 `to_offset()` 轻松将 `Point` 和 `Anchor` 转换为 `Offset`。`Offset` 适合遍历字符，因为不需要关心 row 和 column 数字变化。

### DisplayPoint 与 DisplayMap

另一个带有 `row` 和 `column` 的 struct 是 `DisplayPoint`。`DisplayPoint` 只与 `EditorView` 和 `DisplayMap` 相关，与 `Buffer` 无关。
`DisplayPoint` 描述 `DisplayMap` 中的位置。`DisplayMap` 描述 point 如何**显示**，也就是视觉坐标系统。`Buffer` 并不知道自身如何被显示，事实上它可以被显示在多个 view 中。

当发生代码折叠或 soft wrapping 时，`DisplayPoint` 和 `Point` 的值会不同。我们可以使用 `DisplayMap` 在 `DisplayPoint` 和 `Point` 之间转换。

### Selection 与 Cursor

输入框支持我们在 VSCode 中熟悉的多 selection。因此我们的 `EditorView` 有一个 `Vec<Selection>`。

`Selection` 有一个 `start` anchor 和一个 `end` anchor，用来表示其起止位置。

Selection 和 cursor 紧密交织：凡是有 selection 的地方就有 cursor。**单独的 cursor 只是一个 `start==end` 的空 selection。** 因此始终至少有一个 `Selection`，其中第一个 selection 就是 cursor。

### Anchor

`Anchor` 是文本中的一个书签。即使其绝对位置已经改变，它也允许我们索引文本中的相对位置。`Anchor` 可以转换为 `DisplayPoint` 或 `Point`。

例如，假设我的 cursor 位于 'PartialEq' 中字符 'l' 和 'E' 之间。假设绝对位置是第 3 行第 7 列：

```
Partial|Eq
```

然后我们添加 10 个字符：

```
Partial1234567890|Eq
```

cursor 的绝对位置现在是第 3 行第 17 列。为了轻松计算该位置，我们可以将 `Anchor` 转换为第 3 行第 17 列。

#### `AnchorBias::Left`、`AnchorBias::Right`

AnchorBias 决定 cursor 最终位于插入文本的右侧还是左侧。上面的示例是 AnchorBias 为 `Right` 的情况。`AnchorBias::Left` 如下：

```
Partial|1234567890Eq
```

当我们引入协同编辑时，`Anchor` 最有用；即使另一个用户在当前用户所在行中插入文本，也必须让用户知道自己正在何处输入。
