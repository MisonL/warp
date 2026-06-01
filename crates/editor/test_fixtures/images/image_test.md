# Markdown 图片测试

此文件测试在 Markdown 中包含图片的各种方式。

## 示例 JPG 图片

这里是一张 400x300 示例图片：

![示例图片 1](./sample1.jpg)

这里是一个更大的 600x400 示例：

![示例图片 2](./sample2.jpg)

## PNG 图片

一张 300x300 的方形 PNG：

![示例 PNG](./sample3.png)

## 多张图片

下面按顺序展示多张图片：

![示例 1](./sample1.jpg)

![示例 2](./sample2.jpg)

![示例 3](./sample3.png)

## 列表中的图片

这里是一个带图片的项目符号列表：

- 第一项
- ![列表中的行内图片](./sample1.jpg)
- 第三项

## 父目录引用

这里是一张来自父目录的图片：

![父目录图片](../parent_test.jpg)

## 绝对路径

也可以使用绝对路径（不过可移植性较差）：

![绝对路径](/Users/zach/Projects/warp/editor/test_fixtures/images/sample2.jpg)

## 空 Alt Text

![](./sample3.png)

## 图片后的文本

这里是一段位于图片之后的普通文本。图片应随文本流行内渲染。

![示例](./sample1.jpg)

这段文本位于图片之后。

---

## 测试结束

以上覆盖了基本图片渲染场景。
