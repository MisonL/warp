# 测试图片

本目录包含用于验证 Markdown 图片渲染的测试图片。

## 文件

- `image_test.md` - 包含多种图片引用的 Markdown 文件
- `sample1.jpg` - 400x300 JPEG，带垂直渐变（蓝色到红橙色）和白色边框
- `sample2.jpg` - 600x400 JPEG，带棋盘图案和橙色圆形叠层
- `sample3.png` - 300x300 PNG，带径向图案和透明度（alpha channel）
- `parent_test.jpg` - 300x200 JPEG，带对角蓝色条纹，位于父目录中，用于测试相对路径解析

## 目的

这些图片用于测试图片渲染的不同方面：
- 不同格式（JPEG、PNG）
- 不同尺寸（300x300、400x300、600x400）
- 透明度（带 alpha channel 的 PNG）
- 用于验证正确渲染的视觉图案（渐变、图案、形状）
- 相对路径解析（父目录引用）
