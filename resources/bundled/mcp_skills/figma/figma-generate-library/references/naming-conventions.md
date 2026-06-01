> 属于 [figma-generate-library skill](../SKILL.md) 的一部分。

# 命名约定参考

本参考记录 figma-generate-library 工作流中使用的所有命名约定。按顺序覆盖所有命名决策：variables、components、pages、variants、styles、separators、status indicators。最后一节说明何时匹配既有文件约定，何时使用这里的默认值。

---

## 1. Variable 命名

### Slash 层级（通用模式）

所有 Figma variables 都使用 slash 分隔路径。slash 会在 Variables 面板中创建可视化分组，并直接映射到代码中的 token 层级。

```
{category}/{subcategory}/{role}
```

来自 Simple DS 和 Material 3 的真实示例：

```
color/bg/primary
color/bg/secondary
color/text/primary
color/text/muted
color/border/default
color/border/focus
color/feedback/error
color/feedback/success
spacing/xs
spacing/sm
spacing/md
spacing/lg
spacing/xl
spacing/2xl
radius/none
radius/sm
radius/md
radius/lg
radius/full
typography/body/font-size
typography/body/line-height
typography/heading/font-size
typography/heading/font-weight
```

### Primitives collection

Primitive variables 保存原始值，并且**不**暴露给使用者（scope = `[]`）。它们使用扁平的 `{family}/{step}` 格式，与 Simple DS 的 color scale 约定一致：

```
blue/50
blue/100
blue/200
...
blue/900
gray/50
gray/100
...
gray/900
red/500
green/500
```

Step 数字遵循目标 codebase 的约定。如果 codebase 使用 `100-900`，就使用该范围。如果使用 `50-950`，就使用该范围。如果没有 codebase 约定，则使用 `100-900`，步长为 100。

### Semantic collection

Semantic variables 以 primitives 作为 alias。它们使用基于角色的 `{category}/{role}` 或 `{category}/{subcategory}/{role}` 模式：

```
color/bg/primary         → alias: primitives/white (light), primitives/gray/900 (dark)
color/bg/secondary       → alias: primitives/gray/100 (light), primitives/gray/800 (dark)
color/text/primary       → alias: primitives/gray/900 (light), primitives/white (dark)
color/text/secondary     → alias: primitives/gray/600 (light), primitives/gray/400 (dark)
color/border/default     → alias: primitives/gray/200 (light), primitives/gray/700 (dark)
```

**规则**：Semantic variables 绝不能保存原始 hex 值，它们必须始终 alias 到 primitive。如果需要新的 color value，先创建 primitive，再创建 semantic alias。

### 大小写

**默认**：使用带 forward slashes 的 **lowercase**：`color/bg/primary`、`spacing/2xl`。

**何时偏离：**
- 如果既有文件使用 PascalCase（例如 Material 3 使用 `Schemes/Primary`），就匹配它。
- 如果设计团队为了 Variables 面板中的可读性而偏好 PascalCase，也可以接受，前提是 code syntax 单独定义，并使用符合平台的大小写。
- Mode names 可以使用空格和 mixed case（例如 `SDS Light`、`Mode 1 -> Light`），这些是标签，不是标识符。

**绝不要**：在 variable names 中使用 camelCase（把 `colorBgPrimary` 作为 Figma name 是错误的，它只属于 Android code syntax）。绝不要在 path segment 内使用空格：`color/bg primary` 是错误的；`color/bg/primary` 是正确的。

**关键区别**：大小写规则适用于 *Figma variable names*。无论 Figma name 使用什么大小写，code syntax names 都遵循*平台约定*，完整说明见第 9 节。

---

## 2. Component 命名

### 主 components：PascalCase，无前缀

面向 library consumers 发布的 components 使用普通 PascalCase 名称：

```
Button
Input
Checkbox
Toggle
Avatar
Badge
Card
Dialog
Tooltip
Banner
```

不要给 public components 使用 namespace 前缀（例如不要命名为 `DS/Button` 或 `sds-Button`）。component names 中的 slash 会在 Assets 面板中创建嵌套分组，这适用于 sub-components，但不适用于顶层 public components。

### Sub-components：underscore 前缀 + slash namespace

不面向 library consumers 的内部 sub-components 使用 `_` 前缀。这会默认将它们从 Assets 面板中隐藏，并向其他设计师表明它们不应被直接使用。

```
_Button/Slot           (internal icon slot for Button)
_Input/Indicator       (internal state indicator for Input)
_Badge/Dot             (internal dot sub-component of Badge)
_Parts/Avatar.Status   (UI3 pattern: _Parts/{ParentName}.{SubPart})
_Slider/Handle         (UI3 pattern: _{ParentName}/{SubPart})
```

模式规则：
- 所有内部 sub-components 都使用 `_` 前缀，没有例外。
- 使用 slash namespacing 将 sub-components 分组到其 parent 下面：`_Button/IconSlot`。
- 对于多个 parents 共享的 sub-components，使用 `_Parts/{ComponentName}.{SubPart}`。

### Private documentation components

仅用于内部文档（不用于生产）的 components 使用 `.` 前缀：

```
.ExampleCard
.GuidelineHeader
.DemoFrame
```

这会向 consumers 隐藏它们，同时让它们仍可在 canvas 上访问。

---

## 3. Page 命名

五个参考设计系统使用三种不同的命名模式。选择一种模式，并在文件中的所有 pages 上保持一致。

### 模式 1：普通名称（Simple DS、Material 3、Polaris）

最常见的模式。干净、可读、无装饰。

```
Cover
---
Foundations
Icons
---
Accordion
Avatars
Buttons
Cards
Dialog
Inputs
Menu
---
Utilities
Component Playground
```

从零开始时，或目标文件已使用这种风格时，使用此模式。

### 模式 2：Emoji 前缀 + status（UI3 Library）

表达力最强的模式。page name 编码 asset type、design status 和 code readiness。

结构：`[Asset Type Emoji] [Optional FPL Label] [Status Circle] Component Name [Code Status Bracket]`

| 片段 | 取值 |
|---------|--------|
| Asset type | Component pages 使用 C-flag emoji；pattern pages 使用 P-flag emoji |
| Design status | Green circle = Ready，Yellow circle = WIP，Red circle = Do not use |
| Code status | (none) = Ready in code，`[beta]` = Beta，`[future]` = Not yet built |

示例：
```
Overview
Status Key
---
FPL COMPONENTS (go/fpl)
[C-flag] FPL [Green] Buttons
[C-flag] FPL [Green] Inputs
[C-flag] FPL [Yellow] Popovers [future]
---
UI3 COMPONENTS
[C-flag] [Green] Comments
---
PATTERNS
[P-flag] [Green] Editor / Layers
---
[Book] Cover
[Headstone] Deprecated
```

仅在构建需要生命周期跟踪的大型多团队设计系统时，或目标文件已使用此模式时，才使用该模式。

### 模式 3：Emoji 前缀（Shop Minis）

这是 UI3 模式的轻量版本，不包含 status circles。

```
📔 Cover
ℹ️ About
🚀 Getting started
——— THEME ———
Color
Typography
Spacing
——— COMPONENTS ———
Button
Input
Card
```

当目标文件已使用 emoji 前缀但不需要生命周期跟踪时，使用此模式。

### 通用规则（所有模式）

- **Cover** 始终放在第一位。
- **Separator pages** 放在每个逻辑 section 的前后。
- **Foundation/token pages** 始终放在 component pages 之前。
- **Utility and internal pages** 始终放在最后。
- 选择一种约定，不要在同一文件中混用模式。

---

## 4. Variant 命名

### Property=Value 格式

所有 component variant properties 及其 values 在 Figma component set 中都使用 `Property=Value` 格式：

```
Size=Small, Style=Primary, State=Default
Size=Medium, Style=Secondary, State=Hover
Size=Large, Style=Ghost, State=Disabled
```

实际 property names 尽可能匹配 code prop names：

| Figma Property | Code Prop 对应项 |
|---------------|---------------------|
| `Size` | `size` |
| `Style` / `Variant` | `variant` |
| `State` | 通常在 CSS 中由 `:hover`、`:focus`、`:disabled` 控制，但某些系统中使用 `state` |
| `Type` | `type` |
| `Disabled` | `disabled` (boolean) |
| `Icon` | `icon` (boolean or instance swap) |

### Property value 大小写

Property values 在 Figma 中使用 **Title Case**（便于在 Variants 面板中阅读），并映射到代码中的 lowercase：

| Figma value | Code value |
|-------------|-----------|
| `Small` | `"small"` / `"sm"` |
| `Medium` | `"medium"` / `"md"` |
| `Large` | `"large"` / `"lg"` |
| `Primary` | `"primary"` |
| `Disabled` | `disabled` (boolean prop) |
| `Default` | *（通常表示 absent/unset 情况）* |

### Boolean properties

Figma 中的 boolean component properties 使用 `true` / `false` 作为 values（Figma 原生 boolean），而不是 `Yes` / `No` 或 `On` / `Off`。

---

## 5. Style 命名（Text 和 Effect Styles）

### Text styles：category/name

```
Display/Large
Display/Medium
Display/Small
Heading/1
Heading/2
Heading/3
Body/Large
Body/Medium
Body/Small
Label/Large
Label/Small
Code/Inline
```

category 片段映射到 typography role。尽可能使用与 codebase typography scale 相同的 category names。

### Effect styles（shadows）：category/name

```
Shadow/None
Shadow/Subtle
Shadow/Medium
Shadow/Strong
Shadow/Overlay
Elevation/0
Elevation/1
Elevation/2
Elevation/3
Elevation/4
Elevation/5
```

对已命名的 semantic shadows 使用 `Shadow/`。对 Material Design 风格的编号 elevation levels 使用 `Elevation/N`。

---

## 6. Separator Pages

Separator pages 是空 pages，唯一用途是在 Figma page panel 中创建视觉分隔。这里有两种约定：

| 约定 | 示例 | 使用方 |
|------------|---------|---------|
| Three dashes | `---` | Simple DS, UI3, Polaris, Material 3 |
| Decorated text | `--- COMPONENTS ---` | Shop Minis |

three-dash 约定（`---`）最常见，也是新文件的默认值。除非目标文件使用 decorated-text 风格，否则使用它。

**separator 放置位置：**

```
Cover
---                    ← after cover
Foundations
Icons
---                    ← before components
[component pages]
---                    ← before utilities
Utilities
```

---

## 7. Status Indicators（UI3 Emoji System）

UI3 Library 在 page names 中使用彩色圆形 emoji，以便一眼传达 design readiness。该系统是可选的，但对大型团队很有用。

| Emoji | 含义 | 何时使用 |
|-------|---------|-------------|
| Green circle | Ready / Approved | Design 已稳定、已审查，可以安全使用 |
| Yellow circle | WIP / In Progress | Design 正在积极推进，可能变化 |
| Red circle | Do not use | 尚未 ready，不要引用；可能已 deprecated |

Code readiness 通过附加到 component name 的 brackets 表达：

| Bracket | 含义 |
|---------|---------|
| (none) | Component 已在代码中实现且稳定 |
| `[beta]` | Component 已在代码中实现，但尚未稳定（距离 ready 约 3 周） |
| `[future]` | 尚未在代码中实现 |

**Documentation status（位于 component pages 内）：**

如果构建 UI3-style system，每个 documentation frame 都会有一个 status banner，并使用以下标签之一：

- `APPROVED` - 已完整审查
- `READY FOR REVIEW` - 等待 sign-off
- `WORK IN PROGRESS` - 正在积极设计
- `NEEDS UPDATE` - 已过期，需要修订
- `DO NOT REFERENCE` - 不应使用

只有在生命周期跟踪能提供真实价值的大型多团队系统中，才推荐使用该系统。对于较小系统，跳过 emoji status indicators，使用 plain page names。

---

## 8. 何时匹配既有约定，何时使用默认值

**命名任何内容前始终先检查。** 在创建任何 pages 或 variables 之前，运行 `get_metadata` 或 `inspectFileStructure` 来发现既有约定。

### 以下情况匹配既有文件：

- 文件已有 pages，并且使用一致的命名模式（emoji 前缀、separator style、大小写）。
- 文件已有 variable collections，并且有既定命名方案。
- 文件由设计团队启动，并承载了有意做出的决策。
- 任何既有 component names 使用特定模式（PascalCase、kebab-case、namespace prefixes）。

### 以下情况使用本文档默认值：

- 从没有既有内容的全新 Figma 文件开始。
- 既有约定不一致（混用 styles = 没有可匹配的约定）。
- 用户明确要求创建遵循最佳实践的全新设计系统。

### 当 code 和 Figma 不一致时：

如果 codebase 使用 `button-primary`，但 Figma 中的 component 名为 `Button`，不要重命名 Figma component。改为：
- 保持 Figma name 为 `Button`（PascalCase，便于人读）。
- 设置 variable code syntax，使其匹配 codebase 中确切的 CSS token name。
- 设置 Code Connect source path 指向实际代码文件，并使用确切的 code component name。

**规则**：Figma names 面向设计师；code syntax 和 Code Connect source paths 承载确切的 code identifiers。这两个身份系统并行运行。

---

## 9. Figma Variable Names 与 Code Names：完整图景

这是最容易被误解的领域之一。Figma names 和 code names **有意遵循不同约定**，因为它们服务不同受众，并存在于不同环境中。

### 为什么它们不同

| | Figma variable name | Code syntax (WEB) |
|---|---|---|
| **受众** | Variables 面板中的 designers | CSS/Swift/Kotlin 中的 developers |
| **分隔符** | `/`（slash），在 Figma UI 中创建可视化分组 | `-`（hyphen），CSS custom property syntax 要求 |
| **大小写** | lowercase（或用于展示的 PascalCase，见下文） | CSS 使用 kebab-case；JS/Android 使用 camelCase |
| **深度** | 2-4 层 | CSS 扁平；JS 使用 dot-notation |
| **Namespace** | 隐式（按 collection） | 显式前缀（`--p-`、`--md-`、`--cds-`） |

### 转换方式

```
Figma variable name              Code syntax (WEB)
──────────────────               ─────────────────
color/bg/primary          →      var(--color-bg-primary)
spacing/xs                →      var(--spacing-xs)
radius/md                 →      var(--radius-md)
typography/body/font-size →      var(--typography-body-font-size)

Pattern: replace "/" with "-", wrap in var(--)

**CRITICAL: The `var()` wrapper is REQUIRED for WEB code syntax.** Figma expects the full CSS function syntax — not just the property name. If you set `--color-bg-primary` (without `var()`), Dev Mode will show raw hex values instead of the variable reference. Always set `var(--color-bg-primary)`.
```

```
Figma variable name              Code syntax (ANDROID)
──────────────────               ─────────────────────
color/bg/primary          →      colorBgPrimary
spacing/xs                →      spacingXs
radius/md                 →      radiusMd

Pattern: replace "/" with "", capitalize each word after first
```

```
Figma variable name              Code syntax (iOS)
──────────────────               ─────────────────
color/bg/primary          →      Color.bgPrimary
spacing/xs                →      Spacing.xs
radius/md                 →      Radius.md

Pattern: first segment becomes class name, remainder becomes property (camelCase)
```

### 来自 5 个参考文件的真实示例

| File | Figma variable name | WEB code syntax | ANDROID code syntax |
|------|--------------------|-----------------|--------------------|
| Simple DS | `color/bg/primary` | `var(--color-bg-primary)` | `colorBgPrimary` |
| Simple DS | `spacing/sm` | `var(--spacing-sm)` | `spacingSm` |
| Material 3 | `Schemes/Primary` | `var(--md-sys-color-primary)` | `colorPrimary` |
| Material 3 | `Corner/Extra-small` | `var(--md-sys-shape-corner-extra-small)` | `shapeCornerExtraSmall` |
| Polaris | `color/bg/surface` | `var(--p-color-bg-surface)` | none |

**来自 Material 3 的关键观察**：Figma name `Schemes/Primary` 使用带空格的 PascalCase，但 WEB code syntax 是 `var(--md-sys-color-primary)`，完全是带 vendor prefix `md-sys-` 的 kebab-case。Figma name 和 code syntax 几乎没有相似之处。这是有意设计的，并且在成熟设计系统中很常见。

### Figma 中的大小写：lowercase 是默认值，PascalCase 可用于展示

使用 lowercase 是默认指导，而不是普遍规则。真实文件中的证据：

| File | Figma case | Code output case | Why |
|------|-----------|------------------|-----|
| Simple DS | `color/bg/primary` (lowercase) | `var(--color-bg-primary)` | 直接映射，简单 |
| Material 3 | `Schemes/Primary` (PascalCase) | `var(--md-sys-color-primary)` | PascalCase 在 Variables 面板中更易读；code name 独立定义 |
| Polaris | `color/bg/surface` (lowercase) | `var(--p-color-bg-surface)` | 带 vendor prefix 的直接映射 |

**规则**：当 Figma name 会直接映射到 CSS name 时，使用 lowercase。当设计系统拥有与技术 code names 不同、便于人读的 variable names 时，使用 PascalCase（或匹配既有文件）。

### 当 codebase 不使用 CSS custom properties 时

某些 JavaScript-first 系统（Chakra、Ant Design、MUI）根本不使用 CSS `var(--...)`。它们的 tokens 位于 JS theme objects 中：

```
Chakra:    colors.gray[500]         →  JS: theme.colors.gray[500]
Ant:       colorPrimary             →  JS: token.colorPrimary
MUI:       palette.primary.main     →  JS: theme.palette.primary.main
```

在这些情况下，将 WEB code syntax 设置为 JS property path，而不是 CSS variable：
```javascript
// For a JS-object-based system like Chakra:
v.setVariableCodeSyntax('WEB', 'colors.gray.500');

// For Ant Design:
v.setVariableCodeSyntax('WEB', 'colorPrimary');
```

### 层级深度：匹配 codebase

slash 层级数量应镜像 codebase 的嵌套深度：

| Codebase pattern | Figma depth | Example |
|-----------------|------------|---------|
| `--primary` (flat) | 1-2 levels | `color/primary` |
| `--color-bg-surface` (3-part) | 3 levels | `color/bg/surface` |
| `--md-sys-color-primary` (vendor + 3-part) | 3 levels (vendor prefix goes in code syntax only) | `color/primary` |
| `theme.palette.primary.main` (4-part) | 3-4 levels | `color/palette/primary/main` |

**重要**：Vendor prefixes（`--p-`、`--md-sys-`、`--cds-`）属于 **code syntax**，不属于 Figma variable name。Figma name `color/bg/surface` + code syntax `var(--p-color-bg-surface)` 才是正确模式。

### Discovery 阶段的动作

在 Phase 0 discovery 期间，显式捕获映射两侧：

```
For each token found in the codebase:
  CSS variable:   --sds-color-background-brand-default
  Figma name:     color/bg/brand/default        (slash hierarchy, no vendor prefix)
  WEB syntax:     var(--sds-color-background-brand-default)  (exact CSS name)
  ANDROID syntax: sdsColorBackgroundBrandDefault  (camelCase)
  iOS syntax:     Color.backgroundBrandDefault    (dot-notation)
```

将此映射存入状态账本。在 Phase 1 调用 `setVariableCodeSyntax` 时使用它。如果已有原始 CSS variable name，绝不要从 Figma name 推导 code syntax，始终使用原始名称。
