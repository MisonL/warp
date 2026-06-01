> 属于 [figma-generate-library skill](../SKILL.md) 的一部分。

# 错误恢复参考

用于在包含 20-100+ 次调用的设计系统构建中处理失败和未完成运行的协议。

---

## 1. 核心协议：STOP / Inspect / Fix / Retry

**`use_figma` 是原子的，失败的脚本不会执行。** 如果脚本报错，文件不会发生任何更改。失败调用本身不会留下部分节点或半构建状态。修复后重试是安全的。

但是，在多步骤工作流中（20-100+ 次调用），**此前成功的调用**会创建并保留状态。如果工作流中途放弃，早先成功调用创建的节点会留在文件中。本文档中的清理和幂等模式用于处理这种场景。

失败脚本的恢复顺序：

```
1. STOP    — Do not run any more use_figma writes.
2. INSPECT — Read the error message carefully. Optionally call get_metadata or get_screenshot to understand the current file state.
3. FIX     — Correct the script that failed.
4. RETRY   — Re-run the corrected script.
5. PERSIST — Update the state ledger with the outcome.
```

对于**已放弃的多步骤工作流**（需要回滚此前*成功*调用创建的节点），使用第 2 节中的清理协议。

---

## 2. 基于 `sharedPluginData` 的清理：为什么名称匹配很危险

### 为什么名称前缀匹配会失败

如果清理脚本删除“所有名称以 `Button` 开头的节点”，也会删除用户可能手动创建的同名节点，或来自先前已批准阶段的节点。基于名称的清理无法区分“失败尝试留下的孤立节点”和“用户有意创建的节点”。

此外，变体名称（`Size=Medium, Style=Primary, State=Default`）没有一致且可安全定位的前缀，容易同时命中合法节点。

### `setSharedPluginData` / `getSharedPluginData` 的工作方式

`sharedPluginData` 是附加在单个节点上的 key-value 存储。它会跨会话保留，并且在 Figma UI 中对用户不可见。数据按 namespace 隔离，这里使用 `'dsb'`。使用三个 key：

```javascript
node.setSharedPluginData('dsb', 'run_id', 'ds-build-2024-001'); // identifies the build run
node.setSharedPluginData('dsb', 'phase',  'phase3');             // which phase created this node
node.setSharedPluginData('dsb', 'key',    'componentset/button');// unique logical key

// Reading:
const runId = node.getSharedPluginData('dsb', 'run_id'); // returns '' if never set
const key   = node.getSharedPluginData('dsb', 'key');
```

对于未设置的 key，`getSharedPluginData` 返回 `''`（空字符串，不是 null）。始终使用 `!== ''` 检查。

**每个节点创建后立即打标**。如果之后放弃多步骤工作流，这可以启用安全清理。打标应放在创建之后的同一语句序列中：

```javascript
const comp = figma.createComponent();
comp.setSharedPluginData('dsb', 'run_id', RUN_ID);  // tag immediately
comp.setSharedPluginData('dsb', 'key', key);         // tag immediately
// ... then do the rest of the setup
```

### 使用 `run_id` 的完整 `cleanupOrphans` 脚本

此脚本查找所有标记了指定 `run_id` 的节点，并可选按 `phase` 过滤，然后移除它们。请在发生失败的具体页面上运行。

```javascript
const TARGET_RUN_ID = 'ds-build-2024-001'; // run ID to clean
const TARGET_PHASE  = 'phase3';            // optionally filter by phase ('' = all phases)
const PAGE_NAME     = 'Button';            // page to clean (or null for all pages)

const pagesToSearch = PAGE_NAME
  ? [figma.root.children.find(p => p.name === PAGE_NAME)].filter(Boolean)
  : figma.root.children;

const removed = [];
const skipped = [];

for (const page of pagesToSearch) {
  await figma.setCurrentPageAsync(page);

  const orphans = page.findAll(node => {
    const runId = node.getSharedPluginData('dsb', 'run_id');
    if (runId !== TARGET_RUN_ID) return false;
    if (TARGET_PHASE && node.getSharedPluginData('dsb', 'phase') !== TARGET_PHASE) return false;
    return true;
  });

  // Remove leaf-first to avoid removing parents before children
  // Sort by depth (deepest first) to avoid double-remove errors
  const sorted = orphans.slice().sort((a, b) => {
    let depthA = 0, depthB = 0;
    let n = a; while (n.parent) { depthA++; n = n.parent; }
    n = b; while (n.parent) { depthB++; n = n.parent; }
    return depthB - depthA;
  });

  for (const node of sorted) {
    try {
      if (node.removed) continue; // already removed (was a child of removed parent)
      node.remove();
      removed.push({ id: node.id, name: node.name, key: node.getSharedPluginData('dsb', 'key') });
    } catch (e) {
      skipped.push({ id: node.id, name: node.name, error: e.message });
    }
  }
}

return { removed: removed.length, skipped: skipped.length, details: removed };
```

运行清理后，在目标页面调用 `get_metadata`，确认孤立节点已消失后再重试。

---

## 3. 幂等模式：创建前检查

在每个创建操作开始时运行幂等检查。如果实体已经存在（标记了预期的 `key`），跳过创建并返回现有 ID。

### variable collection 的创建前检查

```javascript
const KEY = 'collection/color';
const RUN_ID = 'ds-build-2024-001';
const COLLECTION_NAME = 'Color';

// Check: does a collection tagged with this key already exist?
const allCollections = await figma.variables.getLocalVariableCollectionsAsync();
// Variables/collections support sharedPluginData too — check by name as fallback
// Note: VariableCollection sharedPluginData is set via collection.setSharedPluginData(...)
const existing = allCollections.find(c =>
  c.getSharedPluginData('dsb', 'key') === KEY
);

if (existing) {
  return {
    collectionId: existing.id,
    modeIds: existing.modes.map(m => ({ name: m.name, id: m.modeId })),
    alreadyExisted: true,
  };
}

// Create fresh
const collection = figma.variables.createVariableCollection(COLLECTION_NAME);
collection.setSharedPluginData('dsb', 'run_id', RUN_ID);
collection.setSharedPluginData('dsb', 'key', KEY);

// Rename default mode, add second mode
collection.renameMode(collection.modes[0].modeId, 'Light');
const darkModeId = collection.addMode('Dark');

return {
  collectionId: collection.id,
  modeIds: [
    { name: 'Light', id: collection.modes[0].modeId },
    { name: 'Dark',  id: darkModeId },
  ],
};
```

### page 的创建前检查

```javascript
const KEY = 'page/button';
const PAGE_NAME = 'Button';
const RUN_ID = 'ds-build-2024-001';

// Check by sharedPluginData key first, then by name as fallback
let page = figma.root.children.find(p => p.getSharedPluginData('dsb', 'key') === KEY);
if (!page) {
  page = figma.root.children.find(p => p.name === PAGE_NAME);
}

if (page) {
  // Ensure it's tagged if it was found by name only
  if (!page.getSharedPluginData('dsb', 'key')) {
    page.setSharedPluginData('dsb', 'run_id', RUN_ID);
    page.setSharedPluginData('dsb', 'key', KEY);
  }
  return { pageId: page.id, alreadyExisted: true };
}

page = figma.createPage();
page.name = PAGE_NAME;
page.setSharedPluginData('dsb', 'run_id', RUN_ID);
page.setSharedPluginData('dsb', 'key', KEY);

return { pageId: page.id, alreadyExisted: false };
```

### component set 的创建前检查

```javascript
const KEY = 'componentset/button';
const PAGE_ID = 'PAGE_ID_FROM_STATE';
const RUN_ID = 'ds-build-2024-001';

const page = await figma.getNodeByIdAsync(PAGE_ID);
await figma.setCurrentPageAsync(page);

const existing = page.findAll(n =>
  n.type === 'COMPONENT_SET' && n.getSharedPluginData('dsb', 'key') === KEY
);

if (existing.length > 0) {
  return {
    componentSetId: existing[0].id,
    alreadyExisted: true,
  };
}

// ... proceed with creation
return { componentSetId: null, alreadyExisted: false };
```

---

## 4. 状态账本

### JSON Schema

在多次调用之间，在你的上下文中维护一份状态账本（不存入 Figma 文件）。这是 node ID、已完成步骤和待验证项的事实来源。

```json
{
  "runId": "ds-build-2024-001",
  "phase": "phase3",
  "step": "component-button/combine-variants",
  "completedSteps": [
    "phase0",
    "phase1/collections",
    "phase1/primitives",
    "phase1/semantics",
    "phase2/pages",
    "phase2/foundations-docs",
    "phase3/component-avatar",
    "phase3/component-icon"
  ],
  "entities": {
    "collections": {
      "primitives": "VariableCollectionId:1234:5678",
      "color":      "VariableCollectionId:1234:5679",
      "spacing":    "VariableCollectionId:1234:5680"
    },
    "variables": {
      "color/bg/primary":         "VariableId:2345:1",
      "color/bg/secondary":       "VariableId:2345:2",
      "color/bg/disabled":        "VariableId:2345:3",
      "color/text/on-primary":    "VariableId:2345:4",
      "color/text/on-secondary":  "VariableId:2345:5",
      "color/text/disabled":      "VariableId:2345:6",
      "spacing/sm":               "VariableId:2345:7",
      "spacing/md":               "VariableId:2345:8",
      "spacing/lg":               "VariableId:2345:9",
      "radius/md":                "VariableId:2345:10"
    },
    "modes": {
      "color/light": "2345:1",
      "color/dark":  "2345:2"
    },
    "pages": {
      "Cover":       "0:1",
      "Foundations": "0:2",
      "Button":      "0:3"
    },
    "components": {
      "Icon":        "3456:1",
      "Avatar":      "3456:2",
      "Button":      "3456:3"
    },
    "componentSets": {
      "Button": "4567:1"
    }
  },
  "pendingValidations": [
    "Button:metadata",
    "Button:screenshot"
  ],
  "userCheckpoints": {
    "phase0": "approved-2024-01-15",
    "phase1": "approved-2024-01-15",
    "phase2": "approved-2024-01-15",
    "component-avatar": "approved-2024-01-15"
  }
}
```

### 在调用之间持久化

每次成功的 `use_figma` 调用之后：
1. 从返回值中提取所有 ID
2. 将它们添加到账本中对应的 `entities` 区域
3. 将已完成步骤添加到 `completedSteps`
4. 如果此调用完成了某项验证，则从 `pendingValidations` 中移除
5. 将 `phase` 和 `step` 更新为当前位置

### 会话开始时重新补全状态

如果对话中断后恢复，读取状态账本并验证关键实体仍然存在：

```javascript
// Verify that critical nodes from the ledger still exist
const toVerify = {
  'color-collection':  'VariableCollectionId:1234:5679',
  'button-page':       '0:3',
  'button-componentset': '4567:1',
};

const results = {};
for (const [label, id] of Object.entries(toVerify)) {
  const node = await figma.getNodeByIdAsync(id)
    .catch(() => null);
  results[label] = node ? { found: true, name: node.name } : { found: false };
}

return results;
```

如果任何实体缺失，将创建它的阶段视为未完成，并从该 checkpoint 重新运行。

---

## 5. 恢复协议

### 步骤 1：检查文件中的 `run_id` 标签

```javascript
const TARGET_RUN_ID = 'ds-build-2024-001';
const inventory = { pages: [], variables: [], componentSets: [], frames: [] };

// Scan pages
for (const page of figma.root.children) {
  if (page.getSharedPluginData('dsb', 'run_id') === TARGET_RUN_ID) {
    inventory.pages.push({ id: page.id, name: page.name, key: page.getSharedPluginData('dsb', 'key') });
  }
}

// Scan variables
const allVars = await figma.variables.getLocalVariablesAsync();
for (const v of allVars) {
  if (v.getSharedPluginData('dsb', 'run_id') === TARGET_RUN_ID) {
    inventory.variables.push({ id: v.id, name: v.name, key: v.getSharedPluginData('dsb', 'key') });
  }
}

// Scan all component sets and frames on each page
for (const page of figma.root.children) {
  await figma.setCurrentPageAsync(page);
  const nodes = page.findAll(n => n.getSharedPluginData('dsb', 'run_id') === TARGET_RUN_ID);
  for (const n of nodes) {
    if (n.type === 'COMPONENT_SET') {
      inventory.componentSets.push({ id: n.id, name: n.name, key: n.getSharedPluginData('dsb', 'key') });
    } else if (n.type === 'FRAME') {
      inventory.frames.push({ id: n.id, name: n.name, key: n.getSharedPluginData('dsb', 'key') });
    }
  }
}

return inventory;
```

### 步骤 2：根据 inventory 重建状态

将 inventory 中的 key 映射回状态账本 schema。对于找到的每个带 `key` 的实体，将其 ID 添加到相应区域。将对应步骤标记为 `completedSteps`。

映射示例：
```
key: 'collection/color'        → entities.collections.color
key: 'variable/color/bg/primary' → entities.variables['color/bg/primary']
key: 'page/button'             → entities.pages.Button
key: 'componentset/button'     → entities.componentSets.Button
```

### 步骤 3：识别恢复点

恢复点是工作流中第一个不在 `completedSteps` 中的步骤。如果 inventory 显示 Button component set 已存在，但待验证列表中还有 `'Button:screenshot'`，恢复点就是 screenshot 验证调用，而不是重新创建。

使用工作流中的 checkpoint 表确定从哪个 phase 继续：

```
Phase 0 complete: all planned pages listed in entities.pages
Phase 1 complete: all planned variables listed in entities.variables with correct scopes
Phase 2 complete: all structural pages + foundations doc frames present
Phase 3 complete (per component): componentSet exists + no pending validations + user checkpoint recorded
```

---

## 6. 失败分类

### 可恢复错误

这些错误可以修复并重试，且不会影响已经创建的实体：

| 类别 | 示例 | 恢复方式 |
|---|---|---|
| 布局错误 | variants 堆叠在 (0,0)，padding 值错误 | 只重新运行定位步骤 |
| 命名问题 | variant 名称拼写错误、大小写错误 | 通过 `dsb_key` 查找节点，更新 `name` 属性 |
| 缺少属性接线 | 未设置 `componentPropertyReferences` | 通过 ID 查找 component set，重新运行属性接线步骤 |
| 遗漏 variable 绑定 | fill 被 hardcode，而不是绑定 | 通过 `dsb_key` 查找节点，重新绑定 fill |
| 绑定了错误 variable | 绑定到了错误的 variable ID | 使用正确的 variable ID 重新绑定 |
| 文本不可见 | 写入文本前未加载 font | 先使用 `loadFontAsync` 重新运行文本创建 |
| 脚本超时 | 脚本在完成前超过时间限制 | 脚本是原子的，没有创建任何内容。缩小范围（每次调用创建更少节点）后重试 |

### 结构损坏（需要回滚或重启）

这些错误会让文件处于不可靠状态，继续向前执行并不安全：

| 类别 | 示例 | 恢复方式 |
|---|---|---|
| component 循环 | component instance 被意外嵌套到自身内部 | 完整清理受影响的 component，并从 Call 1 重启该 component |
| 对非 component 使用 combineAsVariants | 将混合节点类型传给 combineAsVariants，导致意外合并 | 移除畸形的 component set，从 variant 创建重新运行 |
| Variable collection ID 漂移 | collection 被删除并重新创建，状态账本中的旧 ID 已过期 | 完整重新运行 Phase 1，并更新状态账本中的所有 ID |
| Page 删除 | component set 创建后，其所在 page 被删除 | 视为 Phase 2 未完成；重新创建 page，并重新运行受影响的 component 创建 |
| Mode 限制超出 | `addMode` 因方案是 Starter 或 Professional 而抛错 | 重新设计 variable collection 架构以符合 mode 限制，然后重启 Phase 1 |

**从结构损坏恢复**：对整个 run ID 运行 `cleanupOrphans`，然后从受影响的 phase 重启。不要尝试就地修补已损坏结构。

---

## 7. 常见错误表

| 错误消息 | 可能原因 | 修复 |
|---|---|---|
| `"Cannot create component from node"` | 尝试对 component 内部节点调用 `createComponentFromNode` | 改为创建新的 component：`figma.createComponent()` |
| `"in addMode: Limited to N modes only"` | 触发 plan mode 限制（Starter=1，Professional=4） | 重新设计为使用更少 mode，或升级 plan |
| `"setCurrentPageAsync: page does not exist"` | page 已删除或 ID 错误 | 使用幂等模式重新创建 page |
| `"Cannot read properties of null"` | `getNodeByIdAsync` 返回 null，节点已删除 | 运行恢复协议以查找现存内容，并更新状态账本 |
| `"Expected nodes to be component nodes"` | 将非 ComponentNode 传给 `combineAsVariants` | 过滤数组：`nodes.filter(n => n.type === 'COMPONENT')` |
| `"in createVariable: Cannot create variable"` | collection 已删除或 ID 错误 | 使用 `getVariableCollectionByIdAsync` 验证 collection 存在 |
| `"font not loaded"` | 未先调用 `loadFontAsync` 就调用文本属性 setter | 在文本操作前添加 `await figma.loadFontAsync({ family, style })` |
| `"Cannot set properties of a read-only array"` | 尝试就地修改 fills/strokes | 先 clone：`const fills = JSON.parse(JSON.stringify(node.fills))` |
| `"Expected RGBA color"` | color 值超出 0-1 范围 | 将 RGB 0-255 值除以 255：`{ r: 65/255, g: 85/255, b: 143/255 }` |
| `"Cannot add children to a non-parent node"` | 尝试向叶子节点（text、rect）追加 child | 确保 parent 是 FrameNode、ComponentNode 或 GroupNode |
| `"in combineAsVariants: nodes must be in the same parent"` | components 位于不同 page | 合并前将所有 components 移到同一 page |
| `"Script exceeded time limit"` | 一次调用中的循环创建了太多节点 | 拆分工作：每次调用创建 N/2 个 variants |
| Component set 自行删除 | 尝试创建没有 child 的 component set | `combineAsVariants` 至少需要 1 个节点，始终传入 1+ 个 |
| `addComponentProperty` 返回意外名称 | 这是正常现象，`BOOLEAN`/`TEXT`/`INSTANCE_SWAP` 会得到 `#id:id` 后缀 | 立即保存返回的 key 并使用它，而不是使用输入名称 |

---

## 8. 分阶段恢复指南

### Phase 1 失败（variable 创建）

由于 `use_figma` 是原子的，失败调用不会创建任何内容。最常见场景是 Phase 1 中部分调用成功（创建了一些 variables），而后续调用失败。

恢复步骤：
1. 运行检查脚本，查找所有标记了你的 `run_id` 的 variables
2. 与计划对比，识别哪些 variables 已成功创建、哪些仍然缺失
3. 如果已成功创建的 variable 值错误，调用 `variable.remove()` 并重新创建
4. 修复失败脚本并重试；这是安全的，因为失败调用没有创建任何内容
5. 在所有计划内 variables 都存在且 scope 与 code syntax 正确之前，不要进入 Phase 2

**最常见的 Phase 1 失败**：创建大量 variables 时脚本超时。修复方式：分批创建 variable，每次调用最多创建 20-30 个 variables。

### Phase 2 中途失败（page/file 结构）

症状：部分 pages 已存在，其他 pages 缺失；foundations doc frames 不完整。

恢复步骤：
1. 识别哪些 pages 已成功创建（检查 `key` 标签）
2. 将剩余 pages 标记为 pending，并在后续调用中创建它们
3. 如果 foundations doc frame 畸形，在该 page 上针对 `dsb_phase: 'phase2'` 运行 `cleanupOrphans`，然后重新创建

Phase 2 失败很少需要回滚 Phase 1，除非 page 结构本身已损坏（这种情况并不常见）。

### Phase 3 失败（component 创建）

这是长构建中最常见的失败模式。由于 `use_figma` 是原子的，失败调用不会创建任何内容；但 component 创建序列中此前成功的调用已经创建了状态。按序列中失败的调用处理：

```
If failure in Call 1 (page creation):
  → Nothing was created. Fix the script and retry.

If failure in Call 2 (doc frame):
  → Call 1's page exists. Fix Call 2 and retry — idempotency check handles it.

If failure in Call 3 (base component):
  → Calls 1-2 succeeded. Fix Call 3 and retry.

If failure in Call 4 (variant creation):
  → Call 3's base component exists. Fix Call 4 and retry.
  → If you need to restart from Call 3, clean up Call 3's nodes first
    using cleanupOrphans scoped to the component page.

If failure in Call 5 (combineAsVariants + layout):
  → Variant ComponentNodes from Call 4 exist but aren't combined yet.
  → Fix Call 5 and retry.
  → If the component set was already created by a prior attempt of Call 5
    that succeeded, remove it first, then re-run.

If failure in Call 6 (component properties):
  → The component set already exists and is structurally sound.
  → Fix Call 6 and retry — addComponentProperty is safe to retry if
    you first check componentPropertyDefinitions for existing properties.
  → Idempotency check: if 'Label' property already exists, skip addComponentProperty.
```

**component properties 的幂等性（Call 6 重试）：**

```javascript
const existingDefs = cs.componentPropertyDefinitions;
const labelKey = existingDefs['Label']
  ? Object.keys(existingDefs).find(k => k.startsWith('Label'))
  : cs.addComponentProperty('Label', 'TEXT', 'Button');
```

### Phase 4 中途失败（QA / Code Connect）

Phase 4 是非破坏性的。这里的失败不会损坏 Phase 3 的工作。常见失败：

- **Accessibility audit 发现对比度失败**：不要尝试自动修复。报告失败的具体 variable ID 和 token name，然后询问用户要更新哪个值。
- **Naming audit 发现重复项**：列出所有重复项及其 `key` 值，询问用户保留哪个，然后移除重复项。
- **Code Connect mapping 失败**：视为未完成，而不是已损坏。继续执行，并将其保留为 pending。
