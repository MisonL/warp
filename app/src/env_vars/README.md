# 环境变量文档

本文档提供关于我们“Environment Variables”功能的信息。在内部，我们将这些对象称为 `EnvVarCollection`（EVC）。绑定到该对象的 view 通常会使用上面的字符串称呼，而函数和变量通常命名为 `env_var_collection`。

本文档截至 2024-06-26 保持最新。除非另有说明，所有引用文件都位于本目录中。

## 核心数据模型

EVC 的核心数据模型定义在 `mod.rs` 中。数据模型背后的动机在上面的文档中有详细说明，其中 v1 tech doc 最相关。

## 云基础设施

背景：EVC 构建在 GenericStringObject（GSO）之上。因此，EVC 没有太多专用的服务端基础设施。我们在服务端的 `Format` enum 中添加了一个 variant，并在客户端也添加了对应项（`JsonObjectType::EnvVarCollection`），再通过一个小型 DB migration 支持该类型。

我们在 `mod.rs` 中定义了 `CloudEnvVarCollection`，它实现 `GenericCloudObjectType` trait。这基本是一个样板实现，用来指定 EVC 应在 Warp Drive 中渲染、可链接/可导出等属性。

EVC 作为 Warp Drive object 的实现位于 `app/src/drive/items/env_var_collection.rs`，其中包含 Warp Drive preview 和点击 action 的代码。

与编辑冲突和从服务端获取 EVC 相关的代码位于 `app/src/server/server_api.rs` 和 `app/src/server/cloud_objects/update_manager.rs`。我们的目标是保持与 workflow 类似的 liveness 属性，也就是说，如果另一个用户进行了并发编辑，当前用户在提交自己的编辑前需要先检查对方的编辑。

## 客户端侧

### Pane

EVC 和 Warp 中的大多数对象一样，是 pane 的子对象。我们的实现定义在 `app/src/pane_group/pane/env_var_collection_pane.rs` 中，基本与其他 pane 实现一致。`EnvVarCollectionPane` 与 `manager.rs` 中定义的 `EnvVarCollectionManager` 紧密耦合。manager 负责创建、销毁和注册所有 EVC pane，而 pane 本身包含 EVC view。

### 核心 UI

我们会按重要性顺序逐个说明 view 目录中每个文件的核心 UI 组件。

- `env_var_collection.rs`：包含 `EnvVarCollectionView` 的核心函数和实现。像 "open_new_env_var_collection" 和 "load" 这样的函数（用于加载现有 EVC，或在冲突后重新加载已打开的 EVC）都有说明其相关性的文档。
- `secrets.rs`：见下方单独章节，因为这是关键流程。
- `command_dialog`
    - `command_dialog_view.rs`：定义 command dialog 的 view。
    - `mod.rs`：包含与 command dialog 相关的功能，也就是监听来自 dialog 的事件。
- `unsaved_changes_dialog.rs`：包含当用户尝试在未保存变更时关闭 pane 所展示 dialog 的相关代码。
- `menus.rs`：定义 EVC 的 menu 相关代码。这包括 secret menu（与钥匙图标或已渲染 secret/command 关联）以及 pane-bound menu（包含对象特定 action 的 overflow menu，以及右键触发的、包含 split pane action 的 context menu）。
- `editors.rs`：定义初始化 editor、处理其事件（例如 tab navigation）以及渲染 "metadata" section 的代码。
- `fixed_view_components.rs`：包含组件渲染函数，例如 trash overflow banner 或 footer 中的 save button。
- `active_env_var_collection_data.rs`：跟踪当前打开的 EVC，包括当前 revision 和保存状态。

### Secret

Secret 初始化最适合通过完整流程说明：

1. 用户点击与某一行关联的 menu（钥匙图标或已渲染 secret/command），派发 `DisplaySecretMenu(VariableRowIndex)` action。
2. 该 action 被处理，并将 `VariableRowIndex` 存储在 `pending_variable_row_index` 状态变量中。
3. 用户选择 menu item（例如 1password），触发 `SelectSecretManager` action，该 action 解析到 `fetch_secret` 函数。
4. 在 `fetch_secret` 中，会发生以下事情：
    1. 获取用户本地 shell 的数据，以运行获取该用户全部 secret 的命令。
    2. 在后台线程中，执行 `app/src/external_secrets/mod.rs` 中的 `verify_installed_and_fetch_secrets` 函数。该函数会检查所选 secret manager 是否已安装，并尝试使用前面提到且文档完善的 local_shell module 获取 secret。如果任一操作失败，`fetch_secret` 会显示 error toast。
5. 假设 secret 获取成功，它们会被发送到可搜索的 secrets dialog（位于 `app/src/search/external_secrets`），该 dialog 会向 EVC view 传播事件，表示应该打开 dialog。
6. 用户选择一个 secret 后，事件会传播到 EVC view，后者将该 secret 存储到 `pending_variable_row_index` 指向的 `VariableEditorRow` 的 value 字段中，并关闭 dialog。

### 其他

- Workflow card（parameterized workflow）中 EVC 部分的代码定义在 `app/src/workflows/info_box.rs`。
- 与 command palette 和 search 功能相关的代码位于 `app/src/search` 下各自对应目录中。
- 调用前附加到 blocklist 的 EVC block 代码位于 `env_var_collection_block.rs`。设置/初始化变量的命令在 `mod.rs` 中建立。调用 EVC 的代码路径位于 `app/src/terminal/view.rs` 的 `invoke_environment_variables`。
