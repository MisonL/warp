> 属于 [figma-generate-library skill](../SKILL.md) 的一部分。

# Discovery Phase 参考

本文档覆盖 design system 构建 Phase 0 所需的全部内容：分析 codebase 中的 token、检查 Figma file 中的既有约定、搜索 subscribed library、制定计划，并在开始任何写入操作前解决冲突。

---

## 1. Codebase 分析：查找 Token 来源

### 搜索优先级

按以下顺序查找 token 来源。一旦找到权威来源就停止；多种格式可以共存：

1. Design token file：`*.tokens.json`、`tokens/*.json`、`src/tokens/**`
2. CSS variable file：`variables.css`、`tokens.css`、`theme.css`、`global.css`
3. Tailwind config：`tailwind.config.js`、`tailwind.config.ts`
4. CSS-in-JS theme object：`theme.ts`、`createTheme`、`ThemeProvider`
5. 平台特定来源：iOS Asset catalog（`.xcassets`）、Android `themes.xml`、`colors.xml`

### CSS Custom Properties（Web 中最常见）

**要搜索的内容：**

```
:root { ... }
@theme { ... }          ← Tailwind v4
--color-*, --spacing-*, --radius-*, --shadow-*, --font-*
```

**模式：** `/--[\w-]+:\s*[^;]+/g`

**常见文件位置：** `src/styles/tokens.css`、`src/styles/variables.css`、`src/theme/*.css`

**抽取与命名转换：**

| CSS Property | Figma Variable Name | Figma Type | WEB Code Syntax |
|---|---|---|---|
| `--color-bg-primary: #fff` | `color/bg/primary` | COLOR | `var(--color-bg-primary)` |
| `--color-text-secondary: #757575` | `color/text/secondary` | COLOR | `var(--color-text-secondary)` |
| `--spacing-sm: 8px` | `spacing/sm` | FLOAT | `var(--spacing-sm)` |
| `--radius-md: 8px` | `radius/md` | FLOAT | `var(--radius-md)` |
| `--font-body: "Inter"` | `typography/body/font-family` | STRING | `var(--font-body)` |

**命名规则：** 在 category 边界处把连字符替换为斜杠。最终 path segment 内的连字符保留：`--color-bg-primary` -> `color/bg/primary`，但 `--color-bg-primary-hover` -> `color/bg/primary-hover`。

**始终保存原始 CSS variable name** 作为 code syntax value。不要从 Figma variable name 推导它。如果 codebase 使用 `--sds-color-background-brand-default`，就在 `setVariableCodeSyntax('WEB', '--sds-color-background-brand-default')` 中精确使用该字符串。

### Tailwind 配置

**在 `tailwind.config.js` 或 `tailwind.config.ts` 中查找的内容：**

```javascript
// theme.extend.colors → Figma color variables
{ primary: { DEFAULT: '#3366FF', light: '#6699FF', dark: '#0033CC' } }
// → color/primary/default, color/primary/light, color/primary/dark

// theme.extend.spacing → Figma FLOAT variables
{ 'xs': '4px', 'sm': '8px', 'md': '16px' }
// → spacing/xs = 4, spacing/sm = 8, spacing/md = 16

// theme.extend.borderRadius → Figma FLOAT variables
{ 'sm': '4px', 'md': '8px', 'lg': '16px' }
// → radius/sm = 4, radius/md = 8, radius/lg = 16
```

Tailwind utility class name（`bg-blue-500`、`p-4`）不是 token。应从 config object 中抽取 value，而不是从 class name 中抽取。

### Design Token Community Group（DTCG）格式

**模式：** `*.tokens.json` 或 `tokens/*.json`。查找 source file，不要使用 Style Dictionary 或 Tokens Studio 的生成产物。

```json
{
  "color": {
    "bg": {
      "primary": { "$type": "color", "$value": "#ffffff" },
      "secondary": { "$type": "color", "$value": "#f5f5f5" }
    }
  },
  "spacing": {
    "sm": { "$type": "dimension", "$value": "8px" }
  }
}
```

Nested key 映射为 slash-separated Figma name：`color.bg.primary` -> `color/bg/primary`。

### CSS-in-JS / Theme Object

**要搜索的内容：** `createTheme`、`ThemeProvider`、`theme = {}`、styled-components、Emotion、Stitches、vanilla-extract

```typescript
// theme.colors.bg.primary → Figma variable: color/bg/primary
// theme.spacing.sm        → Figma variable: spacing/sm
// Multiple theme objects (lightTheme, darkTheme) → modes in the same collection
```

### iOS Token 来源

```swift
// Asset catalog colors in .xcassets/Colors.xcassets
// extension Color { static let bgPrimary = Color("bg-primary") }
// Look for traitCollection.userInterfaceStyle for dark mode detection
```

### Android Token 来源

```kotlin
// res/values/colors.xml  <color name="primary">#3366FF</color>
// res/values-night/colors.xml  (dark mode overrides)
// MaterialTheme.colorScheme.primary in Compose
// val Primary = Color(0xFF3366FF)
```

### 检测 Dark Mode

| Platform | 信号 |
|---|---|
| Web (CSS) | `@media (prefers-color-scheme: dark)`, `.dark { }`, `[data-theme="dark"]` |
| Web (Tailwind) | config 中的 `darkMode: 'class'` 或 `darkMode: 'media'` |
| Web (JS) | 与 `lightTheme` 并存的独立 `darkTheme` object |
| iOS | 带 `traitCollection.userInterfaceStyle` 的 `Color(uiColor:)`，或 dual-appearance asset catalog |
| Android | 包含 `Theme.*.Night` 的 `themes.xml`、Compose 中的 `isSystemInDarkTheme()`、`values-night/` folder |

**Figma 映射：** 如果存在 dark mode，则 semantic color collection 至少有 2 个 mode（Light/Dark）。Primitive collection 保持 single-mode。

### Shadow/Elevation 抽取

Shadow 不能成为 Figma variable。它们会成为 **Effect Style**。

```css
/* Look for: box-shadow, --shadow-* */
--shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
--shadow-md: 0 4px 6px -1px rgba(0,0,0,0.10);
--shadow-lg: 0 10px 15px -3px rgba(0,0,0,0.10);
```

CSS `0 4px 6px -1px rgba(0,0,0,0.1)` -> Figma：
```
{ type: "DROP_SHADOW", offset: {x:0, y:4}, radius: 6, spread: -1, color: {r:0, g:0, b:0, a:0.1} }
```

### Typography 抽取

| Code token | 映射到 |
|---|---|
| `font-size: 16px` | FLOAT variable（scope `FONT_SIZE`）或 Text Style `fontSize` |
| `line-height: 1.5` | Text Style `lineHeight: {value: 24, unit: "PIXELS"}` |
| `font-weight: 600` | Text Style `fontName: {family: "Inter", style: "Semi Bold"}` |
| `letter-spacing: -0.02em` | Text Style `letterSpacing: {value: -2, unit: "PERCENT"}` |
| `font-family: "Inter"` | STRING variable（scope `FONT_FAMILY`）或 Text Style `fontName.family` |

Composite text style（所有 property 打包）映射为 Figma Text Style。单独 property 映射为带适当 scope 的 Figma variable。

### Component 抽取

对每个 component，抽取：

1. **Name** -> Figma component set name
2. **Union-type props** -> VARIANT property
3. **String content props** -> TEXT property
4. **Boolean props** -> BOOLEAN property（与 interaction state 组合时，也对应 VARIANT State）
5. **Child/slot props** -> INSTANCE_SWAP property

```typescript
// React example:
interface ButtonProps {
  size: 'sm' | 'md' | 'lg';          // → VARIANT: Size = sm|md|lg
  variant: 'primary' | 'secondary';   // → VARIANT: Style = primary|secondary
  disabled?: boolean;                  // → VARIANT: State (combine: default|hover|pressed|disabled)
  label: string;                       // → TEXT: Label
  icon?: ReactNode;                    // → INSTANCE_SWAP: Icon + BOOLEAN: Show Icon
}
// → Component Set "Button", variant count: 3 sizes × 2 styles × 4 states = 24
```

---

## 2. Figma File 检查

每次 build 开始时运行这些 `use_figma` 片段。它们全部是 read-only，可在任何用户 checkpoint 前安全运行。

### 列出所有 Page

```javascript
const pages = figma.root.children.map((p, i) => ({
  index: i,
  name: p.name,
  id: p.id,
  childCount: p.children.length
}));
return { pages };
```

解读：记录 page name 的命名约定（是 PascalCase 还是 sentence case？），统计 separator page（`---`），识别现有 component page 与 foundation page。

### 列出带 Mode 的 Variable Collection

```javascript
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const result = collections.map(c => ({
  id: c.id,
  name: c.name,
  modes: c.modes,                    // [{modeId, name}, ...]
  variableCount: c.variableIds.length,
  defaultModeId: c.defaultModeId
}));
return { collections: result };
```

解读：识别现有 primitive/semantic 分层，记录 mode name（是使用 "Light/Dark" 还是 "SDS Light/SDS Dark"？），统计 variable 以理解范围。

### 列出 Collection 中的 Variable（包含 name、type、scope 和 sample value）

```javascript
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const targetName = "Color"; // change to the collection you want to inspect
const coll = collections.find(c => c.name === targetName);
if (!coll) { return { error: `Collection "${targetName}" not found` }; }

const allVars = await figma.variables.getLocalVariablesAsync();
const vars = allVars.filter(v => v.variableCollectionId === coll.id);

const result = vars.map(v => ({
  id: v.id,
  name: v.name,
  resolvedType: v.resolvedType,
  scopes: v.scopes,
  codeSyntax: v.codeSyntax,
  // First mode value only, for a sample
  sampleValue: v.valuesByMode[coll.defaultModeId]
}));

return { collection: coll.name, variableCount: result.length, variables: result };
```

解读：检查 variable 是否使用 `ALL_SCOPES`（不佳），检查命名约定（是否为 slash-separated hierarchy），检查是否设置 code syntax，并识别 alias chain。

### 列出带 Property 的 Component Set

```javascript
await figma.setCurrentPageAsync(figma.currentPage); // ensures page context
const componentSets = figma.currentPage.findAll(n => n.type === 'COMPONENT_SET');
const result = componentSets.map(cs => ({
  id: cs.id,
  name: cs.name,
  variantCount: cs.children.length,
  properties: Object.entries(cs.componentPropertyDefinitions).map(([key, def]) => ({
    name: key,
    type: def.type,
    variantOptions: def.variantOptions || null,
    defaultValue: def.defaultValue
  }))
}));
return { componentSets: result, count: result.length };
```

注意：要搜索所有 page，请遍历 `figma.root.children`，并对每个 page 调用 `setCurrentPageAsync`。

### 列出所有 Style

```javascript
const [textStyles, effectStyles, paintStyles] = await Promise.all([
  figma.getLocalTextStylesAsync(),
  figma.getLocalEffectStylesAsync(),
  figma.getLocalPaintStylesAsync()
]);

return {
  textStyles: textStyles.map(s => ({ id: s.id, name: s.name, fontSize: s.fontSize, fontName: s.fontName })),
  effectStyles: effectStyles.map(s => ({ id: s.id, name: s.name, effectCount: s.effects.length })),
  paintStyles: paintStyles.map(s => ({ id: s.id, name: s.name })),
  counts: { text: textStyles.length, effect: effectStyles.length, paint: paintStyles.length }
};
```

### 检查现有 Component 的命名约定

```javascript
// Replace with the node ID of an existing component to analyze
const node = await figma.getNodeByIdAsync("YOUR_NODE_ID");
if (!node) { return { error: "Node not found" }; }

// Check fills for variable bindings
const fillInfo = [];
if ('fills' in node && Array.isArray(node.fills)) {
  for (const fill of node.fills) {
    if (fill.type === 'SOLID' && fill.boundVariables?.color) {
      fillInfo.push({ type: 'variable_alias', id: fill.boundVariables.color.id });
    } else if (fill.type === 'SOLID') {
      fillInfo.push({ type: 'hardcoded', r: fill.color.r, g: fill.color.g, b: fill.color.b });
    }
  }
}

return {
  name: node.name,
  type: node.type,
  fills: fillInfo,
  sharedPluginData: node.getSharedPluginData('dsb', 'key') || null
};
```

---

## 3. 使用 search_design_system

### 搜索内容

`search_design_system` 会针对给定文件，在 **subscribed design library** 中并行运行三类搜索：

1. **Components**：已发布的 library component，通过 recommendation engine 按 name/description 搜索（按相关性排序，不是精确匹配）
2. **Variables**：subscribed library 中的 design token（颜色、spacing 等）
3. **Styles**：paint style、text style 和 effect style

只会搜索该文件已订阅的 library。如果结果为空，该文件可能没有订阅任何 design system library。

### 输入

```
search_design_system({
  query: "button",              // required — text query
  fileKey: "abc123",            // required — your file key
  includeComponents: true,      // default true
  includeVariables: true,       // default true
  includeStyles: true           // default true
})
```

### 返回内容

```json
{
  "components": [
    {
      "name": "Button",
      "libraryName": "Design System",
      "assetType": "component_set",
      "componentKey": "abc123def",
      "description": "Primary action button"
    }
  ],
  "variables": [
    {
      "name": "colors/primary/500",
      "variableType": "COLOR",
      "variableSetKey": "set1key",
      "key": "var1key",
      "scopes": ["FRAME_FILL", "SHAPE_FILL"],
      "variableCollectionName": "Colors"
    }
  ],
  "styles": [
    {
      "name": "Heading/H1",
      "styleType": "TEXT",
      "key": "style1key"
    }
  ]
}
```

### 如何解读结果

**Components：** `componentKey` 可在 `use_figma` 中用于导入 component：
```javascript
const component = await figma.importComponentByKeyAsync("abc123def");
// or for component sets:
const componentSet = await figma.importComponentSetByKeyAsync("abc123def");
```

**Variables：** `variableSetKey` 是 collection key。`key` 是 variable key。使用这些信息理解当前命名约定，以及有哪些 token 可作为 alias 来源。

**Styles：** `key` 可配合 `figma.importStyleByKeyAsync(key)` 导入到当前文件。

### 何时搜索

- **Phase 0, step 0c**：在规划任何内容前广泛搜索（`query: "button"`、`query: "color"`、`query: "spacing"`）。这会建立 reuse baseline。
- **每次创建 component 前立即执行**：在编写任何 `use_figma` 创建代码前，搜索具体 component name。

**复用决策：**

| 条件 | 决策 |
|---|---|
| 找到 variant API 匹配且 token model 相同的 component | 导入并复用 |
| 找到 component，但 variant property 错误或存在 hardcoded value | 重建 |
| 找到视觉匹配但 API 不兼容的 component | Wrap：作为 instance 嵌套进新的 wrapper component |

---

## 4. 制定计划

完成 codebase 分析和 Figma 检查后，生成 mapping table 并展示给用户。

### Token -> Variable 映射表

对代码中找到的每个 token，记录：

| Code Token | CSS Name | Raw Value | Figma Collection | Figma Variable Name | Figma Type | Mode(s) |
|---|---|---|---|---|---|---|
| `theme.colors.blue[500]` | `--color-blue-500` | `#3B82F6` | Primitives | `blue/500` | COLOR | Value |
| `theme.colors.bg.primary` | `--color-bg-primary` | (light: blue/50, dark: gray/900) | Color | `color/bg/primary` | COLOR | Light, Dark |
| `theme.spacing.sm` | `--spacing-sm` | `8px` | Spacing | `spacing/sm` | FLOAT | Value |
| `theme.radii.md` | `--radius-md` | `8px` | Spacing | `radius/md` | FLOAT | Value |
| `theme.shadows.md` | `--shadow-md` | `0 4px 6px rgba(0,0,0,0.1)` | N/A | N/A | Effect Style | N/A |

### Component -> Component Set 映射表

| Code Component | Props -> Variant Axes | Variant Count | Figma Page | Reuse? |
|---|---|---|---|---|
| `Button` | size (sm/md/lg) x variant (primary/secondary) x state (default/hover/disabled) | 18 | Buttons | Search first |
| `Avatar` | size (sm/md/lg) x type (image/initials/icon) | 9 | Avatars | Search first |

### Gap 识别

对比代码中发现的内容和 Figma 中已有的内容：

- **New：** 存在于代码中但不存在于 Figma 中的 token 或 component -> 创建
- **Existing：** Figma 中已有且名称匹配的 token 或 component -> 验证 scope/code-syntax，然后跳过或更新
- **Conflict：** 同名但 value 不同 -> 升级给用户决策（见第 5 节）
- **Figma-only：** 存在于 Figma 中但不存在于代码中 -> 标记给用户，通常跳过

### 面向用户的 Checkpoint Message 模板

继续前展示此消息。没有用户明确批准时，绝不开始 Phase 1。

```
Here's what I found and what I plan to build:

CODEBASE ANALYSIS
  Colors: {N} primitives ({families}), {M} semantic tokens ({light/dark if applicable})
  Spacing: {N} tokens ({range})
  Typography: {N} text styles, {M} individual scale tokens
  Shadows: {N} levels → will become Effect Styles
  Components: {list of component names}

EXISTING FIGMA FILE
  Collections: {N} existing collections
  Variables: {M} existing variables
  Styles: {K} text, {L} effect, {J} paint styles
  Components: {list}

PLAN
  New collections: {list with mode counts}
  New variables: ~{N} ({breakdown by collection})
  New styles: {N} text, {M} effect
  New components: {list}
  Libraries to search before each component: {list}

GAPS / CONFLICTS NEEDING DECISIONS
  ⚠ {conflict description} — Code says X, Figma already has Y. Which wins?

WHAT I WON'T BUILD (and why)
  - {item}: already exists in Figma with matching conventions
  - {item}: not supported as a Figma variable (e.g. z-index, animation timing)

Shall I proceed?
```

---

## 5. 冲突解决：Code 与 Figma 不一致时

当同一个 token/component 同时存在于 code 和 Figma 中，但 value、name 或结构不同，**始终询问用户**。不要静默选择其中一个。

### 决策框架

| 场景 | 询问用户 |
|---|---|
| CSS name 相同但 hex value 不同（例如 code 中 `--color-accent` 是 `#3366FF`，Figma 中是 `#5B7FFF`） | "Code 显示 `#3366FF`，但 Figma 当前的 `color/accent/default` 是 `#5B7FFF`。哪个是正确的？" |
| Component name 相同但 variant axes 不同（code 有 `size: sm/md/lg`，Figma 有 `Size: Small/Large`） | "Code 使用 3 个 size（sm/md/lg），但 Figma 有 2 个（Small/Large）。应添加 Medium，还是重命名以匹配 code？" |
| Code 有 semantic token，但没有 primitive layer；Figma 已有完整分层 system | "Codebase 使用扁平的单层 token model。Figma file 使用 primitive/semantic 分层。应匹配 Figma architecture 还是 code architecture？" |
| Figma variable 存在但使用 `ALL_SCOPES`（按最佳实践不正确） | "我发现 `color/bg/primary` 已存在，但它使用 ALL_SCOPES。我建议改为 `FRAME_FILL, SHAPE_FILL`。可以更新 scope 吗？" |
| Code 使用 camelCase（`backgroundColor`），Figma 使用 slash-separated（`color/bg/default`） | "Codebase 使用 camelCase 命名。Figma file 使用 slash-separated hierarchy。对于新 variable，是否应使用 slash-separated（Figma standard），并通过 code syntax 映射？" |

### Code 优先

以下情况默认以 code 作为事实来源：
- Hex value（code 是线上生产值）
- Token naming（CSS variable name 会成为 code syntax）
- Mode value（light/dark 拆分来自 code）

### Figma 优先

以下情况默认以 Figma 作为事实来源：
- Collection architecture（如果已有结构良好的 system，则扩展它，而不是替换它）
- Variable naming hierarchy（如果 designer 已经在使用具有特定名称的 system）
- Page structure（匹配现有 page 组织模式）

### 两者都不明显正确：协商

当两者都不明显正确时，提出解决方案并询问：
> "我建议采用 [option]。这样可以同时保留 code token name 和 Figma naming convention。这样可以吗？"
