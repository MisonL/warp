---
name: figma-generate-design
description: "与 figma-use 搭配使用，将应用页面、视图或多区块布局转换到 Figma。适用于用户想从代码或描述创建或更新完整页面、屏幕或视图时，会发现并复用设计系统组件、变量和样式，并按区块逐步组装。"
description_zh_CN: "与 figma-use 搭配使用，将应用页面、视图或多区块布局转换到 Figma。适用于用户想从代码或描述创建/更新完整页面、屏幕或视图时，会发现并复用设计系统组件、变量和样式。"
---

# 从 Design System 构建或更新 Figma Screen

使用本技能在 Figma 中创建或更新完整页面，并复用已发布 design system 中的 components、variables 和 styles，而不是用硬编码颜色和值画 primitive。

必须同时加载 `figma-use`，且每次 `use_figma` 调用都遵守其中的 Plugin API 规则。调用 `use_figma` 时传入 `skillNames: "figma-generate-design"`，这只是日志参数，不影响执行。

## 边界

- 交付物是新的或更新后的 Figma screen，且由 design system component instances 组成时使用。
- 若用户要从 Figma 设计生成代码，切换到 `figma-implement-design`。
- 若用户要创建新的可复用 component 或 variant，直接使用 `figma-use`。
- 若用户要写 Code Connect mapping，切换到 `figma-code-connect-components`。

## 前置条件

- Figma MCP server 已连接。
- 目标 Figma 文件有已发布 design system，或可访问 team library。
- 用户提供 Figma URL、file key，或说明要操作的文件。
- 有要构建或更新的 screen 源代码或描述。

## Web App 并行工作流

当来源是可在浏览器中渲染的 web app 时，最好并行做两件事：

1. 用本技能的 `use_figma` + design system component 工作流开始构建 screen。
2. 运行 `generate_figma_design` 捕获运行中 web app 的 pixel-perfect screenshot。

两者完成后，用 capture 作为视觉参考修正 `use_figma` 输出。capture 用于精确 spacing、sizing 和视觉处理；`use_figma` 输出保留正确 component instance 和 design system 连接。确认效果后删除 capture 输出。该并行流程只适用于 web app。

## 必需流程

### 第 1 步：理解 screen

写入 Figma 前先理解要构建什么：

1. 如果从代码构建，读取相关源码，理解页面结构、section 和所用组件。
2. 识别主要 section，例如 Header、Hero、Content Panels、Pricing Grid、FAQ、Footer。
3. 列出每个 section 涉及的 UI component，例如 button、input、card、nav pill、accordion。

### 第 2 步：发现 design system

需要发现三类资产：components、variables、styles。不要在已有 token 时硬编码 hex 或 pixel。

#### 2a. 发现 components

优先检查目标文件中已有 screen。如果已有使用同一 design system 的 screen，先用一次只读 `use_figma` 遍历现有 frame 的 INSTANCE，得到 component map。只有没有可参考 screen 时，才用 `search_design_system`。

搜索时要广泛使用多个词，例如 `button`、`input`、`nav`、`card`、`accordion`、`header`、`footer`、`tag`、`avatar`、`toggle`、`icon`。用 `includeComponents: true` 聚焦组件。

创建 component map 时包含 property 信息，尤其是 TEXT property key。可临时创建 instance，读取 `componentProperties` 和 nested instance properties 后删除。

#### 2b. 发现 variables

优先检查现有 screen 的 bound variables。也可以用 `search_design_system` 加 `includeVariables: true`。

注意两种方法不同：

- `figma.variables.getLocalVariableCollectionsAsync()` 只返回当前文件本地变量。为空不代表没有 design system variable。
- `search_design_system` 的 `includeVariables: true` 会搜索 linked library，包括 remote/published variables。

不要只因为 local variables 为空就断定没有 variables。搜索变量名时使用短查询和多种命名习惯：`gray`、`red`、`blue`、`brand`、`background`、`foreground`、`border`、`surface`、`text`、`space`、`radius`、`gap`、`padding`。

remote variable 用 `figma.variables.importVariableByKeyAsync(key)` import；local variable 用 `figma.variables.getVariableByIdAsync(id)`。

#### 2c. 发现 styles

用 `search_design_system` 加 `includeStyles: true` 搜索 `heading`、`body`、`shadow`、`elevation` 等。也可检查现有 screen 使用的 `textStyleId` 和 `effectStyleId`。library style 用 `figma.importStyleByKeyAsync(key)` import 后应用。

### 第 3 步：先创建 wrapper frame

不要把 section 作为 top-level page children 创建后再 reparent。跨 `use_figma` 调用移动节点可能静默失败并产生 orphan frame。先创建页面 wrapper，再直接把每个 section 构建在 wrapper 内。

wrapper 应放在已有内容右侧空白位置，返回 `wrapperId`，后续每个 section 脚本都从这个 ID 获取 wrapper。

### 第 4 步：逐个 section 构建

这是最关键步骤。每个 section 使用独立 `use_figma` 调用。在每个脚本开头通过第 3 步返回的 ID 获取 wrapper，并把新内容直接 append 到 wrapper。

构建时：

- import design system component by key。
- import variables 并用 bound variable 绑定 fills、padding、gap 等。
- import text/effect styles 并应用。
- 把 component instance 放进 section。
- append 到 wrapper 后再设置 `layoutSizingHorizontal = "FILL"` 等 fill sizing。
- 返回所有 created/mutated node IDs。

每个 section 完成后用 `get_screenshot` 验证，再继续。重点检查文本裁剪、重叠、间距和对齐。

### 覆盖 instance 文本

Component instance 中常有占位文本。优先使用第 2 步发现的 component property key，通过 `setProperties()` 覆盖：

```js
const nestedHeading = cardInstance.findOne(n => n.type === "INSTANCE" && n.name === "Text Heading");
if (nestedHeading) {
  nestedHeading.setProperties({ "Text#2104:5": "Actual heading from source code" });
}
```

只有文本不受 component property 管理时，才直接修改 `node.characters`。

### 仔细读取源码默认值

把代码组件翻译为 Figma instance 时，要查看组件源码中的默认 prop。比如 `<Button size="small">Register</Button>` 没有显式 `variant`，但组件定义可能默认 `variant = "primary"`。选错 variant 会造成明显视觉错误。

## 校验

每个 section 后：

- 用 `get_metadata` 检查层级、instance、mainComponent、ID。
- 用 `get_screenshot` 检查视觉。
- 修复后再继续，不要在错误结构上继续构建。

最终输出应总结：创建或更新的 screen、复用的 design system 资产、仍需用户确认的视觉差异。
