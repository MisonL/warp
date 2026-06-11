# AGENTS.md

本文件提供在本仓库中处理代码时需要遵循的指引。

## 开发命令

### 构建与运行

- `cargo run` - 在本地构建并运行 Warp
- `cargo bundle --bin warp` - 打包主应用

### 连接本地 warp-server 运行

要将 Warp 客户端连接到本地 `warp-server` 实例：

```bash
# 连接默认端口 8080 上的服务器
cargo run --features with_local_server

# 连接自定义端口上的服务器（例如 8082）
SERVER_ROOT_URL=http://localhost:8082 WS_SERVER_URL=ws://localhost:8082/graphql/v2 cargo run --features with_local_server
```

环境变量：

- `SERVER_ROOT_URL` - HTTP 端点（默认：`http://localhost:8080`）
- `WS_SERVER_URL` - WebSocket 端点（默认：`ws://localhost:8080/graphql/v2`）

### 测试

- `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2` - 使用 nextest 运行测试
- `cargo nextest run -p warp_completer --features v2` - 使用 v2 feature 运行 completer 测试
- `cargo test --doc` - 运行文档测试
- `cargo test` - 运行单个 package 的标准测试

### Lint 与格式化

- `./script/presubmit` - 运行所有提交前检查（fmt、clippy 和测试）
- `./script/format` - 格式化代码
- `cargo clippy --workspace --all-targets --all-features --tests -- -D warnings` - 运行 clippy
- `./script/run-clang-format.py -r --extensions 'c,h,cpp,m' ./crates/warpui/src/ ./app/src/` - 格式化 C/C++/Obj-C 代码
- `find . -name "*.wgsl" -exec wgslfmt --check {} +` - 检查 WGSL shader 格式

### 平台环境准备

- `./script/bootstrap` - 执行平台相关环境准备，并根据 `skills-lock.json` 安装通用 agent skill；除非提供目标标志或环境变量覆盖，否则在需要安装或更新时提示选择 project/global。
- `./script/bootstrap --skip-common-skills` - 执行平台环境准备，但不安装或更新通用 agent skill。
- `./script/bootstrap --install-common-skills` - 从 `skills-lock.json` 显式安装通用 agent skill；这是默认行为。
- `./script/bootstrap --install-common-skills-in-repo` - 执行平台环境准备，并将通用 agent skill 安装到当前 checkout 的 `.agents/skills`。
- `./script/bootstrap --install-common-skills-globally` - 执行平台环境准备，并将通用 agent skill 安装到 `~/.agents/skills`。
- `../common-skills/scripts/install_common_skills --repo-root "$PWD" --project --if-needed` - 在当前 checkout 的 `.agents/skills` 中安装或刷新共享 agent skill。
- `../common-skills/scripts/install_common_skills --repo-root "$PWD" --global --if-needed` - 在 `~/.agents/skills` 中安装或刷新共享 agent skill。
- `../common-skills/scripts/remove_common_skills --repo-root "$PWD"` - 从当前 checkout 的 `.agents/skills` 中移除 `skills-lock.json` 列出的共享 agent skill。
- `../common-skills/scripts/remove_common_skills --repo-root "$PWD" --global` - 从 `~/.agents/skills` 中移除 `skills-lock.json` 列出的共享 agent skill。
- `../common-skills/scripts/remove_common_skills --repo-root "$PWD" --clear-lock` - 从当前 checkout 中移除共享 agent skill，并删除 `skills-lock.json`。
- `./script/install_cargo_build_deps` - 安装 Cargo 构建依赖
- `./script/install_cargo_test_deps` - 安装 Cargo 测试依赖

`skills-lock.json` 是由 `npx skills` 管理的标准项目锁文件。`warpdotdev/common-skills/scripts/install_common_skills` 在恢复前需要显式安装目标：传入 `--project`、传入 `--global`、设置 `WARP_COMMON_SKILLS_INSTALL_TARGET`，或回答 bootstrap 的交互提示。非交互流程如果没有显式目标会失败。如果缺少 `skills-lock.json`，安装器会基于 `warpdotdev/common-skills` 创建它；交互默认推荐 global；当 project 和 global 位置同时存在通用 skill 时会报错；会防止一个锁文件固定的全局安装被另一个 checkout 中不同锁文件固定的安装静默覆盖；并会在成功安装或跳过路径后按锁文件验证已安装 skill。`script/run` 和 `script/bootstrap` 会通过 `script/resolve_common_skills` 执行此安装器；该脚本仅在显式设置 `WARP_COMMON_SKILLS_SCRIPTS_DIR` 时使用它，否则运行 `warpdotdev/common-skills` 中的原始脚本。要测试远端 common-skills 分支，请设置 `WARP_COMMON_SKILLS_REF=<branch>`。Cloud 环境准备应使用 `common-skills/scripts/install_common_skills --repo-root <warp-checkout> --project --if-needed --non-interactive`，或设置 `WARP_COMMON_SKILLS_INSTALL_TARGET=project` 以避免提示。要更新已锁定的 common skills，请运行 `npx --yes skills@1.5.6 update -p -y`，并提交生成的 `skills-lock.json` 变更。

### Cargo 构建与测试性能

- 遇到 `rustc` 长时间低 CPU 占用时，先确认它是在编译、链接、等待 Cargo 锁，还是测试二进制已启动；不要把低 CPU 直接当作卡死。
- 使用官方 `--timings` 报告定位真实瓶颈，例如 `cargo test -p warp --lib --timings --no-run` 或 `cargo build --timings`。优先根据报告处理慢依赖、重复依赖、不必要 features、大 crate 和阻塞后续构建的关键 crate。
- 不要并发启动多个指向同一 workspace/target dir 的 `cargo test`。它们会争用 package cache 与 build directory 锁；应运行一个 Cargo 命令，让 Cargo 通过 `--jobs` 自己调度并行度。
- 需要反复跑 `warp --lib` 过滤测试时，优先先构建 test binary：`cargo test -p warp --lib --no-run`，再直接运行 `target/debug/deps/warp-* <test-filter> --nocapture`，减少重复 Cargo 决策、锁等待和链接。
- 当前 checkout 位于机械硬盘时，不要把 Cargo target dir 留在仓库内，也不要把个人绝对路径写入 `.cargo/config.toml` 提交。需要把构建输出放到 home 或临时目录时，使用本机 shell 环境变量，例如 `export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/warp"`，或单次命令加 `--target-dir "$HOME/.cache/cargo-target/warp"`。
- 保持 toolchain、feature set 和 target dir 稳定，避免破坏增量缓存。清理 target dir、切换 feature 或切换 toolchain 都可能触发大规模重编译。
- 可用 `RUSTC_WRAPPER=sccache` 或 Cargo `build.rustc-wrapper` 接入 `sccache` 缓存编译产物；它主要优化编译缓存，对最终链接阶段收益有限。
- macOS 链接器替换需谨慎验证。Warp 链接大量 Apple framework 和 native 库，不要在复查或小修中临时切换 linker 作为性能 workaround。

## 架构概览

这是一个基于 Rust 的终端模拟器，使用名为 **WarpUI** 的自定义 UI 框架。

### 关键组件

**WarpUI 框架**（`ui/`）：

- 使用 Entity-Component-Handle 模式的自定义 UI 框架
- 全局 `App` 对象拥有所有 view/model（entity）
- view 通过 `ViewHandle<T>` 引用其他 view
- `AppContext` 在渲染/事件期间提供对 handle 的临时访问
- element 描述视觉布局（受 Flutter 启发）
- action 系统用于事件处理
- `MouseStateHandle` 必须在构造期间创建一次，然后在任何使用鼠标输入跟踪鼠标变化的位置引用或克隆。渲染时内联创建 `MouseStateHandle::default()` 会导致所有鼠标交互都无法工作。

**主应用**（`app/`）：

- 终端模拟与 shell 管理（`terminal/`）
- AI 集成，包括 Agent Mode（`ai/`）
- 云同步和 Drive 功能（`drive/`）
- 认证和用户管理（`auth/`）
- 设置和偏好（`settings/`）
- workspace 和 session 管理（`workspace/`）

**核心库**：

- `crates/warp_core/` - 核心工具和平台抽象
- `crates/editor/` - 文本编辑功能
- `crates/warpui/` 和 `crates/warpui_core/` - 自定义 UI 框架
- `crates/ipc/` - 进程间通信
- `crates/graphql/` - GraphQL 客户端和 schema

### 关键架构模式

1. **Entity-Handle 系统**：view 通过 handle 引用其他 view，而不是直接拥有它们
2. **模块化结构**：workspace 包含多个 workspace configuration，每个 configuration 包含 terminal、notebook 等
3. **跨平台**：提供 macOS、Windows、Linux 的原生实现，并支持 WASM target
4. **AI 集成**：内置具备上下文感知和代码库索引能力的 AI assistant
5. **云同步**：对象可以通过 Warp Drive 跨设备同步

### 开发指引

**Workspace 结构**：

- 这是一个包含 60 多个 member crate 的 Cargo workspace
- 主二进制在 `app/` 中，UI 框架在 `crates/warpui/` 中
- 平台相关代码使用条件编译
- 集成测试位于 `crates/integration/`

**编码风格偏好**：

- 避免不必要的类型标注，尤其是 closure 参数中的类型标注。
- 避免使用过多 Rust 路径限定符；优先使用 import 保持简洁。按惯例将 import 语句放在文件顶部。
  例外是 cfg guard 保护的代码分支。在这些情况下，可以把 import 放进相关作用域，也可以对一次性用法使用绝对路径。
- 如果函数接收 context 参数（`AppContext`、`ViewContext` 或 `ModelContext`），该参数应命名为 `ctx` 并放在最后。唯一例外是函数接收 closure 参数时，此时 closure 应放在最后。
- 始终彻底移除未使用参数，而不是给参数加 `_` 前缀。相应更新函数签名和所有调用点。
- 在 `println!`、`eprintln!` 和 `format!` 等宏中优先使用内联格式参数（例如用 `eprintln!("{message}")`，不要用 `eprintln!("{}", message)`），以满足 Clippy 的 `uninlined_format_args` lint。
- 不要把 `Itertools::format` 的结果直接传给日志宏（`log::*`、`safe_*` 等）。`Itertools::format` 会生成一次性 formatter，而日志实现可能多次格式化同一条消息。日志参数应使用可复用的 `String`，例如 `iter.join(", ")`。直接用于 `format!` 或 `write!` 没问题。
- 做无关修改时不要删除已有注释。只有当注释描述的逻辑发生变化时，才修改或删除注释。
- 添加可切换设置时，也要添加对应的 Command Palette 启用/禁用入口和所需 context flag，确保该设置可在 Settings 之外被发现。

**Terminal Model 加锁**：

- 调用终端模型（`TerminalModel`）上的 `model.lock()` 时要极其谨慎。从不同调用点对同一 model 获取多个锁可能导致死锁，从而造成 UI 卡死（macOS 上出现 beach ball）。
- 添加新的 `model.lock()` 调用前，请确认当前调用栈中没有调用方已经持有该锁。
- 优先将已经锁定的 model 引用沿调用栈向下传递，而不是重新获取锁。
- 如果必须锁定 model，请让锁作用域尽可能短，并避免调用其他可能再次尝试加锁的函数。

**测试**：

- 使用 `cargo nextest` 进行并行测试执行
- 集成测试使用 `integration/` 中的自定义框架
- 提交前应通过 presubmit 脚本运行测试
- 单元测试应放在单独文件中，并使用 `${filename}_tests.rs` 或 `mod_test.rs` 命名约定
- 测试文件应在对应模块末尾这样引入：

```rust
#[cfg(test)]
#[path = "filename_tests.rs"]  // or "mod_test.rs"
mod tests;
```

**Pull Request 工作流**：

- 打开 PR 或向已有 PR 分支推送更新前，**始终**运行 `./script/format` 和 `cargo clippy`（使用 ./script/presubmit 中指定的版本）
- 创建或更新 pull request 前，这些命令必须完全通过
- 具体来说，确保 `./script/format` 和 `cargo clippy` 检查通过
- 如果它们失败，先修复所有问题，再继续 PR 流程
- 不要创建会披露非公开安全漏洞的公开 pull request 或公开 issue。请改为指引用户查看 `SECURITY.md` 中的正确披露方式。
- 这适用于：
  - 打开新的 pull request
  - 向已有 PR 分支推送新提交
  - 任何将进入审查的分支更新
- 打开 PR 时，使用 `.github/pull_request_template.md` 中的 PR 模板
- 适当时按 PR 模板底部格式添加 changelog 条目。使用以下前缀（不带 `{{}}` 括号）：
  - `CHANGELOG-NEW-FEATURE:` 用于新的、相对较大的功能（谨慎使用，这类条目可能进入市场/文档材料）
  - `CHANGELOG-IMPROVEMENT:` 用于现有功能的新能力
  - `CHANGELOG-BUG-FIX:` 用于与已知 bug 或回归相关的修复
  - `CHANGELOG-IMAGE:` 用于 GCP 托管图片 URL
  - 如果不需要 changelog 条目，请将 changelog 行留空或移除

**数据库**：

- 使用 Diesel ORM 和 SQLite
- migration 位于 `crates/persistence/migrations/`
- schema 定义在 `crates/persistence/src/schema.rs`

**GraphQL**：

- schema 和客户端代码由 `crates/warp_graphql_schema/api/schema.graphql` 生成
- 为前端集成生成 TypeScript 类型

### Feature Flag

Warp 使用编译期 feature flag，并配有一层小型运行时管线。

添加 feature flag 的方式：

- 在 `warp_core/src/features.rs` 的 `FeatureFlag` enum 中添加新变体
- （可选）将其列入 `DOGFOOD_FLAGS`，以便在 dogfood 构建中默认启用
- 使用 `FeatureFlag::YourFlag.is_enabled()` 对代码路径进行门控
- 对 preview 或 release rollout，视情况分别添加到 `PREVIEW_FLAGS` 或 `RELEASE_FLAGS`

最佳实践：

- **优先使用运行时检查而不是 cfg 指令**：优先使用 `FeatureFlag::YourFlag.is_enabled()`，不要优先使用 `#[cfg(...)]` 编译期指令，这样 flag 可以在无需重新编译的情况下切换，后续也更容易清理。仅在没有该 flag 代码就无法编译时才使用 `#[cfg(...)]`（例如平台相关代码，或 feature 禁用时不存在的依赖）。
- 保持 flag 处于高层级并面向产品，不要按每个调用点创建 flag
- 功能发布稳定后移除 flag 和失效分支
- 对暴露新功能的 UI section，使用同一个 flag 隐藏 UI

示例：

```rust
#[derive(Sequence)]
pub enum FeatureFlag {
    YourNewFeature,
}

// dogfood 构建默认启用
pub const DOGFOOD_FLAGS: &[FeatureFlag] = &[
    FeatureFlag::YourNewFeature,
];

// 在代码中使用
if FeatureFlag::YourNewFeature.is_enabled() {
    // 受门控的行为
}
```

### 穷尽匹配

添加或编辑 match 语句时，尽可能避免使用通配符 `_`。穷尽匹配有助于确保所有变体都被处理，尤其是在未来向 enum 添加新变体时。
