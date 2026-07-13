---
name: figma-generate-library
description: "从代码库在 Figma 中构建或更新专业级设计系统。适用于创建变量或 token、组件库、主题、基础文档，或对齐代码与 Figma 差异时。此技能说明构建内容和顺序，应与 `figma-use` 一起加载。"
description_zh_CN: "从代码库在 Figma 中构建或更新专业级设计系统。适用于创建变量/token、组件库、主题、基础文档，或对齐代码与 Figma 差异时。此技能说明构建内容和顺序，应与 `figma-use` 一起加载。"
disable-model-invocation: false
---

# Design System Builder - Figma MCP Skill

本技能用于从代码库在 Figma 中构建专业级 design system。它编排跨 20 到 100 次以上 `use_figma` 调用的多阶段流程，强调真实 design system 的质量模式。`figma-use` 负责 Plugin API 细节，本技能负责构建内容和顺序。

每次 `use_figma` 调用都必须同时遵守 `figma-use`。调用时传入 `skillNames: "figma-generate-library"`，这只是日志参数。

## 1. 最重要的规则

这绝不是 one-shot 任务。构建设计系统需要跨多个阶段执行大量小步 `use_figma` 调用，并在阶段之间设置用户确认点。试图一次创建所有内容会产生损坏、不完整或难以恢复的结果。

## 2. 强制阶段顺序

不要跳过或重排阶段。

```text
Phase 0: DISCOVERY
  0a. 分析代码库，提取 tokens、components、命名约定
  0b. 检查 Figma 文件中的 pages、variables、components、styles 和既有约定
  0c. 用 search_design_system 搜索 subscribed libraries 中可复用资产
  0d. 锁定 v1 scope，明确 token set 和 component list
  0e. 对齐 code -> Figma，发现冲突时询问用户
  USER CHECKPOINT: 提交完整计划，等待明确批准

Phase 1: FOUNDATIONS
  1a. 创建 variable collections 和 modes
  1b. 创建 primitive variables
  1c. 创建 semantic variables，alias 到 primitives，支持 mode
  1d. 设置所有 variable scopes
  1e. 设置所有 variable code syntax
  1f. 创建 effect styles 和 text styles
  Exit: 计划中所有 token 存在，scope 和 syntax 均已设置
  USER CHECKPOINT

Phase 2: FILE STRUCTURE
  2a. 创建 page skeleton: Cover -> Getting Started -> Foundations -> --- -> Components -> --- -> Utilities
  2b. 创建 foundations documentation pages
  Exit: 页面完整，foundation docs 可浏览
  USER CHECKPOINT

Phase 3: COMPONENTS
  逐个 component 按依赖顺序构建，atoms 先于 molecules:
    3a. 创建专属 page
    3b. 用 auto-layout 和 variable binding 构建 base component
    3c. 创建所有 variant combinations，combineAsVariants 后手动网格布局
    3d. 添加 TEXT、BOOLEAN、INSTANCE_SWAP properties
    3e. 把 properties 连接到 child nodes
    3f. 添加页面文档
    3g. 用 get_metadata 和 get_screenshot 验证
    3h. 可选：上下文新鲜时创建轻量 Code Connect mapping
  每个 component 都需要 USER CHECKPOINT

Phase 4: INTEGRATION + QA
  4a. 完成 Code Connect mappings
  4b. 可访问性审计
  4c. 命名审计
  4d. 未解析 binding 审计
  4e. 每页最终截图
  USER CHECKPOINT
```

## 3. 关键规则

Plugin API 基础：

- 用 `return` 返回数据。不要包 IIFE，不要调用 `figma.closePlugin()`。
- 每次返回所有创建或修改的 node IDs。
- 每次调用页面上下文都会重置，开始时用 `await figma.setCurrentPageAsync(page)`。
- 不要用 `figma.notify()`。
- 颜色用 0 到 1，不是 0 到 255。
- 写 text 前必须 `await figma.loadFontAsync(...)`。
- `layoutSizingHorizontal/Vertical = "FILL"` 必须在 `parent.appendChild(child)` 之后设置。

Design system 规则：

1. Variables 先于 components。component 要绑定 variables。
2. 创建前先 inspect，匹配现有约定。
3. 默认每个 component 一个 page；紧密相关 family 可共用 page。
4. 默认把 fills、strokes、padding、radius、gap 等视觉属性绑定到 variables。
5. 每个 variable 都设置 scope，不要留 `ALL_SCOPES`。
6. 每个 variable 都设置 code syntax。WEB syntax 必须使用 `var(--name)` 包装。
7. semantic variables alias 到 primitives，不要重复 raw value。
8. `combineAsVariants` 后手动重新布局 variants。
9. icon 用 `INSTANCE_SWAP`，不要为每个 icon 建 variant。
10. 使用确定性命名，以便幂等清理和恢复。
11. 不做破坏性 cleanup，按 name convention 或 returned IDs 精确清理。
12. 验证后再继续。
13. 永远不要并行执行 `use_figma`。Figma 状态变更必须严格串行。
14. 不要猜 Node ID，只使用之前调用返回的 state ledger。
15. 优先复用 `scripts/` 中 helper script，不要从零写 200 行 inline script。
16. 每个 checkpoint 都要明确下一阶段；用户说 "looks good" 不等于批准跳到别的阶段。

## 4. 状态管理

`getPluginData()` 和 `setPluginData()` 在 `use_figma` 中不支持。使用 `getSharedPluginData()`、`setSharedPluginData()`，或通过返回 ID 维护 state ledger。

| Entity type | Idempotency key | 检查方式 |
| --- | --- | --- |
| Scene nodes | shared plugin data 或唯一名称 | `node.getSharedPluginData(...)` 或按 name 查找 |
| Variables | collection 内名称 | 在 local variables 中按 name 和 collection 查 |
| Styles | name | `getLocalTextStyles()` 等按 name 查 |

创建 scene node 后立即打标：

```javascript
node.setSharedPluginData('dsb', 'run_id', RUN_ID);
node.setSharedPluginData('dsb', 'phase', 'phase3');
node.setSharedPluginData('dsb', 'key', 'component/button');
```

不要只依赖会话上下文保存 ledger。写入磁盘：

```text
/tmp/dsb-state-{RUN_ID}.json
```

每轮开始重读该文件。长流程中上下文可能被截断，文件是 source of truth。

## 5. search_design_system 复用决策

Phase 0 先搜索，每个 component 创建前再搜索：

```text
search_design_system({ query, fileKey, includeComponents: true, includeVariables: true, includeStyles: true })
```

复用条件：

- component property API 满足需求。
- token binding model 兼容。
- 命名约定匹配目标文件。
- component 可编辑或可被正确引用。

重建条件：

- API 不兼容。
- token model 不兼容。
- ownership 导致无法修改。

视觉匹配但 API 不兼容时，可以把 library component 包在新的 wrapper component 中，并暴露清晰 API。

优先级：本地已有 -> subscribed library import -> 新建。

## 6. 用户确认点

必须设置。设计决策需要人类判断。

| 阶段 | 提供内容 | 询问 |
| --- | --- | --- |
| Discovery + scope lock | token list、component list、gap analysis | 是否批准开始创建 |
| Foundations | variable summary、style list | 是否进入文件结构 |
| File structure | page list + screenshot | 是否进入 components |
| 每个 component | component page screenshot | variant 和视觉是否正确 |
| 每个冲突 | 展示 code 和 Figma 差异 | 哪一方为准 |
| Final QA | 每页截图和 audit report | 是否 sign off |

用户拒绝时，先修复再继续。

## 7. 命名和 token 架构

匹配现有文件约定。全新项目可使用：

```text
color/bg/primary
color/text/secondary
color/border/default
spacing/xs
spacing/sm
spacing/md
radius/sm
radius/md
typography/body/font-size
```

组件名如 `Button`、`Input`、`Card`、`Avatar`。variant 名如 `Size=Medium, Style=Primary, State=Default`。

复杂度建议：

- 少于 50 tokens：单 collection，Light/Dark 两个 mode。
- 50 到 200 tokens：Primitives + Color semantic + Spacing + Typography。
- 超过 200 tokens：多 semantic collection 和多 mode。

## 8. 反模式

- Phase 0 前不要创建任何内容。
- 不要无视现有文件约定。
- 不要跳过 `search_design_system`。
- 不要使用 `ALL_SCOPES`。
- 不要在 semantic layer 重复 raw value。
- 不要省略 code syntax。
- 不要在 token taxonomy 未确认前创建 component token。
- 不要并行运行 `use_figma`。
- 不要基于未验证结果继续构建。
