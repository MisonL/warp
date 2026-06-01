> 属于 [figma-generate-library skill](../SKILL.md) 的一部分。

# Code Connect Setup Reference

此 reference 覆盖 figma-generate-library agent 可用的全部 Code Connect tooling：`add_code_connect_map` tool、用于验证的 `get_code_connect_map`、用于批量应用的 `send_code_connect_mappings`、variable code syntax、framework label，以及何时按 component 映射、何时在最终 pass 中映射的决策。

---

## 1. Code Connect 做什么

Code Connect 会将 Figma component node 链接到其代码实现，从而：

- **Dev Mode** 在开发者检查 component 时显示真实代码片段（来自你的代码库），而不是 auto-generated approximation。
- **MCP `get_design_context`** 与 design token 一起返回 `componentName`、`source` 和 rendered snippet，从而支持准确的 AI-assisted code generation。
- **`search_design_system`** 可以在返回 Figma component metadata 的同时返回 code reference。

---

## 2. 三个 MCP Tool

### 2a. add_code_connect_map - 单个 mapping

将一个 Figma node 映射到一个代码 component。

**参数：**

| 参数 | 类型 | 是否必需 | 说明 |
|-----------|------|----------|-------|
| `nodeId` | string | 是（remote）/ 可选（desktop） | Format `123:456`。必须是已发布的 component 或 component set。 |
| `fileKey` | string | 是（remote） | Figma file key。 |
| `source` | string | 是 | 代码库中的路径（例如 `src/components/Button.tsx`）或 URL。 |
| `componentName` | string | 是 | 代码 component 名称（例如 `Button`）。 |
| `label` | enum | 是 | Framework label，合法值见第 4 节。 |
| `template` | string | 可选 | 可执行 JS template code。提供此项会创建 **figmadoc**（template）mapping，而不是简单的 **component_browser** mapping。需要 `pixie_mcp_enable_writing_code_connect_templates` feature flag。 |
| `templateDataJson` | string | 可选 | 包含可选字段的 JSON string：`isParserless`、`imports`、`nestable`、`props`。 |

**两层 mapping：**

1. **Simple mapping（component_browser）：** 只提供 `source`、`componentName` 和 `label`。将 Figma component 关联到 code path + name。Dev Mode 会根据 Figma prop name 生成基础 JSX snippet。这是默认方式，应优先使用。

2. **Template mapping（figmadoc）：** 额外提供 `template`。template 在 sandboxed QuickJS environment 中执行，并根据实际 instance property value 动态渲染 snippet。只有当用户需要精确的 prop-level Code Connect 时才使用。

**常见错误码：**

| Error | 含义 | 修复 |
|-------|---------|-----|
| `CODE_CONNECT_MAPPING_ALREADY_EXISTS` | Component 已经映射 | 先在 Figma UI 中断开现有 mapping |
| `CODE_CONNECT_ASSET_NOT_FOUND` | 找不到已发布 component | 确保 component 已发布到 library |
| `CODE_CONNECT_INSUFFICIENT_PERMISSIONS` | 没有 edit access | 请求该 file 的 edit permission |
| `CODE_CONNECT_NO_LIBRARY_FOUND` | File 未发布为 library | 先将 file 发布为 Figma library |

**用法示例：**

```
Tool: add_code_connect_map
Args: {
  nodeId: "123:456",
  fileKey: "abc123",
  source: "src/components/Button.tsx",
  componentName: "Button",
  label: "React"
}
```

---

### 2b. get_code_connect_map - 验证

获取某个 node 当前的 Code Connect mapping。在 `add_code_connect_map` 后立即使用它确认 mapping 已保存；在 `send_code_connect_mappings` 前也可用来审计现有状态。

**参数：**

| 参数 | 类型 | 是否必需 | 说明 |
|-----------|------|----------|-------|
| `nodeId` | string | 可选 | 要检查的 node。省略时获取 file 中所有 mapping。 |
| `fileKey` | string | 是（remote） | Figma file key。 |
| `codeConnectLabel` | string | 可选 | 将结果过滤到指定 framework label。 |

**返回：** `nodeId -> { componentName, source, label, snippet, snippetImports }` 的 map。

**验证方式：**

```
1. Call add_code_connect_map with the node.
2. Immediately call get_code_connect_map(nodeId, fileKey).
3. Confirm the returned object has the expected componentName and source.
4. If the mapping is missing, check for error codes from step 1.
```

---

### 2c. send_code_connect_mappings - 批量应用

一次调用应用多个 Code Connect mapping。在 `get_code_connect_suggestions` 返回一批未映射 component 后使用，或在 Phase 4 结束时做 final-pass bulk mapping。

**参数：**

| 参数 | 类型 | 是否必需 | 说明 |
|-----------|------|----------|-------|
| `nodeId` | string | 可选 | mappings array 为空时用于 design fallback 的 context node。 |
| `fileKey` | string | 是（remote） | Figma file key。 |
| `mappings` | array | 是 | Mapping object array。 |

**每个 mapping object：**

| 字段 | 类型 | 是否必需 | 说明 |
|-------|------|----------|-------|
| `nodeId` | string | 是 | Figma node identifier。 |
| `componentName` | string | 是 | Code component name。 |
| `source` | string | 是 | 代码库中的路径。 |
| `label` | enum | 是 | Framework label。 |
| `template` | string | 可选 | 用于 figmadoc mapping 的 JS template code。 |
| `templateDataJson` | string | 可选 | JSON template metadata。 |

**行为：**

- 所有 mapping 都会通过 POST 并行发送到 backend 处理。
- 如果某个 mapping 失败，会按 mapping 报告错误，其余 mapping 仍会成功。
- 如果完全成功，会对这些 node 调用 `get_design_context` 并返回 fresh design context。

**批量工作流：**

```
1. Collect all {nodeId, componentName, source, label} pairs.
2. Call send_code_connect_mappings({ fileKey, mappings: [...all pairs...] }).
3. Review reported errors and call add_code_connect_map individually for any failures.
4. Call get_code_connect_map on a sample of nodes to spot-check.
```

---

## 3. Variable Code Syntax（Token Round-Tripping）

在 variable 上设置 code syntax，会在 Figma token 与代码库 token system 之间建立双向链接。这样 Dev Mode 可以在 design value 旁边显示 `var(--color-bg-primary)`，而不是 raw hex。

**三个 platform：**

```javascript
// In use_figma:
variable.setVariableCodeSyntax('WEB', 'var(--color-bg-primary)');
variable.setVariableCodeSyntax('ANDROID', 'Theme.colorBgPrimary');
variable.setVariableCodeSyntax('iOS', 'Color.bgPrimary');
```

- `WEB` - 用于 CSS custom property、design token JSON 和任何 web framework。
- `ANDROID` - 用于 Jetpack Compose theme reference 和 Android resource name。
- `iOS` - 用于 SwiftUI Color extension 和 UIKit color method。

**推导规则（按优先级）：**

1. **最佳：** 使用代码库中的精确 token name。搜索代码库中的 CSS custom property（`--`）、Swift color extension 或 Kotlin theme reference，并使用这些精确字符串。
2. **可接受：** 从 Figma variable name 以一致转换方式推导：将 `/` 和空格替换为 `-`，加上 `var(--` 前缀和 `)` 后缀。
   - 示例：`color/bg/primary` -> `var(--color-bg-primary)`
3. **避免：** 猜测或发明代码库中不存在的名称。

**一致性规则：** 转换必须统一。如果对一个 variable 使用 `var(--color-bg-primary)`，同一 collection 中的所有 variable 都应使用相同的 `var(--{path-with-hyphens})` pattern。

**WEB syntax 批量示例：**

```javascript
// In use_figma — set WEB code syntax on all variables in a collection
const collections = await figma.variables.getLocalVariableCollectionsAsync();
for (const coll of collections) {
  if (coll.name !== 'Color') continue;
  for (const varId of coll.variableIds) {
    const v = await figma.variables.getVariableByIdAsync(varId);
    if (!v) continue;
    // Derive: "color/bg/primary" → "var(--color-bg-primary)"
    const cssName = 'var(--' + v.name.toLowerCase().replace(/\//g, '-').replace(/\s+/g, '-') + ')';
    v.setVariableCodeSyntax('WEB', cssName);
  }
}
```

---

## 4. Framework Labels

以下 label 对所有 Code Connect MCP operation 都有效。使用与代码库 framework 匹配的 label。

| Label | 适用场景 |
|-------|---------|
| `React` | React / JSX / TSX components |
| `Web Components` | Native Web Components, Lit, FAST |
| `Vue` | Vue 2 and Vue 3 SFCs |
| `Svelte` | Svelte components |
| `Storybook` | Storybook stories with Code Connect integration |
| `Javascript` | Plain JavaScript, framework-agnostic |
| `Swift` | Swift / UIKit |
| `Swift UIKit` | UIKit specifically |
| `Objective-C UIKit` | Objective-C with UIKit |
| `SwiftUI` | SwiftUI view components |
| `Compose` | Jetpack Compose (Android) |
| `Java` | Java Android components |
| `Kotlin` | Kotlin Android (non-Compose) |
| `Android XML Layout` | Android XML layout files |
| `Flutter` | Flutter / Dart widgets |
| `Markdown` | Documentation or MDX components |

**HTML note：** `HTML` label 由 Code Connect CLI 的 HTML parser 使用（用于 Angular、Vue 以及没有 framework-specific parser 的 Web Components），但 MCP tools 直接使用 `Web Components` 或 `Vue`。选择前先检查代码库 framework。

---

## 5. Per-Component 与 Final-Pass 策略

### Per-component（新构建时首选）

创建 component 后立即映射 Code Connect，此时上下文最新鲜（SKILL.md workflow 的 Phase 3，step 3h）：

**优点：**
- node ID 已从创建 script 中获得。
- 你清楚知道此 Figma component 对应哪个代码 component，因为刚刚按它设计。
- 错误会提前暴露，不会拖到构建 dependent component 之后。

**何时使用：** 任何创建了与现有代码 component 有清晰 1:1 匹配的 Figma component 时。

### Final pass（Phase 4 的 bulk mapping）

收集所有未映射 component，并在一次 `send_code_connect_mappings` 调用中映射：

**优点：**
- 一次 bulk call，而不是 N 次单独调用。
- 可以用 `get_code_connect_suggestions` 自动发现未映射 component。
- 更适合导入你没有控制创建过程的现有 Figma file。

**何时使用：** 为现有 file 补做 Code Connect，或代码库 mapping 需要调研且更适合在所有 component 创建完后统一处理时。

### Hybrid（大型系统推荐）

- 在 Phase 3 中按 **per-component** 映射 atoms（Button、Input、Badge、Avatar）。
- 所有 atoms 映射后，在 Phase 4 中用 **final pass** 映射 molecules 和 organisms，因为 molecule snippet 会引用 atom Code Connect ID。

---

## 6. 在 Dev Mode 中验证

映射后：

1. 在 browser 或 desktop app 中打开 Figma file。
2. 切换到 Dev Mode（toolbar 中的 `</>` icon）。
3. 选择 component instance（不是 main component，而是放在 page 上的 instance）。
4. 在 Inspect panel 中，code snippet 应显示 Code Connect output，而不是 auto-generated code。
5. 如果 snippet 缺失或显示 `[auto-generated]`，通过 MCP 运行 `get_code_connect_map` 确认 mapping 存在，然后检查 component 是否已发布。

**通过 MCP（agent workflow 中更快）：**

```
get_code_connect_map(nodeId: "<the component set node ID>", fileKey: "<file key>")
```

响应应包含 `componentName`、`source`、`label` 和非空 `snippet`。

---

## 7. 重要约束

- **仅限已发布 component：** `add_code_connect_map` 要求 component 已发布到 library。如果 file 尚未发布，mapping 会以 `CODE_CONNECT_NO_LIBRARY_FOUND` 失败。
- **每个 node 的每个 label 只有一个 mapping：** 一个 node 可以有多个 mapping（每个 framework label 一个），但同一 label 只能有一个。向同一 node 添加第二个 React mapping 会返回 `CODE_CONNECT_MAPPING_ALREADY_EXISTS`。
- **Template mapping 受 gate 控制：** `template` 参数需要 `pixie_mcp_enable_writing_code_connect_templates` feature flag。除非用户明确要求 template-level Code Connect，否则使用 simple mapping。
- **从简单开始，再升级：** 始终从 simple mapping（`source` + `componentName` + `label`）开始。只有当用户需要精确的 prop-level snippet rendering 时，才添加 `template`。
