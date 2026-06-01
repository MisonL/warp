# lsp

该 crate 为 Warp 提供仅基于 stdio 的 Language Server Protocol（LSP）客户端 transport。它会：

- 启动并管理 language server 进程（child process）
- 使用 JSON-RPC 通过 stdio 通信，并带有正确的 Content-Length framing

示例实现见 `examples/rust-lsp/main.rs`。
