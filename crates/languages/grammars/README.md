# Language Grammar

新增 language grammar 的示例：https://github.com/warpdotdev/warp-internal/pull/11501/files

## TSLanguage

我们需要一个 [TSLanguage](https://tree-sitter.github.io/tree-sitter/using-parsers#the-basic-objects) object 来解析源代码文件。我们使用开源库提供的函数来创建这些 object。这些函数通常用 C 编写，但可以在 Rust crate 中使用。

## config.yaml

你可以在特定语言的文档和 style guide 中找到这些信息。

另一个可参考的位置是 Zed 的 config.toml 文件，例如：https://github.com/zed-industries/zed/blob/85bdd9329b550475aae34340e50abd4e79f2dd82/crates/languages/src/python/config.toml

**注意：** 我们不使用自定义 highlights.scm 文件，而是使用 arborium 捆绑的 highlighting query。

## indents.scm

该文件控制我们如何判断 cursor 相对于上一行何时应缩进。

其他代码编辑器也需要这些信息，因此我们可以参考其他开源代码编辑器。我们主要需要支持 indent 和 outdent capture。功能更完整的代码编辑器会支持更高级的能力。

可查看的一些示例来源包括：
1. https://github.com/helix-editor/helix/blob/101a74bf6edbbfdf9b0628a0bdbbc307ebe10ff2/runtime/queries/python/indents.scm
1. https://github.com/zed-industries/zed/blob/85bdd9329b550475aae34340e50abd4e79f2dd82/crates/languages/src/python/indents.scm
