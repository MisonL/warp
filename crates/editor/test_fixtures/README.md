# 测试 Fixture

本目录包含用于 Markdown editor 手动测试和验证的测试文件。

## 图片

`images/` 目录包含示例图片和一个测试 Markdown 文件（`image_test.md`），用于展示多种图片渲染场景：

- 相对路径（`./sample1.jpg`）
- 父目录引用（`../parent_test.jpg`）
- 绝对路径
- 不同图片格式（JPG、PNG）
- 列表中的图片
- 空 alt text

要测试图片渲染，请在 Warp 中打开 `images/image_test.md`。

## ToC 导航

`toc_anchor_test.md` 覆盖 Markdown fragment link 导航的手动验证（大小写不敏感的 heading 匹配和滚动）。

测试时，请在 Warp 的 Markdown viewer 中打开 `toc_anchor_test.md`，并点击目录链接。
