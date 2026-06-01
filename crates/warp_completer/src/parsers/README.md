# 基础 parser

该 parser 是一个类型驱动的 recursive descent parser。这份仍在完善中的指南会通过示例，以友好方式概览它的工作方式。部分位置包含 `你知道吗？` 备注，它们是值得了解的小技巧，因为 parser 会依赖这些内容。

# 步骤

1. Lex. - [开始 parse 时会发生什么](#开始-parse-时会发生什么)
2. Lite Parse. - [处理 token](#处理-token)
3. 类型驱动 full parse. - [Full parse](#)

## 开始 parse 时会发生什么？

假设我们想 parse 输入 `warp --disable-telemetry`。Command call 通常采用 `cmd <arg1> <arg2> <argN>` 形式，其中 `<arg>` 是 positional parameter。我们使用 parser 的主要函数来检查。第一步是调用 tokenizer（这里是 `lex` 函数）：

```rust
let input = "warp --disable_telemetry";
let start_offset = 0;

let (tokens, _) = lex(input, start_offset);
println!("{:#?}", tokens);
```

输出如下：

```rust
(
    [
        Token {
            contents: Baseline(
                "warp",
            ),
            span: Span {
                start: 0,
                end: 4,
            },
        },
        Token {
            contents: Space,
            span: Span {
                start: 4,
                end: 5,
            },
        },
        Token {
            contents: Baseline(
                "--disable-telemetry",
            ),
            span: Span {
                start: 5,
                end: 24,
            },
        },
    ],
    None,
)
```

实际上，我们在这里得到的是被 *tokenize* 后的输入源。Parser 使用 `start_offset` 来计算每个 bare word 的 span。再仔细看一点，可以注意到这些 token 带有 `span` 字段。[`Span` struct type](../meta.rs) 有 `start` 和 `end` 数字字段，有助于了解某个内容所在的位置。

#### 你知道吗？`Span` struct type。

下面使用上方输出中的 span 数字，并使用 `Span` struct 关联的 `slice` 函数。该函数接收一个字符串作为输入，并使用 `Span` 的 `start` 和 `end` 值返回它的一个 slice。我们用上面输出中的数字创建三个 `Span`，并从传给 lexer 的输入字符串中切出每个片段。

```rust
let input = "warp --disable-telemetry";

let word1 = Span::new(0,4);
let word2 = Span::new(4,5);
let word3 = Span::new(5,24);

assert_eq!(word1.slice(input), "warp");
assert_eq!(word2.slice(input), " ");
assert_eq!(word3.slice(input), "--disable-telemetry");
```

## 处理 token

基础 parser 的下一步与传统 parsing 中的 lexing/parsing 并没有太大不同。当前任务是理解 token 的边界，并准备好用于 full parse 的一般形态。此时我们暂时不需要做更多事情，因为这些 token 中的每一个都可以传给没有注册 signature 的命令（*稍后会详细说明*）。我们称这一步为 `Lite` parse step。

下面是 grammar rule 的极简视图：

```
LiteRootNode    := LiteGroup
LiteGroup       := LitePipeline (';' LitePipeline)*
LitePipeline    := LiteCommand ('|' LiteCommand)*
LiteCommand     := argument+
// (*more grammar later*)
```

它们表示为基础 parser 生成的 struct：

```rust
pub struct LiteRootNode {
    pub groups: Vec<LiteGroup>,
}

pub struct LiteGroup {
    pub pipelines: Vec<LitePipeline>,
}

pub struct LitePipeline {
    pub commands: Vec<LiteCommand>,
}

pub struct LiteCommand {
    // this is important!
    pub parts: Vec<Spanned<String>>,
    pub post_whitespace: Option<Span>,
}
```

#### 你知道吗？`Spanned<T>` generic struct。

`LiteCommand` 有一个 `parts` 字段，用来保存 `Spanned<String>` 的 vector。我们前面提到过 `Span` 类型。这里要说的是一个 generic `Spanned<T>`，它允许将任意 `T` 与一个 `Span` 值包装在一起。下面给出该类型，以及一些使用其 helper function 的示例。

```rust
pub struct Spanned<T> {
    pub span: Span,
    pub item: T,
}

let example = Spanned { item: String::from("warp"), span: Span::new(0,4) };
assert_eq!(example.item, "warp".to_string());
assert_eq!(example.span, Span::new(0,4));

let example = String::from("warp").spanned(Span::new(0,4));
assert_eq!(example.item, "warp".to_string());
assert_eq!(example.span, Span::new(0,4));

let example = "warp -p --disable-telemetry";

let full_span = Span::new(0, example.len());
let first_flag_span = Span::new(5,7);

assert_eq!(first_flag_span.slice(example), "-p");
assert_eq!(first_flag_span.until(full_span), Span::new(5,27));
assert_eq!(first_flag_span.until(full_span).slice(example), "-p --disable-telemetry");

```

这很有用，因为一旦 `lite` parse step 发生，我们就会得到所需的全部输出，并且 `span` 都已正确计算。下面继续对原始示例执行 lite parse（函数 `parse_tokens`），输入为 lexer 处理 `warp --disable-telemetry` 时生成的 token，如下：

```rust
let input = "warp --disable-telemetry";
let start_offset = 0;

let (tokens, _) = lex(input, start_offset);
let (lite_node, _) = parse_tokens(tokens);

let expected_word1 = String::from("warp").spanned(Span::new(0,4));
let expected_word2 = String::from("--disable-telemetry").spanned(Span::new(5,24));

assert_eq!(lite_node.groups[0].pipelines[0].commands[0].parts, vec![expected_word1, expected_word2]);
assert_eq!(lite_node.groups[0].pipelines[0].commands.len(), 1);

println!("{:#?}", lite_node);
```

我们得到一个清晰的 lite node：

```rust
LiteRootNode {
    groups: [
        LiteGroup {
            pipelines: [
                LitePipeline {
                    commands: [
                        LiteCommand {
                            parts: [
                                Spanned {
                                    span: Span {
                                        start: 0,
                                        end: 4,
                                    },
                                    item: "warp",
                                },
                                Spanned {
                                    span: Span {
                                        start: 5,
                                        end: 24,
                                    },
                                    item: "--disable-telemetry",
                                },
                            ],
                            post_whitespace: None,
                        },
                    ],
                },
            ],
        },
    ],
}
```

对于更复杂的输入（例如命令由 `|` 分隔和/或包含 `;`），lite parser 会有效创建必要的 `LitePipeline` 来表达它。下面看看对输入 `warp config-set --extension-path="/path/to/dir" ; echo $WARP_VAR"` 执行 lite parse 会发生什么（*这里因为 ; 字符而有两个 pipeline*）：

```rust
let input = "warp config-set --extension-path=\"/path/to/dir\" ; echo $WARP_VAR";
let start_offset = 0;

let (tokens, _) = lex(input, start_offset);
let (lite_node, _) = parse_tokens(tokens);

println!("{:#?}", lite_node);
```

```rust
LiteRootNode {
    groups: [
        LiteGroup {
            pipelines: [
                LitePipeline {
                    commands: [
                        LiteCommand {
                            parts: [
                                Spanned {
                                    span: Span {
                                        start: 0,
                                        end: 4,
                                    },
                                    item: "warp",
                                },
                                Spanned {
                                    span: Span {
                                        start: 5,
                                        end: 15,
                                    },
                                    item: "config-set",
                                },
                                Spanned {
                                    span: Span {
                                        start: 16,
                                        end: 47,
                                    },
                                    item: "--extension-path=\"/path/to/dir\"",
                                },
                            ],
                            post_whitespace: Some(
                                Span {
                                    start: 47,
                                    end: 48,
                                },
                            ),
                        },
                    ],
                },
                LitePipeline {
                    commands: [
                        LiteCommand {
                            parts: [
                                Spanned {
                                    span: Span {
                                        start: 50,
                                        end: 54,
                                    },
                                    item: "echo",
                                },
                                Spanned {
                                    span: Span {
                                        start: 55,
                                        end: 64,
                                    },
                                    item: "$WARP_VAR",
                                },
                            ],
                            post_whitespace: None,
                        },
                    ],
                },
            ],
        },
    ],
}
 ```

 ## 类型驱动 full parse

TODO
