> 属于 [figma-generate-library skill](../SKILL.md) 的一部分。

# Token 创建参考

本文档覆盖 Phase 1：创建 variable collections、modes、primitives、semantic aliases、scopes、code syntax、styles 和 validation。所有代码都可直接复制粘贴用于 `use_figma`。

---

## 1. Collection 架构

选择与你的 token 数量和复杂度匹配的模式：

### 简单模式（< 50 tokens）

一个 collection，2 个 modes。适用于小型项目或 brand kits。

```
Collection: "Tokens"    modes: ["Light", "Dark"]
  color/bg/primary → Light: #FFFFFF, Dark: #1A1A1A
  spacing/sm = 8
```

### 标准模式（50-200 tokens）- 推荐起点

将 primitives 与 semantics 分离。真实世界参考是 Figma 的 Simple Design System（SDS）：7 个 collections、368 个 variables，semantic colors 使用 light/dark modes，primitives 使用 single-mode。

```
Collection: "Primitives"    modes: ["Value"]       ← raw hex values, no modes
  blue/500 = #3B82F6
  gray/900 = #111827
  white/1000 = #FFFFFF

Collection: "Color"         modes: ["Light", "Dark"] ← aliases to Primitives
  color/bg/primary → Light: alias Primitives/white/1000, Dark: alias Primitives/gray/900
  color/text/primary → Light: alias Primitives/gray/900, Dark: alias Primitives/white/1000

Collection: "Spacing"       modes: ["Value"]
  spacing/xs = 4, spacing/sm = 8, spacing/md = 16, spacing/lg = 24, spacing/xl = 32

Collection: "Typography Primitives"  modes: ["Value"]
  family/sans = "Inter", scale/01 = 12, scale/02 = 14, scale/03 = 16, weight/regular = 400

Collection: "Typography"    modes: ["Value"]        ← aliases to Typography Primitives
  body/font-family → alias family/sans
  body/size-md → alias scale/03
```

### 高级模式（200+ tokens）- M3 Model

多个 semantic collections，4-8 个 modes。当需要 light/dark、contrast、brand 或 responsive breakpoints 的组合时使用。

```
Collection: "M3"           modes: ["Light", "Dark", "Light High Contrast", "Dark High Contrast", ...]
Collection: "Typeface"     modes: ["Baseline", "Wireframe"]
Collection: "Typescale"    modes: ["Value"]  ← aliases into Typeface
Collection: "Shape"        modes: ["Value"]
```

来自 M3 的关键洞察：全部 196 个 semantic color variables 都位于一个包含 8 个 modes 的单一 collection 中。只需切换一次 frame 的 mode，就能同时更新所有颜色。

---

## 2. 创建 Collections + Modes

### 创建 Primitives Collection

```javascript
const RUN_ID = "ds-build-2024-001"; // use the same RUN_ID throughout the build

// Create the collection
const primColl = figma.variables.createVariableCollection("Primitives");

// Rename the default "Mode 1" to "Value"
primColl.renameMode(primColl.modes[0].modeId, "Value");
const valueMode = primColl.modes[0].modeId;

// Tag for idempotency
primColl.setSharedPluginData('dsb', 'run_id', RUN_ID);
primColl.setSharedPluginData('dsb', 'key', 'collection/primitives');

return {
  collectionId: primColl.id,
  modeId: valueMode,
  name: primColl.name
};
```

### 创建带 Light/Dark Modes 的 Semantic Color Collection

```javascript
const RUN_ID = "ds-build-2024-001";

const colorColl = figma.variables.createVariableCollection("Color");

// Rename default "Mode 1" to "Light"
colorColl.renameMode(colorColl.modes[0].modeId, "Light");
const lightModeId = colorColl.modes[0].modeId;

// Add "Dark" mode — requires Professional plan or higher
// Throws "in addMode: Limited to N modes only" on Starter plan
const darkModeId = colorColl.addMode("Dark");

colorColl.setSharedPluginData('dsb', 'run_id', RUN_ID);
colorColl.setSharedPluginData('dsb', 'key', 'collection/color');

return {
  collectionId: colorColl.id,
  lightModeId,
  darkModeId
};
```

**Mode plan 限制**：Starter = 1 mode，Professional = 4 modes，Organization/Enterprise = 40 modes。如果 `addMode` 抛错，说明该文件位于 Starter plan。告知用户并询问如何继续。

### 创建 Spacing Collection（single mode）

```javascript
const RUN_ID = "ds-build-2024-001";

const spacingColl = figma.variables.createVariableCollection("Spacing");
spacingColl.renameMode(spacingColl.modes[0].modeId, "Value");
const valueMode = spacingColl.modes[0].modeId;

spacingColl.setSharedPluginData('dsb', 'run_id', RUN_ID);
spacingColl.setSharedPluginData('dsb', 'key', 'collection/spacing');

return {
  collectionId: spacingColl.id,
  modeId: valueMode
};
```

---

## 3. 创建所有 Variable Types

### hex 到 {r, g, b} 转换辅助函数

Figma Plugin API 中的 colors 使用 0-1 范围，而不是 0-255。将此辅助函数嵌入任何创建 color variables 的脚本中：

```javascript
function hexToRgb(hex) {
  const clean = hex.replace('#', '');
  return {
    r: parseInt(clean.substring(0, 2), 16) / 255,
    g: parseInt(clean.substring(2, 4), 16) / 255,
    b: parseInt(clean.substring(4, 6), 16) / 255
  };
}

// With alpha channel (for semi-transparent primitives like Black/200 at 10%):
function hexToRgba(hex) {
  const clean = hex.replace('#', '');
  const hasAlpha = clean.length === 8;
  return {
    r: parseInt(clean.substring(0, 2), 16) / 255,
    g: parseInt(clean.substring(2, 4), 16) / 255,
    b: parseInt(clean.substring(4, 6), 16) / 255,
    a: hasAlpha ? parseInt(clean.substring(6, 8), 16) / 255 : 1
  };
}

// Usage:
// hexToRgb('#3B82F6')        → {r: 0.231, g: 0.510, b: 0.965}
// hexToRgb('#14AE5C')        → {r: 0.078, g: 0.682, b: 0.361}
// hexToRgba('#0c0c0d1a')     → {r: 0.047, g: 0.047, b: 0.051, a: 0.102}
```

### 创建 Primitive Color Variables（真实 SDS 数据）

这会创建 Simple Design System 的 `Color Primitives` collection 的一个子集（Blue family，来自真实设计系统使用的标准模式）：

```javascript
function hexToRgb(hex) {
  const c = hex.replace('#', '');
  return { r: parseInt(c.slice(0,2),16)/255, g: parseInt(c.slice(2,4),16)/255, b: parseInt(c.slice(4,6),16)/255 };
}

const RUN_ID = "ds-build-2024-001";

// Get the Primitives collection created in the previous step
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const primColl = collections.find(c => c.getSharedPluginData('dsb', 'key') === 'collection/primitives');
if (!primColl) throw new Error("Primitives collection not found — run collection creation first");
const valueMode = primColl.modes[0].modeId;

    // Define primitives — use real values from your codebase
    const primitiveColors = [
      // Blue scale
      { name: 'blue/100', hex: '#EFF6FF' },
      { name: 'blue/200', hex: '#DBEAFE' },
      { name: 'blue/300', hex: '#93C5FD' },
      { name: 'blue/400', hex: '#60A5FA' },
      { name: 'blue/500', hex: '#3B82F6' },
      { name: 'blue/600', hex: '#2563EB' },
      { name: 'blue/700', hex: '#1D4ED8' },
      { name: 'blue/800', hex: '#1E40AF' },
      { name: 'blue/900', hex: '#1E3A8A' },
      // Gray scale
      { name: 'gray/100', hex: '#F9FAFB' },
      { name: 'gray/200', hex: '#F3F4F6' },
      { name: 'gray/300', hex: '#D1D5DB' },
      { name: 'gray/400', hex: '#9CA3AF' },
      { name: 'gray/500', hex: '#6B7280' },
      { name: 'gray/600', hex: '#4B5563' },
      { name: 'gray/700', hex: '#374151' },
      { name: 'gray/800', hex: '#1F2937' },
      { name: 'gray/900', hex: '#111827' },
      // White / Black
      { name: 'white/1000', hex: '#FFFFFF' },
      { name: 'black/1000', hex: '#000000' },
    ];

    const created = [];
    for (const { name, hex } of primitiveColors) {
      const v = figma.variables.createVariable(name, primColl, 'COLOR');
      v.setValueForMode(valueMode, hexToRgb(hex));
      // Primitives: EMPTY scopes (hidden from all pickers — designers use semantics)
      v.scopes = [];
      // Code syntax from the actual CSS variable name
      v.setVariableCodeSyntax('WEB', `var(--color-${name.replace('/', '-')})`);
      v.setSharedPluginData('dsb', 'run_id', RUN_ID);
      v.setSharedPluginData('dsb', 'key', `primitive/${name}`);
      created.push({ name, id: v.id });
    }

return { created, count: created.length };
```

**Primitives 的关键 scope 规则**：设置 `v.scopes = []`。这会从每个 picker 中隐藏 primitives。设计师应该只看到 semantic tokens。例外是半透明 overlay primitives（带 alpha 的 Black/White），它们使用 `["EFFECT_COLOR"]`，以便出现在 shadow pickers 中。

### 创建 FLOAT Variables（Spacing、Radius、Font Size）

```javascript
const RUN_ID = "ds-build-2024-001";
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const spacingColl = collections.find(c => c.getSharedPluginData('dsb', 'key') === 'collection/spacing');
if (!spacingColl) throw new Error("Spacing collection not found");
const valueMode = spacingColl.modes[0].modeId;

const spacingTokens = [
  { name: 'spacing/xs',  value: 4,  scope: 'GAP', cssVar: '--spacing-xs' },
  { name: 'spacing/sm',  value: 8,  scope: 'GAP', cssVar: '--spacing-sm' },
  { name: 'spacing/md',  value: 16, scope: 'GAP', cssVar: '--spacing-md' },
  { name: 'spacing/lg',  value: 24, scope: 'GAP', cssVar: '--spacing-lg' },
  { name: 'spacing/xl',  value: 32, scope: 'GAP', cssVar: '--spacing-xl' },
  { name: 'spacing/2xl', value: 48, scope: 'GAP', cssVar: '--spacing-2xl' },
];

const radiusTokens = [
  { name: 'radius/none', value: 0,    scope: 'CORNER_RADIUS', cssVar: '--radius-none' },
  { name: 'radius/sm',   value: 4,    scope: 'CORNER_RADIUS', cssVar: '--radius-sm' },
  { name: 'radius/md',   value: 8,    scope: 'CORNER_RADIUS', cssVar: '--radius-md' },
  { name: 'radius/lg',   value: 16,   scope: 'CORNER_RADIUS', cssVar: '--radius-lg' },
  { name: 'radius/full', value: 9999, scope: 'CORNER_RADIUS', cssVar: '--radius-full' },
];

const created = [];
for (const { name, value, scope, cssVar } of [...spacingTokens, ...radiusTokens]) {
  const v = figma.variables.createVariable(name, spacingColl, 'FLOAT');
  v.setValueForMode(valueMode, value);
  v.scopes = [scope];
  v.setVariableCodeSyntax('WEB', `var(${cssVar})`);
  v.setSharedPluginData('dsb', 'run_id', RUN_ID);
  v.setSharedPluginData('dsb', 'key', name);
  created.push({ name, value, id: v.id });
}

return { created, count: created.length };
```

### 创建 STRING Variables（Font Family、Font Style）

```javascript
const RUN_ID = "ds-build-2024-001";
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const typoPrimColl = collections.find(c => c.getSharedPluginData('dsb', 'key') === 'collection/typography-primitives');
if (!typoPrimColl) throw new Error("Typography Primitives collection not found");
const valueMode = typoPrimColl.modes[0].modeId;

const fontTokens = [
  { name: 'family/sans',  value: 'Inter',       scope: 'FONT_FAMILY', cssVar: '--font-family-sans' },
  { name: 'family/mono',  value: 'Roboto Mono',  scope: 'FONT_FAMILY', cssVar: '--font-family-mono' },
  // Font style strings — these are the Figma fontName.style values:
  { name: 'weight/regular',  value: 'Regular',   scope: 'FONT_STYLE',  cssVar: '--font-weight-regular' },
  { name: 'weight/medium',   value: 'Medium',    scope: 'FONT_STYLE',  cssVar: '--font-weight-medium' },
  { name: 'weight/semibold', value: 'Semi Bold', scope: 'FONT_STYLE',  cssVar: '--font-weight-semibold' },
  { name: 'weight/bold',     value: 'Bold',      scope: 'FONT_STYLE',  cssVar: '--font-weight-bold' },
];

const created = [];
for (const { name, value, scope, cssVar } of fontTokens) {
  const v = figma.variables.createVariable(name, typoPrimColl, 'STRING');
  v.setValueForMode(valueMode, value);
  v.scopes = [scope];
  v.setVariableCodeSyntax('WEB', `var(${cssVar})`);
  v.setSharedPluginData('dsb', 'run_id', RUN_ID);
  v.setSharedPluginData('dsb', 'key', `typo-prim/${name}`);
  created.push({ name, value, id: v.id });
}

return { created, count: created.length };
```

### 创建 BOOLEAN Variables

BOOLEAN variables 没有 scopes（BOOLEAN type 不支持 scopes）。

```javascript
const RUN_ID = "ds-build-2024-001";
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const coll = collections.find(c => c.getSharedPluginData('dsb', 'key') === 'collection/tokens');
if (!coll) throw new Error("Collection not found");
const valueMode = coll.modes[0].modeId;

const v = figma.variables.createVariable('feature-flags/show-beta-badge', coll, 'BOOLEAN');
v.setValueForMode(valueMode, false);
// No scopes — BOOLEAN does not support scopes
v.setSharedPluginData('dsb', 'run_id', RUN_ID);
v.setSharedPluginData('dsb', 'key', 'feature-flags/show-beta-badge');

return { id: v.id, name: v.name };
```

---

## 4. Variable Aliasing（VARIABLE_ALIAS）：Primitive 到 Semantic 链

Semantic tokens 通过 `VARIABLE_ALIAS` 引用 primitives。这是 light/dark theming 能够工作的核心模式。

**架构：**
```
Color Primitives collection (1 mode: Value)
  blue/500 = #3B82F6          ← raw value

Color collection (2 modes: Light, Dark)
  color/bg/accent/default:
    Light → VARIABLE_ALIAS → Primitives/blue/500
    Dark  → VARIABLE_ALIAS → Primitives/blue/300
```

### 完整的 Semantic Alias 创建脚本（SDS-style）

```javascript
function hexToRgb(hex) {
  const c = hex.replace('#', '');
  return { r: parseInt(c.slice(0,2),16)/255, g: parseInt(c.slice(2,4),16)/255, b: parseInt(c.slice(4,6),16)/255 };
}

const RUN_ID = "ds-build-2024-001";
const collections = await figma.variables.getLocalVariableCollectionsAsync();

const primColl = collections.find(c => c.getSharedPluginData('dsb', 'key') === 'collection/primitives');
const colorColl = collections.find(c => c.getSharedPluginData('dsb', 'key') === 'collection/color');
if (!primColl || !colorColl) throw new Error("Collections not found — run primitive/color collection creation first");

const primValueMode = primColl.modes[0].modeId;
const lightModeId = colorColl.modes.find(m => m.name === 'Light').modeId;
const darkModeId = colorColl.modes.find(m => m.name === 'Dark').modeId;

// Load all primitive variables for lookup
const allVars = await figma.variables.getLocalVariablesAsync();
const primsByKey = {};
for (const v of allVars) {
  if (v.variableCollectionId === primColl.id) {
    primsByKey[v.getSharedPluginData('dsb', 'key')] = v;
  }
}

function getPrim(name) {
  const v = primsByKey[`primitive/${name}`];
  if (!v) throw new Error(`Primitive not found: primitive/${name}`);
  return v;
}

// Define semantic → [lightPrimitiveName, darkPrimitiveName]
// Following the SDS pattern: Background/{Intent}/{Emphasis}
const semanticColors = [
  // Background
  { name: 'color/bg/default/default',   lightPrim: 'white/1000', darkPrim: 'gray/900',
    cssVar: '--color-bg-default-default', scopes: ['FRAME_FILL', 'SHAPE_FILL'] },
  { name: 'color/bg/default/secondary', lightPrim: 'gray/100', darkPrim: 'gray/800',
    cssVar: '--color-bg-default-secondary', scopes: ['FRAME_FILL', 'SHAPE_FILL'] },
  { name: 'color/bg/brand/default',     lightPrim: 'blue/600', darkPrim: 'blue/300',
    cssVar: '--color-bg-brand-default', scopes: ['FRAME_FILL', 'SHAPE_FILL'] },
  // Text
  { name: 'color/text/default/default', lightPrim: 'gray/900', darkPrim: 'white/1000',
    cssVar: '--color-text-default-default', scopes: ['TEXT_FILL'] },
  { name: 'color/text/default/secondary', lightPrim: 'gray/500', darkPrim: 'gray/400',
    cssVar: '--color-text-default-secondary', scopes: ['TEXT_FILL'] },
  { name: 'color/text/brand/default',   lightPrim: 'blue/700', darkPrim: 'blue/200',
    cssVar: '--color-text-brand-default', scopes: ['TEXT_FILL'] },
  // Border
  { name: 'color/border/default/default', lightPrim: 'gray/300', darkPrim: 'gray/600',
    cssVar: '--color-border-default-default', scopes: ['STROKE_COLOR'] },
  { name: 'color/border/brand/default',   lightPrim: 'blue/500', darkPrim: 'blue/400',
    cssVar: '--color-border-brand-default', scopes: ['STROKE_COLOR'] },
];

const created = [];
for (const { name, lightPrim, darkPrim, cssVar, scopes } of semanticColors) {
  const v = figma.variables.createVariable(name, colorColl, 'COLOR');
  // Alias to primitive in Light mode
  v.setValueForMode(lightModeId, figma.variables.createVariableAlias(getPrim(lightPrim)));
  // Alias to primitive in Dark mode
  v.setValueForMode(darkModeId, figma.variables.createVariableAlias(getPrim(darkPrim)));
  // Set scopes (semantic layer — these ARE shown in pickers)
  v.scopes = scopes;
  // Code syntax
  v.setVariableCodeSyntax('WEB', `var(${cssVar})`);
  v.setSharedPluginData('dsb', 'run_id', RUN_ID);
  v.setSharedPluginData('dsb', 'key', name);
  created.push({ name, id: v.id });
}

return { created, count: created.length };
```

**关键 API 要点：**
- `figma.variables.createVariableAlias(variable)` 接收 Variable 对象，并返回 `{type:'VARIABLE_ALIAS', id: variable.id}`
- 被 alias 的 variable 必须与 semantic variable 拥有相同的 `resolvedType`
- 绝不要在 semantic layer 中复制 raw values，始终使用 alias

---

## 5. Variable Scopes 完整参考表

| Semantic Role | 推荐 Scopes | Variable Type |
|---|---|---|
| Primitive colors（raw） | `[]`，为空，从所有 pickers 中隐藏 | COLOR |
| Semi-transparent overlay primitives | `["EFFECT_COLOR"]` | COLOR |
| Background fills（frame、shape） | `["FRAME_FILL", "SHAPE_FILL"]` | COLOR |
| Text color | `["TEXT_FILL"]` | COLOR |
| Icon / shape fill | `["SHAPE_FILL", "STROKE_COLOR"]` | COLOR |
| Border / stroke color | `["STROKE_COLOR"]` | COLOR |
| Background + border 组合 | `["FRAME_FILL", "SHAPE_FILL", "STROKE_COLOR"]` | COLOR |
| Shadow color | `["EFFECT_COLOR"]` | COLOR |
| Spacing / items 之间的 gap | `["GAP"]` | FLOAT |
| Padding（如果与 gap 分开） | `["GAP"]` | FLOAT |
| Corner radius | `["CORNER_RADIUS"]` | FLOAT |
| Width / height dimensions | `["WIDTH_HEIGHT"]` | FLOAT |
| Font size | `["FONT_SIZE"]` | FLOAT |
| Line height | `["LINE_HEIGHT"]` | FLOAT |
| Letter spacing | `["LETTER_SPACING"]` | FLOAT |
| Font weight（numeric） | `["FONT_WEIGHT"]` | FLOAT |
| Stroke width | `["STROKE_FLOAT"]` | FLOAT |
| Effect blur radius | `["EFFECT_FLOAT"]` | FLOAT |
| Opacity | `["OPACITY"]` | FLOAT |
| Font family | `["FONT_FAMILY"]` | STRING |
| Font style（例如 "Semi Bold"） | `["FONT_STYLE"]` | STRING |
| Boolean flags | *（不支持 scopes）* | BOOLEAN |

**绝不要在任何 variable 上使用 `ALL_SCOPES`**。它会用无关 tokens 污染每个 picker。作为黄金标准的 Simple Design System（SDS）在每个 variable 上都使用定向 scopes。

**`ALL_FILLS` 说明**：`ALL_FILLS` 在 fill scopes 中是排他的，它同时覆盖 `FRAME_FILL`、`SHAPE_FILL` 和 `TEXT_FILL`。如果设置了它，就不能再添加单独的 fill scopes。为保证精确性，优先指定单独 scopes。

### 批量设置 Scope（Variables 创建后）

如果你创建 variables 时没有设置 scopes，并且需要批量设置：

```javascript
const allVars = await figma.variables.getLocalVariablesAsync();

// Scope mapping: partial name match → scopes
const scopeRules = [
  { match: 'color/bg/',     scopes: ['FRAME_FILL', 'SHAPE_FILL'] },
  { match: 'color/text/',   scopes: ['TEXT_FILL'] },
  { match: 'color/icon/',   scopes: ['SHAPE_FILL', 'STROKE_COLOR'] },
  { match: 'color/border/', scopes: ['STROKE_COLOR'] },
  { match: 'spacing/',      scopes: ['GAP'] },
  { match: 'radius/',       scopes: ['CORNER_RADIUS'] },
  { match: 'blue/',         scopes: [] },   // primitives — hide
  { match: 'gray/',         scopes: [] },
  { match: 'white/',        scopes: [] },
  { match: 'black/',        scopes: [] },
];

const updated = [];
for (const v of allVars) {
  if (v.remote) continue; // skip library variables
  for (const rule of scopeRules) {
    if (v.name.startsWith(rule.match)) {
      v.scopes = rule.scopes;
      updated.push({ name: v.name, scopes: rule.scopes });
      break;
    }
  }
}

return { updated, count: updated.length };
```

---

## 6. Code Syntax：WEB/ANDROID/iOS

每个 variable 都必须设置 code syntax。这是 developer handoff 体验的基础：

**code syntax 的作用**：当开发者在 Figma Dev Mode 中检查任何具有 variable-bound property（fill、padding、radius 等）的元素时，显示的代码片段会使用 variable 的 code syntax name，而不是 Figma variable name。例如，绑定到 `color/bg/primary` 的 button background fill 会在 CSS snippet 中显示 `background: var(--color-bg-primary)`，而不是 `color/bg/primary`。如果未设置 code syntax，Dev Mode 会显示 raw hex values 或没有用的信息。

每个 variable 最多可以设置 **3 种 syntaxes**，每个平台一种（Web、iOS、Android）。如果 codebase 面向多个平台，就全部设置；如果只是 web-only 项目，则只设置 WEB。

```javascript
// WEB: MUST include the var() wrapper — this is the full CSS function syntax
variable.setVariableCodeSyntax('WEB', 'var(--color-bg-primary)');
//                                     ^^^^                   ^
//                              var() wrapper is REQUIRED

// ANDROID: Kotlin property name — camelCase, no wrapper
variable.setVariableCodeSyntax('ANDROID', 'colorBgPrimary');

// iOS: Swift property — dot-notation, no wrapper
variable.setVariableCodeSyntax('iOS', 'Color.bgPrimary');
```

> **关键：WEB code syntax 必须使用 `var()` wrapper。** 只设置 `--color-bg-primary`（不带 `var()`）会导致 Dev Mode 显示 raw hex values，而不是 CSS variable reference。始终使用完整的 `var(--name)` 形式。ANDROID 和 iOS 不使用 wrapper。

**根据 CSS variable name 推导平台语法的规则：**

| Platform | Pattern | Example |
|---|---|---|
| WEB | **`var(--{css-var-name})`**，需要 `var()` wrapper | `var(--sds-color-bg-primary)` |
| ANDROID | camelCase，无 wrapper，移除 `--` 前缀 | `sdsColorBgPrimary` |
| iOS | `.` 后使用 PascalCase，无 wrapper，移除 `--` 前缀 | `Color.SdsColorBgPrimary` or `Color.bgPrimary` |

**始终使用来自 codebase 的实际 CSS variable name**，不要从 Figma variable name 推导。如果代码使用 `--sds-color-background-brand-default`，那么这个确切字符串就是 WEB code syntax（再加上你补充的 `var()` wrapper）。

### 批量设置 Code Syntax

```javascript
const allVars = await figma.variables.getLocalVariablesAsync();
const updated = [];

for (const v of allVars) {
  if (v.remote) continue;
  // If code syntax already set, skip
  if (v.codeSyntax['WEB']) continue;

  // FALLBACK: derive from Figma name: color/bg/primary → var(--color-bg-primary)
  // PREFERRED: pass in a cssVarMap built from actual codebase CSS variable names
  // e.g. cssVarMap = { 'color/bg/primary': '--color-bg-primary', ... }
  const cssName = cssVarMap?.[v.name]
    ?? v.name.replace(/\//g, '-').replace(/\s/g, '-').toLowerCase();
  v.setVariableCodeSyntax('WEB', `var(--${cssName})`);
  updated.push({ name: v.name, web: `var(--${cssName})` });
}

return { updated, count: updated.length };
```

注意：推导名称只作为 fallback。只要已知 codebase 中的实际 CSS variable names，就始终优先用它们覆盖。

---

## 7. Effect Styles（Shadows）和 Text Styles

Shadows 和 composite typography 不能作为 variables，它们是 Styles。

### 创建 Effect Styles（Shadows）

参考 SDS（15 个 effect styles）以及 SDS shadow 模式 `Shadow/{Level}`：

```javascript
const RUN_ID = "ds-build-2024-001";

// Shadow definitions — CSS equivalent in comments
// CSS: 0 1px 2px rgba(0,0,0,0.05)
const shadows = [
  {
    name: 'Shadow/Subtle',
    effects: [{
      type: 'DROP_SHADOW',
      color: { r: 0, g: 0, b: 0, a: 0.05 },
      offset: { x: 0, y: 1 },
      radius: 2,
      spread: 0,
      visible: true,
      blendMode: 'NORMAL'
    }]
  },
  {
    // CSS: 0 4px 6px -1px rgba(0,0,0,0.10), 0 2px 4px -1px rgba(0,0,0,0.06)
    name: 'Shadow/Medium',
    effects: [
      {
        type: 'DROP_SHADOW',
        color: { r: 0, g: 0, b: 0, a: 0.10 },
        offset: { x: 0, y: 4 },
        radius: 6,
        spread: -1,
        visible: true,
        blendMode: 'NORMAL'
      },
      {
        type: 'DROP_SHADOW',
        color: { r: 0, g: 0, b: 0, a: 0.06 },
        offset: { x: 0, y: 2 },
        radius: 4,
        spread: -1,
        visible: true,
        blendMode: 'NORMAL'
      }
    ]
  },
  {
    // CSS: 0 10px 15px -3px rgba(0,0,0,0.10), 0 4px 6px -2px rgba(0,0,0,0.05)
    name: 'Shadow/Strong',
    effects: [
      {
        type: 'DROP_SHADOW',
        color: { r: 0, g: 0, b: 0, a: 0.10 },
        offset: { x: 0, y: 10 },
        radius: 15,
        spread: -3,
        visible: true,
        blendMode: 'NORMAL'
      },
      {
        type: 'DROP_SHADOW',
        color: { r: 0, g: 0, b: 0, a: 0.05 },
        offset: { x: 0, y: 4 },
        radius: 6,
        spread: -2,
        visible: true,
        blendMode: 'NORMAL'
      }
    ]
  }
];

// M3-style dual shadow (umbra + penumbra pattern):
const m3Shadows = [
  {
    name: 'Elevation/1',
    effects: [
      { type: 'DROP_SHADOW', color: {r:0,g:0,b:0,a:0.30}, offset:{x:0,y:1}, radius:2, spread:0, visible:true, blendMode:'NORMAL' },
      { type: 'DROP_SHADOW', color: {r:0,g:0,b:0,a:0.15}, offset:{x:0,y:1}, radius:3, spread:1, visible:true, blendMode:'NORMAL' }
    ]
  },
  {
    name: 'Elevation/2',
    effects: [
      { type: 'DROP_SHADOW', color: {r:0,g:0,b:0,a:0.30}, offset:{x:0,y:1}, radius:2, spread:0, visible:true, blendMode:'NORMAL' },
      { type: 'DROP_SHADOW', color: {r:0,g:0,b:0,a:0.15}, offset:{x:0,y:2}, radius:6, spread:2, visible:true, blendMode:'NORMAL' }
    ]
  },
  {
    name: 'Elevation/3',
    effects: [
      { type: 'DROP_SHADOW', color: {r:0,g:0,b:0,a:0.30}, offset:{x:0,y:1}, radius:3, spread:0, visible:true, blendMode:'NORMAL' },
      { type: 'DROP_SHADOW', color: {r:0,g:0,b:0,a:0.15}, offset:{x:0,y:4}, radius:8, spread:3, visible:true, blendMode:'NORMAL' }
    ]
  }
];

const created = [];
for (const { name, effects } of shadows) {
  const style = figma.createEffectStyle();
  style.name = name;
  style.effects = effects;
  style.setSharedPluginData('dsb', 'run_id', RUN_ID);
  style.setSharedPluginData('dsb', 'key', `effect-style/${name}`);
  created.push({ name, id: style.id });
}

return { created, count: created.length };
```

### 创建 Text Styles

创建 text styles 前必须先加载 fonts。

```javascript
const RUN_ID = "ds-build-2024-001";

// Define text styles — based on SDS typography hierarchy
const textStyles = [
  // Display / Hero
  { name: 'Display/Hero',    family: 'Inter', style: 'Bold',      size: 72, lineHeight: 80, letterSpacing: -1.5 },
  // Headings
  { name: 'Heading/H1',      family: 'Inter', style: 'Bold',      size: 48, lineHeight: 56, letterSpacing: -1.0 },
  { name: 'Heading/H2',      family: 'Inter', style: 'Bold',      size: 40, lineHeight: 48, letterSpacing: -0.5 },
  { name: 'Heading/H3',      family: 'Inter', style: 'Semi Bold', size: 32, lineHeight: 40, letterSpacing: 0 },
  { name: 'Heading/H4',      family: 'Inter', style: 'Semi Bold', size: 24, lineHeight: 32, letterSpacing: 0 },
  // Body
  { name: 'Body/Large',      family: 'Inter', style: 'Regular',   size: 18, lineHeight: 28, letterSpacing: 0 },
  { name: 'Body/Medium',     family: 'Inter', style: 'Regular',   size: 16, lineHeight: 24, letterSpacing: 0 },
  { name: 'Body/Small',      family: 'Inter', style: 'Regular',   size: 14, lineHeight: 20, letterSpacing: 0 },
  // Label
  { name: 'Label/Large',     family: 'Inter', style: 'Medium',    size: 14, lineHeight: 20, letterSpacing: 0.1 },
  { name: 'Label/Medium',    family: 'Inter', style: 'Medium',    size: 12, lineHeight: 16, letterSpacing: 0.5 },
  { name: 'Label/Small',     family: 'Inter', style: 'Medium',    size: 11, lineHeight: 16, letterSpacing: 0.5 },
  // Code
  { name: 'Code/Base',       family: 'Roboto Mono', style: 'Regular', size: 14, lineHeight: 20, letterSpacing: 0 },
];

// Load all required fonts first
const fontSet = new Set(textStyles.map(s => JSON.stringify({ family: s.family, style: s.style })));
await Promise.all([...fontSet].map(f => figma.loadFontAsync(JSON.parse(f))));

const created = [];
for (const { name, family, style, size, lineHeight, letterSpacing } of textStyles) {
  const ts = figma.createTextStyle();
  ts.name = name;
  ts.fontName = { family, style };
  ts.fontSize = size;
  ts.lineHeight = { value: lineHeight, unit: 'PIXELS' };
  ts.letterSpacing = { value: letterSpacing, unit: 'PIXELS' };
  ts.setSharedPluginData('dsb', 'run_id', RUN_ID);
  ts.setSharedPluginData('dsb', 'key', `text-style/${name}`);
  created.push({ name, id: ts.id });
}

return { created, count: created.length };
```

---

## 8. 幂等性：创建前检查模式

每个创建脚本都应该在创建实体前检查它是否已经存在。这可以防止脚本在部分失败后重新运行时产生重复项。

### Collections 的创建前检查

```javascript
const DSB_KEY = 'collection/primitives';
const RUN_ID = "ds-build-2024-001";

// Check if already exists
const existing = await figma.variables.getLocalVariableCollectionsAsync();
let primColl = existing.find(c => c.getSharedPluginData('dsb', 'key') === DSB_KEY);

if (primColl) {
  return { status: 'already_exists', collectionId: primColl.id, name: primColl.name };
}

// Create only if not found
primColl = figma.variables.createVariableCollection("Primitives");
primColl.renameMode(primColl.modes[0].modeId, "Value");
primColl.setSharedPluginData('dsb', 'run_id', RUN_ID);
primColl.setSharedPluginData('dsb', 'key', DSB_KEY);

return { status: 'created', collectionId: primColl.id };
```

### Variables 的创建前检查

```javascript
const VARIABLE_KEY = 'primitive/blue/500';
const RUN_ID = "ds-build-2024-001";

// Check if already exists by sharedPluginData key
const allVars = await figma.variables.getLocalVariablesAsync();
const existing = allVars.find(v => v.getSharedPluginData('dsb', 'key') === VARIABLE_KEY);

if (existing) {
  return { status: 'already_exists', id: existing.id, name: existing.name };
}

// ... create the variable ...
return { status: 'created' };
```

### sharedPluginData 打标策略

每个节点创建后立即打标。`key` 是用于幂等检查的稳定逻辑标识符。`run_id` 标识创建它的 build run（对清理有用）。

```javascript
node.setSharedPluginData('dsb', 'run_id', RUN_ID);       // build run ID
node.setSharedPluginData('dsb', 'phase', 'phase1');       // which phase
node.setSharedPluginData('dsb', 'key', 'color/bg/primary'); // stable logical key
```

**按 run ID 清理（安全，只定位已打标节点，绝不定位用户拥有的节点）：**

```javascript
const TARGET_RUN_ID = "ds-build-2024-001"; // run to remove
const allVars = await figma.variables.getLocalVariablesAsync();
const removed = [];
for (const v of allVars) {
  if (v.getSharedPluginData('dsb', 'run_id') === TARGET_RUN_ID) {
    removed.push(v.name);
    v.remove();
  }
}
return { removed, count: removed.length };
```

**绝不要按名称前缀清理**（例如删除所有以 `color/` 开头的内容）。这会破坏恰好共享该前缀的用户自建 variables。

---

## 9. Validation：验证 Counts、Aliases 和 Scopes

在 Phase 1 后运行这些脚本，确认所有内容都已正确创建，再进入 Phase 2。

### 验证 Collection 和 Variable 数量

```javascript
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const allVars = await figma.variables.getLocalVariablesAsync();

const summary = collections.map(c => {
  const vars = allVars.filter(v => v.variableCollectionId === c.id);
  return {
    name: c.name,
    id: c.id,
    modes: c.modes.map(m => m.name),
    variableCount: vars.length,
    missingScopes: vars.filter(v => v.scopes.length === 0 && v.resolvedType !== 'BOOLEAN').length,
    missingCodeSyntax: vars.filter(v => !v.codeSyntax['WEB'] && !v.remote).length,
    sampleVariables: vars.slice(0, 3).map(v => v.name)
  };
});

return {
  collectionCount: collections.length,
  totalVariables: allVars.length,
  collections: summary
};
```

解释：`missingScopes > 0`（针对非 primitives 且非 BOOLEANs）表示 scope-setting 失败，需要重新运行 scope 脚本。`missingCodeSyntax > 0` 表示 code syntax 未设置，需要运行批量 code syntax 脚本。

注意：primitives 正确状态是 `scopes = []`（为空、隐藏）。上面的 `missingScopes` 会统计 scopes 为空的非 BOOLEAN variables，请审查列表以确认它们全部都是 primitives。

### 验证 Aliases 可解析

```javascript
const allVars = await figma.variables.getLocalVariablesAsync();
const collections = await figma.variables.getLocalVariableCollectionsAsync();

const brokenAliases = [];
const aliasedVars = [];

for (const v of allVars) {
  if (v.remote) continue;
  const coll = collections.find(c => c.id === v.variableCollectionId);
  if (!coll) continue;

  for (const [modeId, val] of Object.entries(v.valuesByMode)) {
    if (val && typeof val === 'object' && val.type === 'VARIABLE_ALIAS') {
      aliasedVars.push({ name: v.name, aliasTargetId: val.id });
      // Verify the target exists
      const target = allVars.find(t => t.id === val.id);
      if (!target) {
        brokenAliases.push({ variable: v.name, modeId, missingTargetId: val.id });
      }
    }
  }
}

return {
  totalAliased: aliasedVars.length,
  brokenAliases,
  brokenCount: brokenAliases.length,
  status: brokenAliases.length === 0 ? 'all_aliases_resolve' : 'BROKEN_ALIASES_FOUND'
};
```

解释：`brokenCount > 0` 表示某个 semantic variable 引用了已删除或尚未创建的 primitive。创建缺失的 primitives，然后针对受影响的 semantic variables 重新运行 alias 创建。

### 验证 Style 数量

```javascript
const [textStyles, effectStyles] = await Promise.all([
  figma.getLocalTextStylesAsync(),
  figma.getLocalEffectStylesAsync()
]);

return {
  textStyles: textStyles.map(s => ({ name: s.name, fontSize: s.fontSize, fontFamily: s.fontName.family })),
  effectStyles: effectStyles.map(s => ({ name: s.name, effectCount: s.effects.length })),
  counts: { text: textStyles.length, effect: effectStyles.length }
};
```

### Phase 1 退出标准清单

进入 Phase 2 之前，确认以下所有项目：

- 每个计划内 collection 都存在，并且 mode 数量正确
- Primitive variables：`scopes = []`，code syntax 已设置
- Semantic variables：targeted scopes 已设置，code syntax 已设置，aliases 指向 primitives（不是 raw values）
- 所有 broken alias count = 0
- 所有计划内 text styles 都存在，并且 font family/size/weight 正确
- 所有计划内 effect styles 都存在，并且 shadow values 正确
- 除非用户明确批准，否则没有 variable 使用 `ALL_SCOPES`
