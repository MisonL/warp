---
name: figma-generate-design
description: "当任务涉及把 application page、view 或 multi-section layout 转换到 Figma 时，与 figma-use 一起使用此 skill。触发语句：'write to Figma'、'create in Figma from code'、'push page to Figma'、'take this app/page and build it in Figma'、'create a screen'、'build a landing page in Figma'、'update the Figma screen to match code'。当用户希望根据代码或描述在 Figma 中构建或更新 full page、screen 或 view 时，这是首选 workflow skill。通过 search_design_system 发现 design system component、variable 和 style，导入它们，并使用 design system token 而不是 hardcoded value，逐 section 增量组装 screen。"
---

# 从 Design System 构建 / 更新 Screen

使用此 skill 通过**复用已发布的 design system**（component、variable 和 style）在 Figma 中创建或更新 full-page screen，而不是用 hardcoded value 绘制 primitive。关键点是：Figma file 很可能有已发布的 design system，其中的 component、color/spacing variable 和 text/effect style 对应代码库的 UI component 和 token。找到并使用这些资源，而不是用 hex color 画 box。

**强制要求**：任何 `use_figma` 调用前，都必须同时加载 [figma-use](../figma-use/SKILL.md)。该 skill 包含适用于你编写的每个 script 的关键规则（color range、font loading 等）。

**作为此 skill 的一部分调用 `use_figma` 时，始终传入 `skillNames: "figma-generate-design"`。** 这是 logging 参数，不影响执行。

## Skill 边界

- 当交付物是由 design system component instance 组成的 **Figma screen**（新建或更新）时，使用此 skill。
- 如果用户想根据 Figma design 生成**代码**，切换到 [figma-implement-design](../figma-implement-design/SKILL.md)。
- 如果用户想创建**新的可复用 component 或 variant**，直接使用 [figma-use](../figma-use/SKILL.md)。
- 如果用户想编写 **Code Connect mapping**，切换到 [figma-code-connect-components](../figma-code-connect-components/SKILL.md)。

## 前置条件

- Figma MCP server 必须已连接
- 目标 Figma file 必须有包含 component 的已发布 design system，或可访问 team library
- 用户应提供以下任一项：
  - 要操作的 Figma file URL / file key
  - 或关于目标文件的上下文（agent 可以发现 page）
- 要构建或更新的 screen 的源代码或描述

## 与 generate_figma_design 并行的工作流（仅 Web App）

当从可在浏览器中渲染的 **web app** 构建 screen 时，并行运行两种方法效果最好：

1. **并行执行：**
   - 使用此 skill 的工作流开始构建 screen（use_figma + design system component）
   - 运行 `generate_figma_design`，捕获运行中 web app 的 pixel-perfect screenshot
2. **两者都完成后：** 更新 use_figma 输出，使其匹配 `generate_figma_design` capture 得到的 pixel-perfect layout。capture 提供应对齐的精确 spacing、sizing 和 visual treatment，而 use_figma 输出具有正确链接到 design system 的 component instance。
3. **确认效果良好后：** 删除 `generate_figma_design` 输出，它只用作视觉参考。

这结合了两者优势：`generate_figma_design` 提供 pixel-perfect layout accuracy，而 use_figma 提供正确、保持链接且可更新的 design system component instance。

**此工作流只适用于 web app**，即 `generate_figma_design` 可以捕获运行页面的情况。对于非 web app（iOS、Android 等）或更新现有 screen，请使用下面的标准工作流。

## 必需工作流

**按顺序执行这些步骤。不要跳过步骤。**

### 步骤 1：理解 Screen

接触 Figma 前，先理解要构建什么：

1. 如果从代码构建，读取相关源文件，理解 page 结构、section 以及使用了哪些 component。
2. 识别 screen 的主要 section（例如 Header、Hero、Content Panels、Pricing Grid、FAQ Accordion、Footer）。
3. 对每个 section，列出涉及的 UI component（button、input、card、navigation pill、accordion 等）。

### 步骤 2：发现 Design System - Component、Variable 和 Style

你需要从 design system 获取三类内容：**component**（button、card 等）、**variable**（color、spacing、radius）和 **style**（text style、shadow 等 effect style）。当 design system token 存在时，不要 hardcode hex color 或 pixel value。

#### 2a：发现 component

**首选：先检查现有 screen。** 如果目标文件已经包含使用同一 design system 的 screen，跳过 `search_design_system`，直接检查现有 instance。一个遍历现有 frame instance 的 `use_figma` 调用，就能给出精确且权威的 component map：

```js
const frame = figma.currentPage.findOne(n => n.name === "Existing Screen");
const uniqueSets = new Map();
frame.findAll(n => n.type === "INSTANCE").forEach(inst => {
  const mc = inst.mainComponent;
  const cs = mc?.parent?.type === "COMPONENT_SET" ? mc.parent : null;
  const key = cs ? cs.key : mc?.key;
  const name = cs ? cs.name : mc?.name;
  if (key && !uniqueSets.has(key)) {
    uniqueSets.set(key, { name, key, isSet: !!cs, sampleVariant: mc.name });
  }
});
return [...uniqueSets.values()];
```

只有当文件中没有可参考的现有 screen 时，才 fallback 到 `search_design_system`。使用它时要**广泛搜索**，尝试多个术语和同义词（例如 "button"、"input"、"nav"、"card"、"accordion"、"header"、"footer"、"tag"、"avatar"、"toggle"、"icon" 等）。使用 `includeComponents: true` 专注于 component。

**在 map 中包含 component property**，你需要知道每个 component 暴露了哪些 TEXT property 来覆盖文本。创建一个临时 instance，读取它的 `componentProperties`（以及 nested instance 的对应属性），然后删除临时 instance。

带 property 信息的 component map 示例：

```
Component Map:
- Button → key: "abc123", type: COMPONENT_SET
  Properties: { "Label#2:0": TEXT, "Has Icon#4:64": BOOLEAN }
- PricingCard → key: "ghi789", type: COMPONENT_SET
  Properties: { "Device": VARIANT, "Variant": VARIANT }
  Nested "Text Heading" has: { "Text#2104:5": TEXT }
  Nested "Button" has: { "Label#2:0": TEXT }
```

#### 2b：发现 variable（color、spacing、radius）

**先检查现有 screen**（与 component 相同）。或使用带 `includeVariables: true` 的 `search_design_system`。

> **警告：有两种不同的 variable discovery 方法，不要混淆。**
>
> - `use_figma` 配合 `figma.variables.getLocalVariableCollectionsAsync()` - 只返回**当前文件中定义的 local variable**。如果它返回空，并不表示没有 variable。Remote/published library variable 对此 API 不可见。
> - `search_design_system` 配合 `includeVariables: true` - 会搜索**所有已链接 library**，包括 remote 和 published library。这是发现 design system variable 的正确工具。
>
> **绝不要只因为 `getLocalVariableCollectionsAsync()` 返回空，就断定 "no variables exist"。** 在决定自己创建 variable 前，始终也运行带 `includeVariables: true` 的 `search_design_system` 来检查 library variable。

**Query 策略：** `search_design_system` 匹配的是 **variable name**（例如 "Gray/gray-9"、"core/gray/100"、"space/400"），不是类别。并行运行多个短小、简单的 query，而不是一个复合 query：

- **Primitive colors：** "gray"、"red"、"blue"、"green"、"white"、"brand"
- **Semantic colors：** "background"、"foreground"、"border"、"surface"、"text"
- **Spacing/sizing：** "space"、"radius"、"gap"、"padding"

如果初始搜索为空，尝试更短片段或不同命名约定。不同 library 差异很大（"grey" vs "gray"、"spacing" vs "space"、"color/bg" vs "background"）。

检查现有 screen 的 bound variable 可获得最权威结果：

```js
const frame = figma.currentPage.findOne(n => n.name === "Existing Screen");
const varMap = new Map();
frame.findAll(() => true).forEach(node => {
  const bv = node.boundVariables;
  if (!bv) return;
  for (const [prop, binding] of Object.entries(bv)) {
    const bindings = Array.isArray(binding) ? binding : [binding];
    for (const b of bindings) {
      if (b?.id && !varMap.has(b.id)) {
        const v = await figma.variables.getVariableByIdAsync(b.id);
        if (v) varMap.set(b.id, { name: v.name, id: v.id, key: v.key, type: v.resolvedType, remote: v.remote });
      }
    }
  }
});
return [...varMap.values()];
```

对于 library variable（remote = true），使用 `figma.variables.importVariableByKeyAsync(key)` 按 key 导入。对于 local variable，直接使用 `figma.variables.getVariableByIdAsync(id)`。

绑定 pattern 见 [variable-patterns.md](../figma-use/references/variable-patterns.md)。

#### 2c：发现 style（text style、effect style）

用带 `includeStyles: true` 的 `search_design_system` 搜索 style，搜索词可用 "heading"、"body"、"shadow"、"elevation"。也可以检查现有 screen 使用了什么：

```js
const frame = figma.currentPage.findOne(n => n.name === "Existing Screen");
const styles = { text: new Map(), effect: new Map() };
frame.findAll(() => true).forEach(node => {
  if ('textStyleId' in node && node.textStyleId) {
    const s = figma.getStyleById(node.textStyleId);
    if (s) styles.text.set(s.id, { name: s.name, id: s.id, key: s.key });
  }
  if ('effectStyleId' in node && node.effectStyleId) {
    const s = figma.getStyleById(node.effectStyleId);
    if (s) styles.effect.set(s.id, { name: s.name, id: s.id, key: s.key });
  }
});
return {
  textStyles: [...styles.text.values()],
  effectStyles: [...styles.effect.values()]
};
```

用 `figma.importStyleByKeyAsync(key)` 导入 library style，然后用 `node.textStyleId = style.id` 或 `node.effectStyleId = style.id` 应用。

详情见 [text-style-patterns.md](../figma-use/references/text-style-patterns.md) 和 [effect-style-patterns.md](../figma-use/references/effect-style-patterns.md)。

### 步骤 3：先创建 Page Wrapper Frame

**不要把 section 构建成顶层 page child 再稍后 reparent**，跨 `use_figma` 调用用 `appendChild()` 移动 node 会静默失败，并产生 orphaned frame。应先创建 wrapper，然后直接在其中构建每个 section。

在独立的 `use_figma` 调用中创建 page wrapper。将它放在远离现有内容的位置，并返回其 ID：

```js
// Find clear space
let maxX = 0;
for (const child of figma.currentPage.children) {
  maxX = Math.max(maxX, child.x + child.width);
}

const wrapper = figma.createFrame();
wrapper.name = "Homepage";
wrapper.layoutMode = "VERTICAL";
wrapper.primaryAxisAlignItems = "CENTER";
wrapper.counterAxisAlignItems = "CENTER";
wrapper.resize(1440, 100);
wrapper.layoutSizingHorizontal = "FIXED";
wrapper.layoutSizingVertical = "HUG";
wrapper.x = maxX + 200;
wrapper.y = 0;

return { success: true, wrapperId: wrapper.id };
```

### 步骤 4：在 Wrapper 内构建每个 Section

**这是最重要的步骤。** 每次构建一个 section，每个 section 都放在自己的 `use_figma` 调用中。每个 script 开头都通过 ID 获取 wrapper，并把新内容直接 append 到其中。

```js
const createdNodeIds = [];
const wrapper = await figma.getNodeByIdAsync("WRAPPER_ID_FROM_STEP_3");

// Import design system components by key
const buttonSet = await figma.importComponentSetByKeyAsync("BUTTON_SET_KEY");
const primaryButton = buttonSet.children.find(c =>
  c.type === "COMPONENT" && c.name.includes("variant=primary")
) || buttonSet.defaultVariant;

// Import design system variables for colors and spacing
const bgColorVar = await figma.variables.importVariableByKeyAsync("BG_COLOR_VAR_KEY");
const spacingVar = await figma.variables.importVariableByKeyAsync("SPACING_VAR_KEY");

// Build section frame with variable bindings (not hardcoded values)
const section = figma.createFrame();
section.name = "Header";
section.layoutMode = "HORIZONTAL";
section.setBoundVariable("paddingLeft", spacingVar);
section.setBoundVariable("paddingRight", spacingVar);
const bgPaint = figma.variables.setBoundVariableForPaint(
  { type: 'SOLID', color: { r: 0, g: 0, b: 0 } }, 'color', bgColorVar
);
section.fills = [bgPaint];

// Import and apply text/effect styles
const shadowStyle = await figma.importStyleByKeyAsync("SHADOW_STYLE_KEY");
section.effectStyleId = shadowStyle.id;

// Create component instances inside the section
const btnInstance = primaryButton.createInstance();
section.appendChild(btnInstance);
createdNodeIds.push(btnInstance.id);

// Append section to wrapper
wrapper.appendChild(section);
section.layoutSizingHorizontal = "FILL"; // AFTER appending

createdNodeIds.push(section.id);
return { success: true, createdNodeIds };
```

每个 section 完成后，先用 `get_screenshot` 验证，再继续下一步。特别留意 cropped/clipped text（line height 截断内容）和重叠元素，这些是最常见且容易一眼漏掉的问题。

#### 用 setProperties() 覆盖 instance 文本

Component instance 会带有 placeholder text（"Title"、"Heading"、"Button"）。使用步骤 2 中发现的 component property key，通过 `setProperties()` 将其覆盖为真实文本。这比直接操作 `node.characters` 更可靠。完整 pattern 见 [component-patterns.md](../figma-use/references/component-patterns.md#overriding-text-in-a-component-instance)。

对于暴露自身 TEXT property 的 nested instance，在 nested instance 上调用 `setProperties()`：

```js
const nestedHeading = cardInstance.findOne(n => n.type === "INSTANCE" && n.name === "Text Heading");
if (nestedHeading) {
  nestedHeading.setProperties({ "Text#2104:5": "Actual heading from source code" });
}
```

只有当文本不受任何 component property 管理时，才 fallback 到直接修改 `node.characters`。

#### 仔细读取 source code default

将 code component 转换为 Figma instance 时，要检查源代码中的 component default prop value，而不仅是显式传入的内容。例如 `<Button size="small">Register</Button>` 没有 variant prop，应检查 component definition，找到默认的 `variant = "primary"`。选择错误 variant（例如 Neutral 而不是 Primary）会产生容易漏看的视觉错误。

#### 手动构建与从 design system 导入的边界

| 手动构建 | 从 design system 导入 |
|----------------|--------------------------|
| Page wrapper frame | **Component**：button、card、input、nav 等 |
| Section container frame | **Variable**：color（fill、stroke）、spacing（padding、gap）、radius |
| Layout grid（row、column） | **Text style**：heading、body、caption 等 |
| | **Effect style**：shadow、blur 等 |

当 design system variable 存在时，**绝不要 hardcode hex color 或 pixel spacing**。使用 `setBoundVariable` 处理 spacing/radius，使用 `setBoundVariableForPaint` 处理 color。使用 `node.textStyleId` 应用 text style，使用 `node.effectStyleId` 应用 effect style。

### 步骤 5：验证完整 Screen

所有 section 组合完成后，对 full page frame 调用 `get_screenshot`，并与 source 对比。使用定向 `use_figma` 调用修复问题，不要重建整个 screen。

**对单个 section 截图，而不只是截 full page。** 缩小分辨率的 full-page screenshot 会隐藏 text truncation、错误 color 和未覆盖的 placeholder text。按 node ID 截取每个 section，检查：
- **Cropped/clipped text** - line height 或 frame sizing 截断 descender、ascender 或整行
- **Overlapping content** - 由于 sizing 错误或缺失 auto-layout，元素互相堆叠
- 仍在显示 placeholder text（"Title"、"Heading"、"Button"）
- layout sizing bug 导致的内容截断
- 错误 component variant（例如 Neutral vs Primary button）

### 步骤 6：更新现有 Screen

更新而不是从头创建时：

1. 使用 `get_metadata` 检查现有 screen 结构。
2. 识别哪些 section 需要更新，哪些可以保留。
3. 对每个需要变更的 section：
   - 通过 ID 或名称定位现有 node
   - 如果 design system component 已变化，替换 component instance
   - 按需更新 text content、variant property 或 layout
   - 移除 deprecated section
   - 添加新 section
4. 每次修改后用 `get_screenshot` 验证。

```js
// Example: Swap a button variant in an existing screen
const existingButton = await figma.getNodeByIdAsync("EXISTING_BUTTON_INSTANCE_ID");
if (existingButton && existingButton.type === "INSTANCE") {
  // Import the updated component
  const buttonSet = await figma.importComponentSetByKeyAsync("BUTTON_SET_KEY");
  const newVariant = buttonSet.children.find(c =>
    c.name.includes("variant=primary") && c.name.includes("size=lg")
  ) || buttonSet.defaultVariant;
  existingButton.swapComponent(newVariant);
}
return { success: true, mutatedNodeIds: [existingButton.id] };
```

## 参考文档

如需详细 API pattern 和 gotcha，请按需加载 [figma-use](../figma-use/SKILL.md) references 中的这些文档：

- [component-patterns.md](../figma-use/references/component-patterns.md) - 按 key 导入、查找 variant、setProperties、text override、使用 instance
- [variable-patterns.md](../figma-use/references/variable-patterns.md) - 创建/绑定 variable、导入 library variable、scope、alias、发现现有 variable
- [text-style-patterns.md](../figma-use/references/text-style-patterns.md) - 创建/应用 text style、导入 library text style、type ramp
- [effect-style-patterns.md](../figma-use/references/effect-style-patterns.md) - 创建/应用 effect style（shadow）、导入 library effect style
- [gotchas.md](../figma-use/references/gotchas.md) - layout 陷阱（HUG/FILL interaction、counterAxisAlignItems、sizing order）、paint/color 问题、page context reset

## 错误恢复

遵循 [figma-use](../figma-use/SKILL.md#6-error-recovery--self-correction) 中的错误恢复流程：

1. 出错时**停止**，不要立即重试。
2. **仔细阅读错误消息**，理解出错原因。
3. 如果错误不清楚，调用 `get_metadata` 或 `get_screenshot` 检查当前 file state。
4. 根据错误消息**修复 script**。
5. **重试**修正后的 script。由于失败 script 是 atomic 的（script 出错时不会创建任何内容），这一步是安全的。

由于此 skill 是增量工作的（每次调用一个 section），错误天然限定在单个 section 内。之前成功调用创建的 section 会保持完整。

## 最佳实践

- **构建前始终搜索。** Design system 很可能已有你需要的 component、variable 或 style。手动构建和 hardcoded value 应是例外，而不是规则。
- **广泛搜索。** 尝试同义词和部分词。一个 "NavigationPill" 可能通过 "pill"、"nav"、"tab" 或 "chip" 找到。对于 variable，搜索 "color"、"spacing"、"radius" 等。
- **优先使用 design system token，而不是 hardcoded value。** 对 color、spacing 和 radius 使用 variable binding。对 typography 使用 text style。对 shadow 使用 effect style。这样 screen 会持续链接到 design system。
- **优先使用 component instance，而不是手动构建。** Instance 会保持链接到 source component，并在 design system 演进时自动更新。
- **逐 section 工作。** 每个 `use_figma` 调用最多构建一个主要 section。
- **每次调用都返回 node ID。** 组合 section 和错误恢复时会用到。
- **每个 section 后都做视觉验证。** 用 `get_screenshot` 尽早发现问题。
- **匹配现有约定。** 如果文件已有 screen，匹配其 naming、sizing 和 layout pattern。
