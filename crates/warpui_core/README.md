# WarpUI

## 快速导览

WarpUI 包含许多相互嵌套的概念。如果不引用系统中的其他部分，很难解释其中任意一部分。因此，本指南会先探索主要概念之间的关系，提供整体概览，然后再深入其中某个概念的细节。

Rust 严格的 ownership 规则对用户界面是一项挑战，因为多方向数据流通常很关键。如果每个 object 都有且只有一个 owner，我们该如何表达 event handler 这类事物？

## 全局 App object、entity 和 handle

WarpUI 使用 `App` object 解决这个问题。`App` object 是应用中所有 view 和 model 的唯一 owner。我们将 view 和 model 统称为 **entity**。Entity 可以通过 **handle** 持有对其他 entity 的引用。Handle 会在特定且受限的场景中提供对 entity 的访问。以多 tab terminal 为例：应用 window 由一个 `WorkspaceView` 占据，我们希望这个 workspace 包含多个 `TerminalView`。`Workspace` 不会直接持有这些 `TerminalView`，而是持有一个 `ViewHandle<TerminalView>` 的 vector。

```rust
struct WorkspaceView {
    sessions: Vec<ViewHandle<TerminalView>>,
}
```

`ViewHandle` 本身做不了太多事情。它的存在会阻止被引用的 view 被全局 `App` object 丢弃，但它不提供对被引用 view 的直接访问。Handle 基本上是一个更强一点的 identifier。要将 handle 转换成实际引用，需要一个 **app context** object 的引用；该 object 会由全局 `App` object 在特定时间点提供。

其中一个时间点可以是 `WorkspaceView` 上的 `render` 方法。每当 workspace 的屏幕表示更新时，框架会调用该方法。`render` 的参数之一是 `&AppContext`，它可以传给 `ViewHandle` 上的 `as_ref` 方法，以获取底层 object 的引用。

下面的代码示例省略了很多细节，以便专注于当前主题。假设我们想知道所有 terminal session 的标题，以便将它们渲染成 tab：

```rust
impl View for WorkspaceView {
    fn render<'a>(&self, ..., ctx: &AppContext) -> ... {
        let titles = self.sessions.iter().map(|handle| handle.as_ref(ctx).title()).collect::<Vec<String>>;
        ...
    }

    ...
}
```

从 `render` 方法返回，并失去调用期间提供的 `&AppContext` 参数后，我们就不再能访问这些 handle 引用的 terminal view。

当然，entity 也可以直接拥有任何它需要的 state。只有当一个 entity 需要引用另一个 entity 时才需要 handle。随着我们继续勾勒系统更多方面，何时将一块应用状态表达为自己的 entity 会变得更清晰。

## Element

框架要求所有 view 实现 `View` trait，而该 trait 的关键方法之一是上面展示过的 `render`。这个方法的职责是根据 view 当前状态计算其视觉描述，并且每当 view 状态变化时都会被调用。

为了描述 view 的外观，render 会返回一个 **element**。一个 view 可以存在任意长时间，并随着用户与应用交互而改变状态；而 element 被设计为只存在单帧。更准确地说，未变化 view 返回的 element 会跨多帧复用，但从概念上可以将 element 视为一次性 object：每当返回它的 view 发生变化时，就会被丢弃并替换。

框架内置了多个可组合 element，用于执行常见任务，例如绘制背景和边框、添加 padding、渲染 label text、水平和垂直布局 element、处理 event 等。内置 element 大致基于 Flutter framework。定义自己的自定义 element 也很直接，可以让你细粒度控制 layout，并通过硬件加速的 `Scene` API 以 imperative 方式在 scene 上绘制像素。

## Action

### Action handler

### Action dispatch

## View
