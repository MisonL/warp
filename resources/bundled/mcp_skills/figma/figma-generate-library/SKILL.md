---
name: figma-generate-library
description: "根据代码库在 Figma 中构建或更新 professional-grade design system。当用户想创建 variable/token、构建 component library、设置 theming（light/dark mode）、编写 foundation 文档，或调和代码与 Figma 之间的差异时使用。此 skill 说明要构建什么以及按什么顺序构建，并与说明如何调用 Plugin API 的 `figma-use` skill 互补。两个 skill 应一起加载。"
disable-model-invocation: false
---

# Design System Builder - Figma MCP Skill

在 Figma 中构建与代码匹配的 professional-grade design system。此 skill 会编排横跨 20-100+ 次 `use_figma` 调用的多阶段工作流，并强制执行来自真实 design system（Material 3、Polaris、Figma UI3、Simple DS）的质量 pattern。

**前置条件**：每次 `use_figma` 调用都必须同时加载 `figma-use` skill。它提供 Plugin API 语法规则（return pattern、page reset、ID return、font loading、color range）。此 skill 提供 design system 领域知识和工作流编排。

**作为此 skill 的一部分调用 `use_figma` 时，始终传入 `skillNames: "figma-generate-library"`。** 这是 logging 参数，不影响执行。

---

## 1. 最重要的一条规则

**这绝不是 one-shot task。** 构建 design system 需要跨多个阶段进行 20-100+ 次 `use_figma` 调用，并且阶段之间必须设置用户检查点。任何试图在一次调用中创建所有内容的做法，都会产生破损、不完整或无法恢复的结果。把每个操作拆到最小有用单元，验证，获取反馈，再继续。

---

## 2. 必需工作流

每次 design system 构建都遵循此阶段顺序。跳过或重排阶段会导致代价很高的结构性失败。

```
Phase 0: DISCOVERY (always first — no use_figma writes yet)
  0a. Analyze codebase → extract tokens, components, naming conventions
  0b. Inspect Figma file → pages, variables, components, styles, existing conventions
  0c. Search subscribed libraries → use search_design_system for reusable assets
  0d. Lock v1 scope → agree on exact token set + component list before any creation
  0e. Map code → Figma → resolve conflicts (code and Figma disagree = ask user)
  ✋ USER CHECKPOINT: present full plan, await explicit approval

Phase 1: FOUNDATIONS (tokens first — always before components)
  1a. Create variable collections and modes
  1b. Create primitive variables (raw values, 1 mode)
  1c. Create semantic variables (aliased to primitives, mode-aware)
  1d. Set scopes on ALL variables
  1e. Set code syntax on ALL variables
  1f. Create effect styles (shadows) and text styles (typography)
  → Exit criteria: every token from the agreed plan exists, all scopes set, all code syntax set
  ✋ USER CHECKPOINT: show variable summary, await approval

Phase 2: FILE STRUCTURE (before components)
  2a. Create page skeleton: Cover → Getting Started → Foundations → --- → Components → --- → Utilities
  2b. Create foundations documentation pages (color swatches, type specimens, spacing bars)
  → Exit criteria: all planned pages exist, foundations docs are navigable
  ✋ USER CHECKPOINT: show page list + screenshot, await approval

Phase 3: COMPONENTS (one at a time — never batch)
  For EACH component (in dependency order: atoms before molecules):
    3a. Create dedicated page
    3b. Build base component with auto-layout + full variable bindings
    3c. Create all variant combinations (combineAsVariants + grid layout)
    3d. Add component properties (TEXT, BOOLEAN, INSTANCE_SWAP)
    3e. Link properties to child nodes
    3f. Add page documentation (title, description, usage notes)
    3g. Validate: get_metadata (structure) + get_screenshot (visual)
    3h. Optional: lightweight Code Connect mapping while context is fresh
    → Exit criteria: variant count correct, all bindings verified, screenshot looks right
    ✋ USER CHECKPOINT per component: show screenshot, await approval before next component

Phase 4: INTEGRATION + QA (final pass)
  4a. Finalize all Code Connect mappings
  4b. Accessibility audit (contrast, min touch targets, focus visibility)
  4c. Naming audit (no duplicates, no unnamed nodes, consistent casing)
  4d. Unresolved bindings audit (no hardcoded fills/strokes remaining)
  4e. Final review screenshots of every page
  ✋ USER CHECKPOINT: complete sign-off
```

---

## 3. 关键规则

**Plugin API 基础**（来自 use_figma skill，此处同样强制执行）：
- 使用 `return` 返回数据（自动序列化）。不要包在 IIFE 中，也不要调用 closePlugin。
- 每个 return value 都返回所有 created/mutated node ID。
- 每次调用都会重置 page context，开头始终执行 `await figma.setCurrentPageAsync(page)`。
- `figma.notify()` 会 throw，永远不要使用。
- Color 使用 0-1 range，不是 0-255。
- 写入任何文本前必须加载 font：`await figma.loadFontAsync({family, style})`

**Design system rules**：
1. **Variable 在 component 之前**：component 绑定到 variable。没有 token 就没有 component。
2. **创建前先检查**：运行 read-only `use_figma` 发现现有约定，并匹配这些约定。
3. **默认每个 component 一个 page**：例外是强相关 family（例如 Input + helper）可以共用一个 page，但 section 必须清晰分隔。
4. **默认将 visual property 绑定到 variable**：fill、stroke、padding、radius、gap。例外是有意固定的几何值（icon pixel-grid size、static divider）。
5. **每个 variable 都设置 scope**：绝不要保留 `ALL_SCOPES`。Background：`FRAME_FILL, SHAPE_FILL`。Text：`TEXT_FILL`。Border：`STROKE_COLOR`。Spacing：`GAP`。Radii：`CORNER_RADIUS`。Primitive：`[]`（hidden）。
6. **每个 variable 都设置 code syntax**：WEB syntax 必须使用 `var()` wrapper：`var(--color-bg-primary)`，而不是 `--color-bg-primary`。使用代码库中的实际 CSS variable name。ANDROID/iOS 不使用 wrapper。
7. **将 semantic alias 到 primitive**：`{ type: 'VARIABLE_ALIAS', id: primitiveVar.id }`。不要在 semantic layer 中重复 raw value。
8. **combineAsVariants 之后再放置 variant**：它们会堆叠在 (0,0)。需要手动 grid-layout 并 resize。
9. **Icon 使用 INSTANCE_SWAP**：不要为每个 icon 创建 variant。限制 variant matrix：如果 Size x Style x State 超过 30 个组合，拆成 sub-component。
10. **Deterministic naming**：使用一致且唯一的 node name，方便 idempotent cleanup 和 resume。通过 return value 和 state ledger 跟踪 created node ID。
11. **No destructive cleanup**：cleanup script 通过 name convention 或 returned ID 识别 node，不靠猜测。
12. **继续前先验证**：绝不要基于未验证的工作继续构建。每次 create 后执行 `get_metadata`，每个 component 后执行 `get_screenshot`。
13. **绝不要并行化 `use_figma` 调用**：Figma state mutation 必须严格顺序执行。即使工具支持并行调用，也不要同时运行两个 use_figma 调用。
14. **绝不要 hallucinate Node ID**：始终从 previous call 返回的 state ledger 读取 ID。不要凭记忆重建或猜 ID。
15. **使用 helper script**：把 `scripts/` 中的 script 嵌入到 use_figma 调用中。不要从头写 200 行 inline script。
16. **显式 phase approval**：每个 checkpoint 都明确命名下一阶段。如果你询问的是 Phase 1，用户说 "looks good" 并不等于批准进入 Phase 3。

---

## 4. State Management（长工作流必需）

> **`getPluginData()` / `setPluginData()` 在 `use_figma` 中不受支持。** 改用 `getSharedPluginData()` / `setSharedPluginData()`（这些受支持），或使用 name-based lookup 和 state ledger（returned IDs）。

| Entity type | Idempotency key | 如何检查是否存在 |
|-------------|----------------|----------------------|
| Scene nodes（pages、frames、components） | `setSharedPluginData('dsb', 'key', value)` 或 unique name | `node.getSharedPluginData('dsb', 'key')` 或 `page.findOne(n => n.name === 'Button')` |
| Variables | Collection 内的 name | `(await figma.variables.getLocalVariablesAsync()).find(v => v.name === name && v.variableCollectionId === collId)` |
| Styles | Name | `getLocalTextStyles().find(s => s.name === name)` |

每个创建的 **scene node** 都要在创建后立即打 tag：
```javascript
node.setSharedPluginData('dsb', 'run_id', RUN_ID);        // identifies this build run
node.setSharedPluginData('dsb', 'phase', 'phase3');        // which phase created it
node.setSharedPluginData('dsb', 'key', 'component/button');// unique logical key
```

**State persistence**：不要只依赖 conversation context 保存 state ledger。将其写入磁盘：
```
/tmp/dsb-state-{RUN_ID}.json
```
每个 turn 开始时重新读取此文件。长工作流中 conversation context 会被截断，此文件是 source of truth。

维护 state ledger 来跟踪：
```json
{
  "runId": "ds-build-2024-001",
  "phase": "phase3",
  "step": "component-button",
  "entities": {
    "collections": { "primitives": "id:...", "color": "id:..." },
    "variables": { "color/bg/primary": "id:...", "spacing/sm": "id:..." },
    "pages": { "Cover": "id:...", "Button": "id:..." },
    "components": { "Button": "id:..." }
  },
  "pendingValidations": ["Button:screenshot"],
  "completedSteps": ["phase0", "phase1", "phase2", "component-avatar"]
}
```

**Idempotency check**：每次创建前，按 name + state ledger ID 查询。如果已存在，跳过或更新，绝不重复创建。

**Resume protocol**：session 开始或 context truncation 后，运行 read-only `use_figma`，按 name 扫描所有 page、component、variable 和 style，重建 `{key -> id}` map。然后在可用时重新读取磁盘中的 state file。

**Continuation prompt**（在新 chat 中 resume 时给用户）：
> "I'm continuing a design system build. Run ID: {RUN_ID}. Load the figma-generate-library skill and resume from the last completed step."

---

## 5. search_design_system - Reuse Decision Matrix

先在 Phase 0 搜索，然后在每次 component 创建前再次搜索。

```
search_design_system({ query, fileKey, includeComponents: true, includeVariables: true, includeStyles: true })
```

**满足以下全部条件时复用**：
- Component property API 满足需求（相同 variant axes，类型兼容）
- Token binding model 兼容（使用相同或可 alias 的 variable）
- Naming convention 匹配目标文件
- Component 可编辑（不是你不拥有且锁定在 remote library 中的内容）

**满足以下任一条件时重建**：
- API 不兼容（property name 不同，variant model 错误）
- Token model 不兼容（hardcoded value、不同 variable schema）
- Ownership issue（无法修改 library）

**视觉匹配但 API 不兼容时 wrap**：
- 将 library component 作为 nested instance 导入到新的 wrapper component 内
- 在 wrapper 上暴露干净 API

**三路优先级**：local existing -> subscribed library import -> create new。

---

## 6. 用户检查点

强制要求。Design decision 需要人工判断。

| 之后 | 必需产物 | 询问 |
|-------|-------------------|-----|
| Discovery + scope lock | Token list、component list、gap analysis | "Here's my plan. Approve before I create anything?" |
| Foundations | Variable summary（N collections、M vars、K modes）、style list | "All tokens created. Review before file structure?" |
| File structure | Page list + screenshot | "Pages set up. Review before components?" |
| Each component | component page 的 get_screenshot | "Here's [Component] with N variants. Correct?" |
| Each conflict（code != Figma） | 展示两个版本 | "Code says X, Figma has Y. Which wins?" |
| Final QA | Per-page screenshots + audit report | "Complete. Sign off?" |

**如果用户拒绝**：先修复再继续。绝不要基于被拒绝的工作继续构建。

---

## 7. Naming Conventions

匹配现有 file convention。如果从零开始：

**Variables**（slash-separated）：
```
color/bg/primary     color/text/secondary    color/border/default
spacing/xs  spacing/sm  spacing/md  spacing/lg  spacing/xl  spacing/2xl
radius/none  radius/sm  radius/md  radius/lg  radius/full
typography/body/font-size    typography/heading/line-height
```

**Primitives**：`blue/50` -> `blue/900`，`gray/50` -> `gray/900`

**Component names**：`Button`、`Input`、`Card`、`Avatar`、`Badge`、`Checkbox`、`Toggle`

**Variant names**：`Property=Value, Property=Value`，例如 `Size=Medium, Style=Primary, State=Default`

**Page separators**：`---`（最常见）或 `——— COMPONENTS ———`

> 完整命名参考：[naming-conventions.md](references/naming-conventions.md)

---

## 8. Token Architecture

| Complexity | Pattern |
|-----------|---------|
| < 50 tokens | Single collection, 2 modes（Light/Dark） |
| 50-200 tokens | **Standard**：Primitives（1 mode）+ Color semantic（Light/Dark）+ Spacing（1 mode）+ Typography（1 mode） |
| 200+ tokens | **Advanced**：Multiple semantic collections, 4-8 modes（Light/Dark x Contrast x Brand）。参见 [token-creation.md](references/token-creation.md) 中的 M3 pattern |

Standard pattern（推荐起点）：
```
Collection: "Primitives"    modes: ["Value"]
  blue/500 = #3B82F6, gray/900 = #111827, ...

Collection: "Color"         modes: ["Light", "Dark"]
  color/bg/primary → Light: alias Primitives/white, Dark: alias Primitives/gray-900
  color/text/primary → Light: alias Primitives/gray-900, Dark: alias Primitives/white

Collection: "Spacing"       modes: ["Value"]
  spacing/xs = 4, spacing/sm = 8, spacing/md = 16, ...
```

---

## 9. Per-Phase Anti-Patterns

**Phase 0 anti-patterns：**
- 在与用户锁定 scope 前开始创建任何内容
- 忽略现有 file convention 并强加新 convention
- 在规划 component creation 前跳过 `search_design_system`

**Phase 1 anti-patterns：**
- 对任何 variable 使用 `ALL_SCOPES`
- 在 semantic layer 中复制 raw value，而不是 alias
- 不设置 code syntax（会破坏 Dev Mode 和 round-tripping）
- 在商定 token taxonomy 前创建 component token

**Phase 2 anti-patterns：**
- 跳过 cover page 或 foundations docs
- 将多个无关 component 放在一个 page 上

**Phase 3 anti-patterns：**
- 在 foundation 存在前创建 component
- 在 component 中 hardcode 任何 fill/stroke/spacing/radius value
- 为每个 icon 创建 variant（改用 INSTANCE_SWAP）
- combineAsVariants 后不放置 variant（它们都会堆在 0,0）
- 构建超过 30 个组合的 variant matrix 且不拆分（variant explosion）
- 导入 remote component 后立即 detach

**General anti-patterns：**
- 未先理解错误就重试失败 script
- 使用 name-prefix matching 做 cleanup（会删除用户拥有的 node）
- 基于上一步未验证的工作继续构建
- 为了 "save time" 跳过用户检查点
- 并行化 use_figma 调用（始终顺序执行）
- 凭记忆猜测或 hallucinate node ID（始终从 state ledger 读取）
- 编写巨大的 inline script，而不是使用提供的 helper script
- 用户说 "build the button" 后，在未完成 Phase 0-2 的情况下开始 Phase 3

---

## 10. Reference Docs

按需加载。每个 reference 都是对应 phase 的权威依据：

需要时使用文件读取工具读取这些 docs。不要根据文件名假设其内容。

| Doc | Phase | Required / Optional | Load when |
|-----|-------|---------------------|-----------|
| [discovery-phase.md](references/discovery-phase.md) | 0 | **Required** | Starting any build - codebase analysis + Figma inspection |
| [token-creation.md](references/token-creation.md) | 1 | **Required** | Creating variables, collections, modes, styles |
| [documentation-creation.md](references/documentation-creation.md) | 2 | Required | Creating cover page, foundations docs, swatches |
| [component-creation.md](references/component-creation.md) | 3 | **Required** | Creating any component or variant |
| [code-connect-setup.md](references/code-connect-setup.md) | 3-4 | Required | Setting up Code Connect or variable code syntax |
| [naming-conventions.md](references/naming-conventions.md) | Any | Optional | Naming anything - variables, pages, variants, styles |
| [error-recovery.md](references/error-recovery.md) | Any | **Required on error** | Script fails, multi-step workflow recovery, cleanup of abandoned workflow state |

---

## 11. Scripts

可复用 Plugin API helper functions。嵌入到 `use_figma` 调用中：

| Script | Purpose |
|--------|---------|
| [inspectFileStructure.js](scripts/inspectFileStructure.js) | Discover all pages, components, variables, styles; returns full inventory |
| [createVariableCollection.js](scripts/createVariableCollection.js) | Create a named collection with modes; returns `{collectionId, modeIds}` |
| [createSemanticTokens.js](scripts/createSemanticTokens.js) | Create aliased semantic variables from a token map |
| [createComponentWithVariants.js](scripts/createComponentWithVariants.js) | Build a component set from a variant matrix; handles grid layout |
| [bindVariablesToComponent.js](scripts/bindVariablesToComponent.js) | Bind design tokens to all component visual properties |
| [createDocumentationPage.js](scripts/createDocumentationPage.js) | Create a page with title + description + section structure |
| [validateCreation.js](scripts/validateCreation.js) | Verify created nodes match expected counts, names, structure |
| [cleanupOrphans.js](scripts/cleanupOrphans.js) | Remove orphaned nodes by name convention or state ledger IDs |
| [rehydrateState.js](scripts/rehydrateState.js) | Scan file for all pages, components, variables by name; returns full `{key -> nodeId}` map for state reconstruction |
