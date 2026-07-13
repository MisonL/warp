---
name: figma-use
description: "**必需前置技能**：每次调用 `use_figma` 工具前都必须先调用此技能。不要在未加载此技能时直接调用 `use_figma`。适用于需要在 Figma 文件上下文中执行 JavaScript 的写入操作或特殊读取操作，例如创建、编辑或删除节点，设置变量或 token，构建组件和变体，修改 auto-layout 或填充，绑定变量到属性，或以编程方式检查文件结构。"
description_zh_CN: "**必需前置技能**：每次调用 `use_figma` 工具前都必须先调用此技能。适用于需要在 Figma 文件上下文中执行 JavaScript 的写入操作或特殊读取操作，例如创建/编辑/删除节点、设置变量或 token、构建组件和变体、修改 auto-layout 或填充、绑定变量到属性，或以编程方式检查文件结构。"
disable-model-invocation: false
---

# use_figma - Figma Plugin API Skill

使用 `use_figma` MCP 在 Figma 文件中通过 Plugin API 执行 JavaScript。详细参考文档位于 `references/`。

每次调用 `use_figma` 都传 `skillNames: "figma-use"`。这是 usage logging 参数，不影响执行。

如果任务涉及把代码中的完整页面、screen 或多区块 layout 构建到 Figma，还要加载 `figma-generate-design`。它负责 screen 构建工作流，本技能负责 API 规则。

开始前，读取 `references/plugin-api-standalone.index.md` 理解可用 API。写 Plugin API 代码时，按需 grep `references/plugin-api-standalone.d.ts` 中的具体类型、方法和属性，不要一次加载完整大文件。

处理 design system 时，先读 `references/working-with-design-systems/wwds.md`，再按需读取 components、variables、text styles、effect styles 相关参考。

## 1. 关键规则

1. 用 `return` 返回数据。返回值会自动 JSON 序列化。不要调用 `figma.closePlugin()`，不要包 async IIFE。
2. 写普通 JavaScript，支持 top-level `await` 和 `return`。代码会自动包在 async context 中。
3. `figma.notify()` 会抛出 "not implemented"，不要使用。
4. `getPluginData()` 和 `setPluginData()` 不支持。使用 `getSharedPluginData()` 和 `setSharedPluginData()`，或通过返回 ID 追踪节点。
5. `console.log()` 不会返回给 agent。输出必须用 `return`。
6. 小步增量工作。大型操作拆成多个 `use_figma` 调用，每步后验证。
7. 颜色使用 0 到 1 范围，不是 0 到 255。
8. fills 和 strokes 是只读数组。要 clone、修改、再重新赋值。
9. 写任何 text 前必须先 `await figma.loadFontAsync({family, style})`。
10. 页面是增量加载。切换页面使用 `await figma.setCurrentPageAsync(page)`。
11. `setBoundVariableForPaint` 返回新的 paint，必须接住并重新赋值。
12. `createVariable` 可接收 collection object 或 ID string，优先 object。
13. `layoutSizingHorizontal/Vertical = 'FILL'` 必须在 `parent.appendChild(child)` 后设置。非 auto-layout node 上的 `'HUG'` 也一样。
14. 新 top-level node 要放在远离 `(0,0)` 的空白处，避免覆盖已有内容。
15. `use_figma` 报错时停止，不要立即重试。失败脚本是原子的，文件不会部分修改。先读错误，修正脚本，再重试。
16. 每次创建或修改 canvas node 时，必须返回所有 created/mutated node IDs。
17. 每个 Promise 都要 `await`。不要让 async 调用 fire-and-forget。

## 2. 页面规则

每次 `use_figma` 调用时，`figma.currentPage` 会重置到第一页。若目标在其他页面，脚本开头必须切换：

```js
const targetPage = figma.root.children.find((p) => p.name === "My Page");
await figma.setCurrentPageAsync(targetPage);
```

遍历所有页面时：

```js
for (const page of figma.root.children) {
  await figma.setCurrentPageAsync(page);
  // page.children now loaded
}
```

同步赋值 `figma.currentPage = page` 在 `use_figma` runtime 中会报错。

## 3. `return` 是唯一输出通道

agent 只能看到 `return` 值。

- 创建或修改节点时返回 `{ createdNodeIds: [...], mutatedNodeIds: [...] }`。
- 进度信息返回 count、errors、status 等结构化对象。
- 错误可以直接 `throw`，工具会捕获。
- 不要依赖 `console.log()`。

## 4. Editor mode

`use_figma` 默认在 design mode 中工作。design mode 可用 Rectangle、Frame、Component、Text、Ellipse、Star、Line、Vector、Polygon、BooleanOperation、Slice、Page、Section、TextPath。

FigJam 有不同可用节点类型。design mode 中不支持 Sticky、Connector、ShapeWithText、CodeBlock、Slide、SlideRow、Webpage。

## 5. 增量工作流

最常见错误是一次调用做太多事。

推荐模式：

1. 先 inspect。创建前用只读 `use_figma` 发现已有 pages、components、variables、命名约定。
2. 每次只做一件事。创建 variables、创建 components、组合 layout 分开做。
3. 每次返回 IDs。后续调用需要这些 ID。
4. 每步后验证。用 `get_metadata` 检查结构，用 `get_screenshot` 检查视觉。
5. 修好再继续。不要在坏基础上继续构建。

复杂任务建议顺序：

```text
Step 1: Inspect file
Step 2: Create tokens/variables
Step 3: Create individual components
Step 4: Compose layouts from component instances
Step 5: Final verification
```

验证重点：

| 操作后 | 用 get_metadata 检查 | 用 get_screenshot 检查 |
| --- | --- | --- |
| 创建 variables | collection count、variable count、mode names | 无 |
| 创建 components | child count、variant names、properties | variants 可见且不塌陷 |
| 绑定 variables | node properties 有 binding | 颜色和 token 正确 |
| 组合 layouts | instance 有 mainComponent，层级正确 | 无裁剪、无重叠、间距正确 |

## 6. 错误恢复

`use_figma` 是原子执行。脚本报错时不会修改文件。

报错后：

1. 停止，不要马上重试。
2. 仔细读错误，判断是 API 用法、字体、属性值还是 ID 问题。
3. 错误不清楚时，用 `get_metadata` 或 `get_screenshot` 查看当前状态。
4. 修正脚本。
5. 重试。

常见错误：

| 错误 | 原因 | 修复 |
| --- | --- | --- |
| `"not implemented"` | 使用了 `figma.notify()` | 删除，改用 `return` |
| `"node must be an auto-layout frame..."` | append 前设置了 FILL/HUG | 先 append，再设置 sizing |
| `"Setting figma.currentPage is not supported"` | 用同步 page setter | 改用 `await figma.setCurrentPageAsync(page)` |
| Property value out of range | 颜色用了 0 到 255 | 除以 255 |
| `"Cannot read properties of null"` | 节点不存在或页面上下文错误 | 检查 page 和 ID |
| Script hangs | 无限循环或 Promise 未 resolve | 检查循环和 await |
| `"The node with id X does not exist"` | instance detach 改变了 ID | 从稳定父 frame 重新遍历发现节点 |

脚本成功但结果不对时，先用 metadata 看结构，再用 screenshot 看视觉。定位结构问题还是视觉问题，然后写只改坏部分的 targeted fix，不要全部重建。

## 7. Pre-flight checklist

每次提交 `use_figma` 前检查：

- 使用 `return`，没有 `figma.closePlugin()`。
- 没有 async IIFE。
- 返回值包含 IDs、counts 等可行动信息。
- 没有 `figma.notify()`。
- 没把 `console.log()` 当输出。
- 颜色是 0 到 1。
- fills/strokes 重新赋值。
- 页面切换使用 `await figma.setCurrentPageAsync(page)`。
- FILL/HUG 在 append 后设置。
- 写 text 前加载字体。
- `lineHeight` 和 `letterSpacing` 使用 `{unit, value}`。
- `resize()` 在 sizing mode 之前调用。
- 多步工作流中，前一步 IDs 作为字符串字面值传入。
- 新 top-level node 避开 `(0,0)`。
- 返回所有 created/mutated node IDs。
- 所有 async 调用都 `await`。

## 8. 创建前发现约定

不同 Figma 文件有不同命名、variable 结构和 component 模式。先 inspect，不要强加新约定。

列出 pages 和 top-level nodes：

```js
const pages = figma.root.children.map(p => `${p.name} id=${p.id} children=${p.children.length}`);
return pages.join('\n');
```

列出所有 pages 中的 components：

```js
const results = [];
for (const page of figma.root.children) {
  await figma.setCurrentPageAsync(page);
  page.findAll(n => {
    if (n.type === 'COMPONENT' || n.type === 'COMPONENT_SET')
      results.push(`[${page.name}] ${n.name} (${n.type}) id=${n.id}`);
    return false;
  });
}
return results.join('\n');
```

列出 variable collections：

```js
const collections = await figma.variables.getLocalVariableCollectionsAsync();
const results = collections.map(c => ({
  name: c.name, id: c.id,
  varCount: c.variableIds.length,
  modes: c.modes.map(m => m.name)
}));
return results;
```

## 9. 参考文档

按任务需要读取：

| 文档 | 何时读取 |
| --- | --- |
| `references/gotchas.md` | 每次 `use_figma` 前了解常见坑 |
| `references/common-patterns.md` | 需要可复用脚手架示例 |
| `references/plugin-api-patterns.md` | 创建或编辑节点 |
| `references/api-reference.md` | 需要确切 API surface |
| `references/validation-and-recovery.md` | 多步写入或错误恢复 |
| `references/component-patterns.md` | 创建 components 和 variants |
| `references/variable-patterns.md` | 创建和绑定 variables |
| `references/text-style-patterns.md` | 创建和应用 text styles |
| `references/effect-style-patterns.md` | 创建和应用 effect styles |
| `references/plugin-api-standalone.index.md` | 理解完整 API index |
| `references/plugin-api-standalone.d.ts` | grep 精确 type signature |
