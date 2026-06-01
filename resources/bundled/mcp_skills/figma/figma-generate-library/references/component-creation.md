> 属于 [figma-generate-library skill](../SKILL.md) 的一部分。

# Component 创建参考

Phase 3 完整指南：使用 variant matrix、variable binding、component property 和文档构建 component。

---

## 1. Component 架构

### 依赖顺序：Atoms 先于 Molecules

始终按 dependency order 构建。包含 atom instance 的 molecule 必须等 atom 发布后才能存在。建议顺序：

```
Tier 0 (atoms): Icon, Avatar, Badge, Spinner
Tier 1 (molecules): Button, Checkbox, Toggle, Input, Select
Tier 2 (organisms): Card, Dialog, Menu, Navigation, Form
```

如果某个 component 嵌入了另一个 component 的 instance，必须先创建被嵌入的 component。在 Phase 0 构建 dependency graph，并把创建顺序写入计划。

### Building Blocks 子组件（M3 模式）

对于拥有独立子元素 state machine 的复杂 component，把子元素抽取成自己的 component set，并使用 `Building Blocks/`（public）或 `.Building Blocks/`（在 assets panel 中隐藏）前缀。点前缀是 Figma 用来从 public assets panel 中隐藏 component 的约定。

**何时使用 Building Blocks：**
- 子元素有自己的 variant axes（state、selection），会导致 parent 中出现组合爆炸
- 子元素会重复出现（nav item、table cell、calendar cell、segmented button segment）
- 子元素的 variant axes 与 parent 不同

**示例（M3 Segmented Button）：**
```
Building Blocks/Segmented button/Button segment (start)   [27 variants: Config × State × Selected]
Building Blocks/Segmented button/Button segment (middle)  [27 variants]
Building Blocks/Segmented button/Button segment (end)     [27 variants]

Segmented button  [16 variants: Segments=2-5 × Density=0/-1/-2/-3]
  Each variant contains instances of the appropriate Building Block segment components.
```

Parent component 管理 composition 和 configuration；Building Block 管理自己的 interaction state。

### Private Component（`__` 前缀）

对不应出现在 team library 中的 internal helper component 使用 `__` 前缀（Shop Minis pattern）。对 documentation-only component 使用 `_`（UI3 pattern）。

```
__asset          // private icon/asset holder
_Label/Direction // documentation annotation helper
```

---

## 2. 创建 Component Page

每个 component 都放在自己的专用 page 上（默认每个 component 一个 page）。该 page 包含左上角的 documentation frame，以及位于其右侧或下方的 component set。

```javascript
// Create or find the component page
let page = figma.root.children.find(p => p.name === 'Button');
if (!page) {
  page = figma.createPage();
  page.name = 'Button';
}
await figma.setCurrentPageAsync(page);

// Documentation frame — positioned at (40, 40)
const docFrame = figma.createFrame();
docFrame.name = 'Button / Documentation';
docFrame.x = 40;
docFrame.y = 40;
docFrame.resize(600, 400);
docFrame.fills = [{ type: 'SOLID', color: { r: 1, g: 1, b: 1 } }];
docFrame.layoutMode = 'VERTICAL';
docFrame.primaryAxisSizingMode = 'AUTO';
docFrame.counterAxisSizingMode = 'FIXED';
docFrame.paddingTop = 40;
docFrame.paddingBottom = 40;
docFrame.paddingLeft = 40;
docFrame.paddingRight = 40;
docFrame.itemSpacing = 16;

// Title text node
await figma.loadFontAsync({ family: 'Inter', style: 'Bold' });
const title = figma.createText();
title.fontName = { family: 'Inter', style: 'Bold' };
title.fontSize = 32;
title.characters = 'Button';
docFrame.appendChild(title);

// Description text node
await figma.loadFontAsync({ family: 'Inter', style: 'Regular' });
const desc = figma.createText();
desc.fontName = { family: 'Inter', style: 'Regular' };
desc.fontSize = 14;
desc.characters = 'Buttons allow users to take actions and make choices with a single tap.';
docFrame.appendChild(desc);

// Tag docFrame with sharedPluginData for idempotency
docFrame.setSharedPluginData('dsb', 'run_id', RUN_ID);
docFrame.setSharedPluginData('dsb', 'key', 'doc/button');

return { docFrameId: docFrame.id, pageId: page.id };
```

---

## 3. Base Component：Auto-Layout、Child Node、Variable Binding

Base component 是克隆所有 variant 的模板。它必须具备：
1. Auto-layout（不是 manual positioning）
2. 所有 child node 都存在
3. 所有 visual property 都绑定到 variable（没有 hardcoded value）

### 完整 Button Base Component 示例

```javascript
const RUN_ID = 'ds-build-2024-001'; // replace with your actual run ID
await figma.setCurrentPageAsync(
  figma.root.children.find(p => p.name === 'Button')
);

// Rehydrate variables from IDs stored in state ledger
const bgVar     = await figma.variables.getVariableByIdAsync('VAR_ID_color_bg_primary');
const textVar   = await figma.variables.getVariableByIdAsync('VAR_ID_color_text_on_primary');
const paddingVar = await figma.variables.getVariableByIdAsync('VAR_ID_spacing_md');
const radiusVar = await figma.variables.getVariableByIdAsync('VAR_ID_radius_md');
const gapVar    = await figma.variables.getVariableByIdAsync('VAR_ID_spacing_sm');

// --- Base component frame ---
const comp = figma.createComponent();
comp.name = 'Size=Medium, Style=Primary, State=Default';
comp.layoutMode = 'HORIZONTAL';
comp.primaryAxisSizingMode = 'AUTO';
comp.counterAxisSizingMode = 'AUTO';
comp.counterAxisAlignItems = 'CENTER';
comp.primaryAxisAlignItems = 'CENTER';

// Padding — bound to spacing variables
comp.setBoundVariable('paddingTop',    paddingVar);
comp.setBoundVariable('paddingBottom', paddingVar);
comp.setBoundVariable('paddingLeft',   paddingVar);
comp.setBoundVariable('paddingRight',  paddingVar);
comp.setBoundVariable('itemSpacing',   gapVar);

// Corner radius — bound to radius variable
comp.setBoundVariable('topLeftRadius',     radiusVar);
comp.setBoundVariable('topRightRadius',    radiusVar);
comp.setBoundVariable('bottomLeftRadius',  radiusVar);
comp.setBoundVariable('bottomRightRadius', radiusVar);

// Background fill — bound to color variable
const bgPaint = figma.variables.setBoundVariableForPaint(
  { type: 'SOLID', color: { r: 0, g: 0, b: 0 } },
  'color',
  bgVar
);
comp.fills = [bgPaint];

// --- Label text node ---
await figma.loadFontAsync({ family: 'Inter', style: 'Medium' });
const label = figma.createText();
label.name = 'label';
label.fontName = { family: 'Inter', style: 'Medium' };
label.fontSize = 14;
label.characters = 'Button';
label.layoutSizingHorizontal = 'HUG';
label.layoutSizingVertical = 'HUG';

// Text fill — bound to color variable
const textPaint = figma.variables.setBoundVariableForPaint(
  { type: 'SOLID', color: { r: 1, g: 1, b: 1 } },
  'color',
  textVar
);
label.fills = [textPaint];
comp.appendChild(label);

// --- Icon placeholder (Rectangle for now — will be INSTANCE_SWAP) ---
const iconBox = figma.createFrame();
iconBox.name = 'icon';
iconBox.resize(16, 16);
iconBox.fills = [];
iconBox.layoutSizingHorizontal = 'FIXED';
iconBox.layoutSizingVertical = 'FIXED';
comp.appendChild(iconBox);

// Tag for idempotency
comp.setSharedPluginData('dsb', 'run_id', RUN_ID);
comp.setSharedPluginData('dsb', 'phase', 'phase3');
comp.setSharedPluginData('dsb', 'key', 'component/button/base');

return { baseCompId: comp.id };
```

**以下所有内容都必须 variable-bound（绝不 hardcoded）：**

| 属性 | Variable 类型 | API 方法 |
|---|---|---|
| 填充色 | COLOR | `setBoundVariableForPaint(..., 'color', var)` |
| 描边色 | COLOR | `setBoundVariableForPaint(..., 'color', var)` |
| 文本填充色 | COLOR | `setBoundVariableForPaint(..., 'color', var)` |
| Padding（全部 4 边） | FLOAT | `comp.setBoundVariable('paddingTop', var)` |
| Gap / itemSpacing | FLOAT | `comp.setBoundVariable('itemSpacing', var)` |
| Corner radius（全部 4 角） | FLOAT | `comp.setBoundVariable('topLeftRadius', var)` 等 |
| Stroke weight | FLOAT | `comp.setBoundVariable('strokeWeight', var)` |

---

## 4. Variant 矩阵

### 定义 Axes

对每个 component，在编写任何代码前识别其 variant axes。标准 axes：

```
Button:
  Size   → [Small, Medium, Large]
  Style  → [Primary, Secondary, Outline, Ghost]
  State  → [Default, Hover, Focused, Pressed, Disabled]
  Total  = 3 × 4 × 5 = 60 combinations — exceeds 30 limit → split by Style
```

### 30 组合上限和拆分策略

当所有 variant axes 的乘积超过 30 个组合时，拆分 matrix。选项：

1. **按 primary axis 拆分**：创建独立 component set，每个 Style 一个（Primary Button、Secondary Button 等）
2. **使用 INSTANCE_SWAP**：从 variant matrix 中完全移除视觉 axis（例如 Icon），改为作为 INSTANCE_SWAP property 暴露
3. **使用 Building Blocks**：将有自身 state axis 的 sub-element 抽取到 Building Block component set 中

对于 Size x State = 15 个组合的 Button，只有当 Style 不超过 2 个选项时，才将 Style 加为 variant axis（15 x 2 = 30）。更多 Style 时应拆分。

### 使用 use_figma 创建所有 Variant

通过 clone base component 并调整每个 variant 不同的 variable binding 来构建各个 variant。从前一次调用的 state 中传入 base component ID。

```javascript
const RUN_ID = 'ds-build-2024-001';
const BASE_COMP_ID = 'BASE_ID_FROM_STATE'; // from state ledger

await figma.setCurrentPageAsync(
  figma.root.children.find(p => p.name === 'Button')
);

const base = await figma.getNodeByIdAsync(BASE_COMP_ID);

// Variable IDs from state ledger
const vars = {
  // Primary style
  bg_primary:    await figma.variables.getVariableByIdAsync('VAR_ID_color_bg_primary'),
  text_primary:  await figma.variables.getVariableByIdAsync('VAR_ID_color_text_on_primary'),
  // Secondary style
  bg_secondary:  await figma.variables.getVariableByIdAsync('VAR_ID_color_bg_secondary'),
  text_secondary: await figma.variables.getVariableByIdAsync('VAR_ID_color_text_secondary'),
  // Disabled
  bg_disabled:   await figma.variables.getVariableByIdAsync('VAR_ID_color_bg_disabled'),
  text_disabled: await figma.variables.getVariableByIdAsync('VAR_ID_color_text_disabled'),
  // Sizes
  padding_sm: await figma.variables.getVariableByIdAsync('VAR_ID_spacing_sm'),
  padding_md: await figma.variables.getVariableByIdAsync('VAR_ID_spacing_md'),
  padding_lg: await figma.variables.getVariableByIdAsync('VAR_ID_spacing_lg'),
};

const axes = {
  Size:  ['Small', 'Medium', 'Large'],
  Style: ['Primary', 'Secondary'],
  State: ['Default', 'Hover', 'Disabled'],
};

const paddingBySize = { Small: vars.padding_sm, Medium: vars.padding_md, Large: vars.padding_lg };

const components = [];

for (const size of axes.Size) {
  for (const style of axes.Style) {
    for (const state of axes.State) {
      const clone = base.clone();
      clone.name = `Size=${size}, Style=${style}, State=${state}`;

      // Bind padding by size
      clone.setBoundVariable('paddingTop',    paddingBySize[size]);
      clone.setBoundVariable('paddingBottom', paddingBySize[size]);
      clone.setBoundVariable('paddingLeft',   paddingBySize[size]);
      clone.setBoundVariable('paddingRight',  paddingBySize[size]);

      // Bind fill by style + state
      const isDisabled = state === 'Disabled';
      const bgVar  = isDisabled ? vars.bg_disabled  : (style === 'Primary' ? vars.bg_primary  : vars.bg_secondary);
      const txtVar = isDisabled ? vars.text_disabled : (style === 'Primary' ? vars.text_primary : vars.text_secondary);

      const bgPaint = figma.variables.setBoundVariableForPaint(
        { type: 'SOLID', color: { r: 0, g: 0, b: 0 } }, 'color', bgVar
      );
      clone.fills = [bgPaint];

      const labelNode = clone.findOne(n => n.name === 'label');
      const textPaint = figma.variables.setBoundVariableForPaint(
        { type: 'SOLID', color: { r: 1, g: 1, b: 1 } }, 'color', txtVar
      );
      labelNode.fills = [textPaint];

      clone.setSharedPluginData('dsb', 'run_id', RUN_ID);
      clone.setSharedPluginData('dsb', 'key', `component/button/variant/${size}/${style}/${state}`);

      components.push(clone);
    }
  }
}

return { variantIds: components.map(c => c.id) };
```

---

## 5. `combineAsVariants` + Grid 布局

所有 variant component 都存在后，将它们合并为 ComponentSet，并按 grid 放置。此步骤必须是单独的 `use_figma` 调用，你必须从前一次调用的 return value 传入所有 variant ID。

### Grid 设计约定

专业 design system 会用可读 grid 排列 variant，其中：
- **列** = 用户最常交互的 property（通常是 **State**：Default、Hover、Focused、Pressed、Disabled）
- **行** = 组合在一起的结构性 axes（通常是 **Size x Style**，其中 Size 变化最快）
- **间距** = variant 之间 16-40px（20px 是安全默认值；如已有文件则匹配现有文件）
- **内边距** = ComponentSet frame 内 grid 周围 40px

```
Visual structure:
                    Default    Hover     Focused   Pressed   Disabled
  ┌──────────────────────────────────────────────────────────────────┐
  │  Small/Primary   [comp]    [comp]    [comp]    [comp]    [comp] │
  │  Small/Secondary [comp]    [comp]    [comp]    [comp]    [comp] │
  │  Medium/Primary  [comp]    [comp]    [comp]    [comp]    [comp] │
  │  Medium/Secondary[comp]    [comp]    [comp]    [comp]    [comp] │
  │  Large/Primary   [comp]    [comp]    [comp]    [comp]    [comp] │
  │  Large/Secondary [comp]    [comp]    [comp]    [comp]    [comp] │
  └──────────────────────────────────────────────────────────────────┘
```

**为什么 State 放在列上？** State 是设计师横向扫描以验证交互一致性的 axis。Size/Style 定义每行的“身份”。这与 professional design system（M3、Polaris、Simple DS）组织 grid 的方式一致。

### 添加行/列 Header Label

布局 grid 后，在 ComponentSet 外部添加 text label，帮助导航。它们是 page 上 ComponentSet 的 sibling，不是它的 child：

```javascript
// Add column headers above the component set
const colLabels = ['Default', 'Hover', 'Focused', 'Pressed', 'Disabled'];
await figma.loadFontAsync({ family: 'Inter', style: 'Medium' });
for (let i = 0; i < colLabels.length; i++) {
  const label = figma.createText();
  label.fontName = { family: 'Inter', style: 'Medium' };
  label.characters = colLabels[i];
  label.fontSize = 11;
  label.fills = [{ type: 'SOLID', color: { r: 0.5, g: 0.5, b: 0.5 } }];
  label.x = cs.x + padding + i * (childWidth + gap);
  label.y = cs.y - 20;
}

// Add row headers to the left of the component set
const rowLabels = ['Small / Primary', 'Small / Secondary', 'Med / Primary', ...];
for (let i = 0; i < rowLabels.length; i++) {
  const label = figma.createText();
  label.fontName = { family: 'Inter', style: 'Medium' };
  label.characters = rowLabels[i];
  label.fontSize = 11;
  label.fills = [{ type: 'SOLID', color: { r: 0.5, g: 0.5, b: 0.5 } }];
  label.x = cs.x - 120;
  label.y = cs.y + padding + i * (childHeight + gap) + childHeight / 2 - 6;
}
```

**注意：** 这些 label 是文档辅助，不是 component 本身的一部分。它们帮助设计师浏览 variant grid。

### Grid 布局代码

```javascript
const VARIANT_IDS = ['ID1', 'ID2', '...']; // from state ledger
const PAGE_ID = 'PAGE_ID'; // from state ledger

await figma.setCurrentPageAsync(await figma.getNodeByIdAsync(PAGE_ID));

// Collect component nodes
const components = await Promise.all(
  VARIANT_IDS.map(id => figma.getNodeByIdAsync(id))
);

// Combine as variants
const cs = figma.combineAsVariants(components, figma.currentPage);
cs.name = 'Button';

// Grid layout: position each variant based on its property values
// Determine column axis (State) and row axes (Size × Style)
const axes = {
  Size:  ['Small', 'Medium', 'Large'],
  Style: ['Primary', 'Secondary'],
  State: ['Default', 'Hover', 'Disabled'],
};
const COL_AXIS = 'State';  // columns
const ROW_AXES = ['Size', 'Style']; // rows (Size changes fastest)

const gap = 16;
const padding = 40;

// Measure child dimensions (all should be same height within Size tier)
// Use the first child as reference for column width
const childWidth  = 120; // approximate; refine after first screenshot
const childHeight = 40;

cs.children.forEach(child => {
  const props = {};
  child.name.split(', ').forEach(part => {
    const [k, v] = part.split('=');
    props[k] = v;
  });

  const colIdx = axes[COL_AXIS].indexOf(props[COL_AXIS]);
  // Row = Size index * number of styles + Style index
  const rowIdx = axes.Size.indexOf(props.Size) * axes.Style.length
               + axes.Style.indexOf(props.Style);

  child.x = padding + colIdx * (childWidth  + gap);
  child.y = padding + rowIdx * (childHeight + gap);
});

// Resize component set to fit all children + padding
let maxX = 0, maxY = 0;
for (const child of cs.children) {
  maxX = Math.max(maxX, child.x + child.width);
  maxY = Math.max(maxY, child.y + child.height);
}
cs.resizeWithoutConstraints(maxX + padding, maxY + padding);

// Style the component set frame
cs.fills = [{ type: 'SOLID', color: { r: 0.95, g: 0.95, b: 0.98 } }];
cs.cornerRadius = 8;

// Position component set on page (to the right of doc frame)
cs.x = 680;
cs.y = 40;

cs.setSharedPluginData('dsb', 'run_id', 'ds-build-2024-001');
cs.setSharedPluginData('dsb', 'key', 'componentset/button');

return { componentSetId: cs.id };
```

**combineAsVariants 的关键规则：**
- `components` 必须是非空 array，并且只包含 `ComponentNode` object（不是 frame，也不是 group）
- 合并后 child 会放在 (0,0) 并互相重叠，你必须手动定位它们
- 定位后必须调用 `resizeWithoutConstraints`，让 component set frame 适配其内容
- 不存在 `figma.createComponentSet()`，你不能创建空的 component set

---

## 6. Component Property

将 TEXT、BOOLEAN 和 INSTANCE_SWAP property 添加到 ComponentSet（不是添加到单个 variant）。`addComponentProperty` 的返回值是真正的 property key（会追加 `#id:id` 后缀），保存此 key，并在设置 `componentPropertyReferences` 时立即使用它。

### TEXT Property

在 instance 中暴露可编辑文本：

```javascript
// On the ComponentSetNode (cs):
const labelKey = cs.addComponentProperty('Label', 'TEXT', 'Button');
// labelKey is now something like "Label#0:1"

// Wire to the label child in each variant:
for (const child of cs.children) {
  const labelNode = child.findOne(n => n.name === 'label');
  if (labelNode) {
    labelNode.componentPropertyReferences = { characters: labelKey };
  }
}
```

### BOOLEAN Property

切换 child node 可见性：

```javascript
const showIconKey = cs.addComponentProperty('Show Icon', 'BOOLEAN', true);

for (const child of cs.children) {
  const iconNode = child.findOne(n => n.name === 'icon');
  if (iconNode) {
    iconNode.componentPropertyReferences = { visible: showIconKey };
  }
}
```

### INSTANCE_SWAP Property

允许替换 nested component instance（例如替换 icon）：

```javascript
// defaultIconCompId is the ID of the default icon component (from state ledger)
const iconKey = cs.addComponentProperty('Icon', 'INSTANCE_SWAP', DEFAULT_ICON_COMP_ID);

for (const child of cs.children) {
  const iconSlot = child.findOne(n => n.name === 'icon');
  if (iconSlot && iconSlot.type === 'INSTANCE') {
    iconSlot.componentPropertyReferences = { mainComponent: iconKey };
  }
}
```

**使用 INSTANCE_SWAP，而不是为每个 icon 创建一个 variant。** 永远不要把 "Icon=ChevronRight, Icon=ChevronLeft, ..." 添加为 VARIANT axes，这会导致组合爆炸。一个 INSTANCE_SWAP property 可覆盖所有 icon。

### 为 INSTANCE_SWAP 创建 Icon Component

INSTANCE_SWAP 需要一个真实 Component ID 作为默认值。接线 INSTANCE_SWAP 前，你至少需要一个 icon component。以下是从 SVG 创建 icon 的方式：

```javascript
// Create a simple icon component from SVG
const svgNode = figma.createNodeFromSvg(
  '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">' +
  '<path d="M9 18l6-6-6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>' +
  '</svg>'
);

// Wrap in a component
const iconComp = figma.createComponent();
iconComp.name = 'Icon/ChevronRight';
iconComp.resize(24, 24);
iconComp.clipsContent = true;

// Move SVG children into the component
for (const child of [...svgNode.children]) {
  iconComp.appendChild(child);
}
svgNode.remove();

// Bind the icon fill to a color variable (so it respects themes)
// Find vector children and bind their fills
iconComp.findAll(n => n.type === 'VECTOR').forEach(vec => {
  // For stroke-based icons:
  if (vec.strokes.length > 0) {
    const strokePaint = figma.variables.setBoundVariableForPaint(
      { type: 'SOLID', color: { r: 0, g: 0, b: 0 } }, 'color', iconColorVar
    );
    vec.strokes = [strokePaint];
  }
});

iconComp.setSharedPluginData('dsb', 'run_id', RUN_ID);
iconComp.setSharedPluginData('dsb', 'key', 'icon/chevron-right');

return { iconCompId: iconComp.id };
```

**然后将返回的 `iconCompId` 用作 INSTANCE_SWAP 的默认值：**
```javascript
const iconKey = cs.addComponentProperty('Icon', 'INSTANCE_SWAP', ICON_COMP_ID);
```

**用 `preferredValues` 限制 swap 选项：**
添加 INSTANCE_SWAP property 后，可以选择限制哪些 component 出现在 swap picker 中：
```javascript
// Get the property definitions to find the exact key
const props = cs.componentPropertyDefinitions;
const iconPropKey = Object.keys(props).find(k => k.startsWith('Icon'));

// Set preferred values (array of component keys or instance IDs)
cs.editComponentProperty(iconPropKey, {
  preferredValues: [
    { type: 'COMPONENT', key: chevronRightComp.key },
    { type: 'COMPONENT', key: chevronLeftComp.key },
    { type: 'COMPONENT', key: closeComp.key },
  ],
});
```

**Icon library 提示：** 构建任何 UI component 前，先在专用 `Icons` page 上创建所有 icon component。然后在接线 INSTANCE_SWAP property 时引用它们的 ID。

### `componentPropertyReferences` 映射

`componentPropertyReferences` object 会将 node 自身的 property 映射到 component property key：

| Node property | Component property 类型 | 用途 |
|---|---|---|
| `characters` | TEXT | 可编辑 text content |
| `visible` | BOOLEAN | show/hide toggle |
| `mainComponent` | INSTANCE_SWAP | 替换 nested instance |

---

## 7. 用于幂等性的 `sharedPluginData` 标记

每个创建的 node 都要在创建后立即打 tag。这会支持安全清理、可恢复执行和幂等性检查。

```javascript
// After creating any node:
node.setSharedPluginData('dsb', 'run_id', RUN_ID);   // identifies the build run
node.setSharedPluginData('dsb', 'phase', 'phase3');  // which phase created it
node.setSharedPluginData('dsb', 'key', KEY);         // unique logical key for this entity

// Reading back:
const runId = node.getSharedPluginData('dsb', 'run_id'); // '' if not set
const key   = node.getSharedPluginData('dsb', 'key');
```

**Key 命名约定：** 使用以 `/` 分隔的 logical path，镜像 entity hierarchy：
```
'component/button/base'
'component/button/variant/Medium/Primary/Default'
'componentset/button'
'doc/button'
'page/button'
```

**创建前的幂等性检查：** 创建 node 前，扫描当前 page 是否已有同 `key` 的 node：

```javascript
const existing = figma.currentPage.findAll(n =>
  n.getSharedPluginData('dsb', 'key') === 'componentset/button'
);
if (existing.length > 0) {
  // Skip creation — already done. Return existing node's ID.
  return { componentSetId: existing[0].id };
}
```

---

## 8. 文档

### 页面标题 + 描述 frame

documentation frame（见第 2 节）应包含：
1. 以大标题形式展示 component 名称（32px+ Bold）
2. 用 1-3 句话说明该 component 是什么、何时使用
3. 规格说明（尺寸、spacing 值、accessibility 注意事项）

### Component `description` 属性

在 ComponentSet 上设置 description。它会显示在 Figma properties panel 中，并作为文档导出：

```javascript
cs.description = 'Buttons allow users to take actions and make choices. Use Primary for the highest-emphasis action on a page.';
```

### `documentationLinks`

链接到外部文档（Storybook、design spec、tokens reference）：

```javascript
cs.documentationLinks = [
  { uri: 'https://your-storybook.com/button' }
];
```

### Node 命名和组织

- ComponentSet：使用普通 component 名称，例如 `'Button'`
- 单个 variant：使用 `'Property=Value, Property=Value'` 格式（匹配文件中已有的大小写）
- Child node：使用语义化名称，例如 `'label'`、`'icon'`、`'container'`、`'state-layer'`
- Documentation frame：`'ComponentName / Documentation'`

---

## 9. 验证

创建或修改 component 后，必须先完成验证，再继续处理下一个 component。

### `get_metadata` 结构检查

创建 component set 后，对 ComponentSet node 调用 `get_metadata` 并确认：
- `variantGroupProperties` 列出了预期 axes，且 value array 正确
- `componentPropertyDefinitions` 包含预期的 TEXT/BOOLEAN/INSTANCE_SWAP property
- `children.length` 等于预期 variant 数量（例如 3x2x3 时为 18）
- 没有 child 命名为 `'Component 1'`（未命名 component 通常意味着存在 bug）

### `get_screenshot` - 视觉验证（关键）

`get_screenshot` 会返回指定 node 的**图像**。请对 **component page node**（不是 component set）调用它，以便查看包含 documentation 和 grid label 的完整页面。

```
Tool: get_screenshot
Args: { nodeId: "PAGE_NODE_ID", fileKey: "FILE_KEY" }
```

**如何使用 screenshot：**

1. **展示给用户**：这是主要目的。把 screenshot 作为用户 checkpoint 的一部分展示：“这是 Button component。看起来正确吗？”
2. **自行分析**：如果你具备视觉能力，按下方视觉检查清单核对。如果没有（text-only agent），则仅通过 `get_metadata` 做结构验证，并用文字说明你创建了什么。

**视觉验证清单**（查看 screenshot 时逐项检查）：

| # | 检查项 | 良好表现 | 异常表现 |
|---|-------|----------------------|------------------------|
| 1 | **Grid 布局** | Variant 按整齐行列排列，spacing 一致 | 所有 variant 堆在左上角（0,0 stacking bug） |
| 2 | **颜色填充** | Component 按 style variant 显示清晰且正确的颜色 | 所有 component 都是黑色或同一种颜色（variable binding 失败） |
| 3 | **尺寸区分** | Small variant 明显小于 Large variant | 所有 variant 尺寸相同（height/padding 未绑定到 variable） |
| 4 | **文本可读性** | Label 使用正确字体和颜色，且可见 | Text 不可见（白底白字）、缺失或显示 `"undefined"` |
| 5 | **Spacing/padding** | 内部 padding 可见，component 没有紧贴内容收缩 | Component 看起来拥挤，或没有可见内部空间 |
| 6 | **State 区分** | Hover/Pressed variant 与 Default 有可见色彩差异 | 所有 state 看起来相同（未应用 state-specific fill） |
| 7 | **Disabled state** | 与 active state 相比，opacity 更低或颜色更弱 | Disabled 看起来与 Default 相同 |
| 8 | **Documentation frame** | Title + description text 在 component grid 上方或旁边可见 | 没有 documentation，或与 component set 重叠 |
| 9 | **Grid label** | Row/column header 在 component set 周围可见（如果已添加） | Label 与 grid 重叠或缺失 |
| 10 | **Component set 边界** | 灰色背景 frame 用均匀 padding 包住所有 variant | Frame 过小（variant 被裁切）或过大 |

**Screenshot -> 诊断 -> 修复对照表：**

| Screenshot 显示 | 诊断 | 修复脚本 |
|-----------------|-----------|------------|
| 所有 variant 堆在左上角 | `combineAsVariants` 后未应用 grid layout | 重新运行 grid layout script（第 5 节） |
| 全部为黑色或同一种颜色 | Variable binding 失败，或 variable 在当前 active mode 下没有值 | 重新运行 variable binding，并检查 mode value |
| 没有可见 text | Font 未加载，或 text fill 与背景同色 | 检查是否调用了 `loadFontAsync`；把 text fill 绑定到 `color/text/*` variable |
| 所有 variant 尺寸相同 | Padding/height 未绑定到 size variable | 使用 size-specific token 重新运行 `bindVariablesToComponent` |
| Component set frame 很小 | 未调用 `resizeWithoutConstraints`，或使用了错误尺寸 | 根据 child 重新计算 bounds 并 resize |
| Doc frame 与 component 重叠 | Component set 被放在与 doc frame 相同的 x,y 位置 | 移动 component set：`cs.x = docFrame.x + docFrame.width + 60` |

**无法进行视觉分析时：**
如果你的模型无法处理图像（text-only mode），改用结构验证：
1. 对 component set 调用 `get_metadata`，确认 child count、property definition、variant name
2. 运行一次 `use_figma`，采样关键 property：
```javascript
const cs = await figma.getNodeByIdAsync(CS_ID);
const sample = cs.children.slice(0, 3).map(c => ({
  name: c.name,
  width: c.width, height: c.height,
  x: c.x, y: c.y,
  fills: c.fills?.map(f => f.type === 'SOLID' ?
    { r: f.color.r.toFixed(2), g: f.color.g.toFixed(2), b: f.color.b.toFixed(2), boundVar: f.boundVariables?.color?.id } : f.type
  ),
}));
return { sampleVariants: sample, totalChildren: cs.children.length };
```
这样无需视觉能力，也能获取 position（grid 是否工作）、dimension（size 是否有差异）和 fill info（binding 是否工作）。

**何时截图：**
- 每个完成的 component 之后（强制，属于用户 checkpoint）
- 创建 foundations documentation page 之后
- 最终 QA 之后（每个 page 都截图）
- 不要在每个中间步骤后截图（浪费 tool call）

### 常见问题

| 现象 | 可能原因 | 修复 |
|---|---|---|
| 所有 variant 堆在 (0,0) | 已调用 `combineAsVariants`，但从未重新定位 children | 重新运行 grid layout script |
| Variant 显示错误颜色 | Variable binding 在 `combineAsVariants` 之后才应用，而不是之前 | 在 component set children 上重新绑定 |
| Variant 数量错误 | clone 循环索引错误 | 合并前打印 `components.map(c => c.name)` |
| BOOLEAN property 没有效果 | `componentPropertyReferences` 设置在 component set frame 上，而不是 child node 上 | 找到实际 child node 并在那里设置 references |
| INSTANCE_SWAP 没有 swap option | 默认值不是有效 component ID | 将真实存在的 component ID 作为 `defaultValue` 传入 |
| `combineAsVariants` 抛错 | Array 中至少有一个 node 不是 `ComponentNode` | 过滤 array：`nodes.filter(n => n.type === 'COMPONENT')` |
| `addComponentProperty` 返回意外 key | 这是预期行为，key 会获得 `#id:id` 后缀 | 立即保存返回值：`const key = cs.addComponentProperty(...)` |

---

## 10. 完整工作示例：Button Component

下面展示 Button component 的完整 `use_figma` 调用序列，包括调用之间的 state 传递。请用 state ledger 中的实际值替换 `RUN_ID` 和 variable ID。

### 调用 1：创建 component page

**目标：** 创建（或找到）Button page。
**状态输入：** 无
**状态输出：** `{ pageId }`

```javascript
let page = figma.root.children.find(p => p.name === 'Button');
if (!page) { page = figma.createPage(); page.name = 'Button'; }
page.setSharedPluginData('dsb', 'run_id', 'ds-build-2024-001');
page.setSharedPluginData('dsb', 'key', 'page/button');
return { pageId: page.id };
```

### 调用 2：创建 documentation frame

**目标：** 添加 title + description frame。
**状态输入：** `{ pageId }`
**状态输出：** `{ docFrameId }`

```javascript
const PAGE_ID = 'PAGE_ID_FROM_STATE';
const page = await figma.getNodeByIdAsync(PAGE_ID);
await figma.setCurrentPageAsync(page);

// Idempotency check
const existing = page.findAll(n => n.getSharedPluginData('dsb', 'key') === 'doc/button');
if (existing.length > 0) {
  return { docFrameId: existing[0].id };
}

await figma.loadFontAsync({ family: 'Inter', style: 'Bold' });
await figma.loadFontAsync({ family: 'Inter', style: 'Regular' });

const docFrame = figma.createFrame();
docFrame.name = 'Button / Documentation';
docFrame.x = 40; docFrame.y = 40;
docFrame.layoutMode = 'VERTICAL';
docFrame.primaryAxisSizingMode = 'AUTO';
docFrame.counterAxisSizingMode = 'FIXED';
docFrame.resize(560, 100);
docFrame.paddingTop = 40; docFrame.paddingBottom = 40;
docFrame.paddingLeft = 40; docFrame.paddingRight = 40;
docFrame.itemSpacing = 16;
docFrame.fills = [{ type: 'SOLID', color: { r: 1, g: 1, b: 1 } }];

const title = figma.createText();
title.fontName = { family: 'Inter', style: 'Bold' };
title.fontSize = 32;
title.characters = 'Button';
docFrame.appendChild(title);

const desc = figma.createText();
desc.fontName = { family: 'Inter', style: 'Regular' };
desc.fontSize = 14;
desc.characters = 'Buttons allow users to take actions with a single tap. Use Primary for the highest-emphasis action on a page, Secondary for supporting actions.';
desc.layoutSizingHorizontal = 'FILL';
docFrame.appendChild(desc);

docFrame.setSharedPluginData('dsb', 'run_id', 'ds-build-2024-001');
docFrame.setSharedPluginData('dsb', 'key', 'doc/button');

return { docFrameId: docFrame.id };
```

### 调用 3：创建 base component

**目标：** 创建带 auto-layout 和完整 variable binding 的 base component。
**状态输入：** `{ pageId }` + Phase 1 中的 variable ID
**状态输出：** `{ baseCompId }`

*（完整代码见第 3 节；请替换为 state ledger 中的实际 variable ID。）*

### 调用 4：创建所有 variant

**目标：** clone base，并生成全部 18 个 variant（3 Size x 2 Style x 3 State）。
**状态输入：** `{ pageId, baseCompId }` + variable ID
**状态输出：** `{ variantIds: ['id1', 'id2', ..., 'id18'] }`

```javascript
const RUN_ID = 'ds-build-2024-001';
const BASE_ID = 'BASE_COMP_ID_FROM_STATE';
const PAGE_ID = 'PAGE_ID_FROM_STATE';
// Variable IDs from state ledger:
const VAR = {
  bg_primary:     'VAR_ID_1',
  text_primary:   'VAR_ID_2',
  bg_secondary:   'VAR_ID_3',
  text_secondary: 'VAR_ID_4',
  bg_disabled:    'VAR_ID_5',
  text_disabled:  'VAR_ID_6',
  padding_sm:     'VAR_ID_7',
  padding_md:     'VAR_ID_8',
  padding_lg:     'VAR_ID_9',
};

const page = await figma.getNodeByIdAsync(PAGE_ID);
await figma.setCurrentPageAsync(page);

const base = await figma.getNodeByIdAsync(BASE_ID);

// Load all variables
const vars = {};
for (const [k, v] of Object.entries(VAR)) {
  vars[k] = await figma.variables.getVariableByIdAsync(v);
}

const axes = {
  Size:  ['Small', 'Medium', 'Large'],
  Style: ['Primary', 'Secondary'],
  State: ['Default', 'Hover', 'Disabled'],
};
const paddingMap = { Small: vars.padding_sm, Medium: vars.padding_md, Large: vars.padding_lg };

const components = [];
for (const size of axes.Size) {
  for (const style of axes.Style) {
    for (const state of axes.State) {
      const clone = base.clone();
      clone.name = `Size=${size}, Style=${style}, State=${state}`;

      clone.setBoundVariable('paddingTop',    paddingMap[size]);
      clone.setBoundVariable('paddingBottom', paddingMap[size]);
      clone.setBoundVariable('paddingLeft',   paddingMap[size]);
      clone.setBoundVariable('paddingRight',  paddingMap[size]);

      const isDisabled = state === 'Disabled';
      const bgV  = isDisabled ? vars.bg_disabled  : (style === 'Primary' ? vars.bg_primary  : vars.bg_secondary);
      const txV  = isDisabled ? vars.text_disabled : (style === 'Primary' ? vars.text_primary : vars.text_secondary);

      clone.fills = [figma.variables.setBoundVariableForPaint(
        { type: 'SOLID', color: { r: 0, g: 0, b: 0 } }, 'color', bgV
      )];

      const labelNode = clone.findOne(n => n.name === 'label');
      labelNode.fills = [figma.variables.setBoundVariableForPaint(
        { type: 'SOLID', color: { r: 1, g: 1, b: 1 } }, 'color', txV
      )];

      clone.setSharedPluginData('dsb', 'run_id', RUN_ID);
      clone.setSharedPluginData('dsb', 'key', `component/button/variant/${size}/${style}/${state}`);
      components.push(clone);
    }
  }
}

return { variantIds: components.map(c => c.id) };
```

### 调用 5：combineAsVariants + grid layout

**目标：** 将全部 18 个 variant 合并成 ComponentSet，并按 grid 布局。
**状态输入：** `{ pageId, variantIds }`（18 个 ID）
**状态输出：** `{ componentSetId }`

*（完整代码见第 5 节。）*

### 调用 6：添加 component property

**目标：** 添加 TEXT、BOOLEAN、INSTANCE_SWAP property，并接线到 child node。
**状态输入：** `{ pageId, componentSetId }`
**状态输出：** `{ componentSetId, properties: { labelKey, showIconKey, iconKey } }`

```javascript
const CS_ID = 'CS_ID_FROM_STATE';
const DEFAULT_ICON_ID = 'ICON_COMP_ID_FROM_STATE';
const page = figma.root.children.find(p => p.name === 'Button');
await figma.setCurrentPageAsync(page);

const cs = await figma.getNodeByIdAsync(CS_ID);
cs.description = 'Buttons allow users to take actions and make choices with a single tap.';
cs.documentationLinks = [{ uri: 'https://your-storybook.com/button' }];

// Add properties — save returned keys
const labelKey    = cs.addComponentProperty('Label', 'TEXT', 'Button');
const showIconKey = cs.addComponentProperty('Show Icon', 'BOOLEAN', true);
const iconKey     = cs.addComponentProperty('Icon', 'INSTANCE_SWAP', DEFAULT_ICON_ID);

// Wire to children
for (const child of cs.children) {
  const labelNode = child.findOne(n => n.name === 'label');
  if (labelNode) labelNode.componentPropertyReferences = { characters: labelKey };

  const iconNode = child.findOne(n => n.name === 'icon');
  if (iconNode) {
    iconNode.componentPropertyReferences = {
      visible: showIconKey,
      ...(iconNode.type === 'INSTANCE' ? { mainComponent: iconKey } : {}),
    };
  }
}

return {
  componentSetId: cs.id,
  properties: { labelKey, showIconKey, iconKey },
};
```

### 调用 7：使用 get_metadata 验证

**目标：** 结构检查，包括 variant count、property、axes。
**动作：** 对 ComponentSet node ID（来自 state）调用 `get_metadata`。在结果中确认：
- `children.length === 18`
- `variantGroupProperties` 包含 `Size`、`Style`、`State` key，且 value array 正确
- `componentPropertyDefinitions` 包含 `Label`、`Show Icon`、`Icon` entry

### 调用 8：使用 get_screenshot 验证

**目标：** 视觉检查，包括 layout、color、text。
**动作：** 对 Button page 调用 `get_screenshot`。检查 screenshot。如果 variant 堆叠，重新运行调用 5；如果颜色错误，检查 variable binding。

### 检查点

调用 8 之后：向用户展示 screenshot，并询问：“这是包含 18 个 variant 的 Button component。看起来正确吗？”在用户确认前，不要继续处理下一个 component。
