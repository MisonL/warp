# WARP.md

## Table 示例截图的视觉 Sanity Check

本项目可以自动生成 table example demo 的截图，然后使用 computer vision 对它们进行 sanity check。目标是在提交或打开 PR 前，快速捕获明显的渲染 bug（例如空 cell、明显错位、缺失 header）。

### 如何捕获图片

- 使用 capture flag 构建并运行示例：
  - Baseline（参考图片）：`../../../../target/debug/examples/table-sample --capture-baseline`
  - Current（用于本地比较）：`../../../../target/debug/examples/table-sample --capture-screenshots`
- 输出目录：
  - Baseline：`screenshots/baseline/`
  - Current：`screenshots/current/`

### Sanity-check 协议（Agent/Agent Mode）

- 使用 read_file tool 上传所选目录（baseline 或 current）中的全部 PNG。
- 对每张图片，扫描以下问题：
  - UI 应渲染的位置出现完全空白/黑色/纯色的大面积区域
  - 明显缺失 header、row 或 column
  - row band 或 header 与 body 明显错位
  - 文本被截断在行中间，或因极端对比度问题不可读
- 报告任何出现上述问题的图片，并附简短说明。

备注：
- 这是快速视觉 smoke test，不是 pixel-perfect 比较。
- 如果发现失败，可通过方向键导航到单个 demo 后重新运行示例，或重新运行完整 capture 并再次检查。
