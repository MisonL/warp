---
name: warpctrl
description: Control and inspect the currently running local Warp application with the warpctrl CLI. Use this skill whenever the user asks the agent to manipulate Warp's own windows, tabs, panes, sessions, input buffer, themes, or UI surfaces; open a file in Warp; inspect local Warp state; or explain how to invoke Warp Control manually.
description_zh_CN: 使用 warpctrl CLI 控制和检查当前正在运行的本地 Warp 应用。用户要操作 Warp 自身的窗口、标签页、窗格、会话、输入框、主题或 UI 表面；在 Warp 中打开文件；检查本地 Warp 状态；或说明如何手动调用 Warp Control 时使用此 skill。
---

# Warp Control

使用 `{{warpctrl_binary_name}}` 检查或控制提供此 skill 的、已经运行的本地 Warp 应用。本 skill 中的命令名和 wrapper 路径会按当前 Warp 渠道注入，因此不要检查运行中的进程，也不要猜测当前活动渠道。

当请求要改变 Warp 自身，而不是用户项目或操作系统时，优先使用 `{{warpctrl_binary_name}}`。例如创建 Warp 标签页、拆分窗格、在 Warp 输入框中暂存文本、打开 Warp 设置，或聚焦 Warp 窗口。

## 如何调用 Warp Control

Warp Control 会随 Warp 应用一起打包。它不是单独的独立二进制文件，而是由正在运行的 Warp 进程提供的隐藏控制模式。

- 当前 Warp 渠道的命令：`{{warpctrl_binary_name}}`
- 当前 Warp 渠道打包的 wrapper：`{{warpctrl_wrapper_path}}`
- 可选 PATH 符号链接：`/usr/local/bin/{{warpctrl_binary_name}}`

### 确认命令可用

首次在任务中调用 Warp Control 前，优先使用最短可用路径，并避免不必要的安装研究：

1. 如果 `command -v {{warpctrl_binary_name}}` 成功，后续任务都使用 `{{warpctrl_binary_name}}`。除非后续命令失败，否则不要检查打包 wrapper 或验证符号链接。
2. 如果 `command -v {{warpctrl_binary_name}}` 失败，验证 `{{warpctrl_wrapper_path}}` 存在且可执行。如果它缺失，告诉用户此 Warp 构建不包含预期 wrapper，然后停止。
3. 检查 `/usr/local/bin/{{warpctrl_binary_name}}`。只有当它是一个符号链接，且解析结果精确指向 `{{warpctrl_wrapper_path}}` 打包 wrapper 时，才视为设置完成。
4. 如果预期符号链接缺失、损坏或指向别处，使用 `ask_user_question` 工具询问用户是否要安装它：在 `/usr/local/bin/{{warpctrl_binary_name}}` 创建指向 `{{warpctrl_wrapper_path}}` 的链接。提供 **Install command** 作为推荐选项，**Not now** 作为备选项。没有明确同意时，不要创建或替换符号链接。
5. 用户同意后，只创建或更新这个预期符号链接：运行 `ln -sf "{{warpctrl_wrapper_path}}" "/usr/local/bin/{{warpctrl_binary_name}}"`。先尝试不提权执行。如果 macOS 权限阻止修改，再通过 `osascript` 以管理员权限运行同一命令；不要直接索要或暴露用户密码。
6. 用 `command -v {{warpctrl_binary_name}}`、`readlink /usr/local/bin/{{warpctrl_binary_name}}` 和 `{{warpctrl_binary_name}} app version` 验证结果。

如果用户选择 **Not now**，不要创建符号链接。本次任务直接使用 `{{warpctrl_wrapper_path}}` 打包 wrapper。

Warp UI 也在 Command Palette 中提供 **Install Warp Control CLI command** 和 **Uninstall Warp Control CLI command**，并在 **Settings > Scripting** 下提供安装控制项。

## 工作流

始终优先从 `{{warpctrl_binary_name}}` 自身发现命令，不要猜测或编造命令。CLI 的完整帮助和 action catalog 是已安装构建支持哪些功能的权威来源。

### 串行执行并验证结果

串行运行 Warp Control 命令。即使命令看似互不相关，也不要通过并行 shell 工具调用同时派发多个 `{{warpctrl_binary_name}}` 命令。它们作用于同一个正在运行的应用，可能改变活动目标或终端上下文，从而影响后续命令的执行和观察。多步骤请求优先使用一个 shell 工具调用顺序执行命令，或逐条发出单独 shell 工具调用。

执行会创建、激活、导航或聚焦窗口、标签页、窗格、会话或表面的动作后，不要假设活动目标仍然不变。后续命令在需要精确目标时使用显式 selector，或先重新运行 `{{warpctrl_binary_name}} app active`。

验证每个结果都对应刚刚调用的命令。如果输出描述了不同动作、报告了意外实例或渠道，或与请求冲突，停止并串行重新运行 `{{warpctrl_binary_name}} instance list` 后再重试。只要有对应的 `list`、`inspect` 或 `get` 命令可用，就必须验证用户请求的最终状态后再报告成功。

### 按意图路由

发现命令前，把请求路由到最窄匹配的顶层分组：

1. 打开、显示、查看或切换某个 Warp UI destination、panel、picker 或 settings page 的请求使用 `surface`。把自然语言名称转换为 kebab case，例如把 "Warp Drive" 转为 `warp-drive`，把 "code review" 转为 `code-review`。当请求的最终状态是打开时，优先使用 `surface <name> open`。如果 destination 或支持的 verb 不明确，使用 `surface list` 或 `surface help`。不要为 UI destination 推断内部 action 名称。
2. 关于窗口、标签页、窗格或会话的请求使用对应的 `window`、`tab`、`pane` 或 `session` 分组。
3. 暂存或检查编辑器输入的请求使用 `input`。
4. 在 Warp 中打开文件的请求使用 `file`。
5. 关于主题、外观、设置或快捷键的请求使用对应的 `theme`、`appearance`、`setting` 或 `keybinding` 分组。
6. 只有在没有专用 CLI 分组匹配时，才使用通用 `action` catalog。内部 action 名称或 catalog action 名称不保证可作为独立 parser 命令调用。

1. 从当前 Warp 渠道发现正在运行的 Warp 实例：

   ```sh
   {{warpctrl_binary_name}} instance list
   ```

2. 如果只有一个同渠道实例在运行，命令会自动选择它。如果有多个同渠道实例在运行，使用 `--instance <instance_id>` 或 `--pid <pid>` 显式选择一个。

3. 从已路由的分组发现准确命令和参数，不要猜测。这是当前命令面支持情况的首选事实来源：

   ```sh
   {{warpctrl_binary_name}} help
   {{warpctrl_binary_name}} <group> help
   {{warpctrl_binary_name}} <group> <command> --help
   ```

   只有当没有专用分组匹配时，才检查通用 action catalog：

   ```sh
   {{warpctrl_binary_name}} action list
   {{warpctrl_binary_name}} action inspect <action.name>
   ```

4. 在修改前检查当前活动目标链，或列出相关目标：

   ```sh
   {{warpctrl_binary_name}} app active
   {{warpctrl_binary_name}} window list
   {{warpctrl_binary_name}} tab list
   {{warpctrl_binary_name}} pane list
   {{warpctrl_binary_name}} session list
   ```

5. 调用满足请求的最窄动作；有用时再用对应的 `list`、`inspect` 或 `get` 命令验证结果。

## 常用动作

这些是常用且可直接调用的命令。较少使用的命令请按意图路由，并用 `{{warpctrl_binary_name}} <group> help` 或 `{{warpctrl_binary_name}} <group> <command> --help` 发现运行中构建支持的准确语法。只有在没有专用分组匹配时，才检查通用 action catalog。

```sh
# 创建和管理标签页与窗格
{{warpctrl_binary_name}} tab create
{{warpctrl_binary_name}} tab create --type agent
{{warpctrl_binary_name}} tab rename "server logs"
{{warpctrl_binary_name}} pane split --direction right
{{warpctrl_binary_name}} pane navigate --direction next

# 在 Warp 输入框中暂存文本，但不提交执行
{{warpctrl_binary_name}} input insert "git status"
{{warpctrl_binary_name}} input replace "cargo test"

# 打开或切换 Warp UI 表面
{{warpctrl_binary_name}} surface list
{{warpctrl_binary_name}} surface settings open
{{warpctrl_binary_name}} surface command-palette open --query "theme"
{{warpctrl_binary_name}} surface command-search open
{{warpctrl_binary_name}} surface theme-picker open
{{warpctrl_binary_name}} surface keybindings open
{{warpctrl_binary_name}} surface warp-drive open
{{warpctrl_binary_name}} surface resource-center toggle
{{warpctrl_binary_name}} surface ai-assistant toggle
{{warpctrl_binary_name}} surface project-explorer open
{{warpctrl_binary_name}} surface global-search open
{{warpctrl_binary_name}} surface conversation-list open
{{warpctrl_binary_name}} surface code-review open
{{warpctrl_binary_name}} surface left-panel toggle
{{warpctrl_binary_name}} surface right-panel toggle
{{warpctrl_binary_name}} surface vertical-tabs open
{{warpctrl_binary_name}} surface agent-management open

# 在 Warp 中打开文件
{{warpctrl_binary_name}} file open ./src/main.rs --line 42

# 检查和更新支持的状态
{{warpctrl_binary_name}} theme get
{{warpctrl_binary_name}} theme set "Dracula"
{{warpctrl_binary_name}} appearance get
{{warpctrl_binary_name}} setting list
{{warpctrl_binary_name}} keybinding list
```

当结构化输出更便于消费时，添加 `--output-format json`：

```sh
{{warpctrl_binary_name}} --output-format json tab list
```

## 目标选择

当动作支持相应作用域时，target selector 可以组合使用：

- 实例：`--instance <instance_id>` 或 `--pid <pid>`
- 窗口：`--window <id>`、`--window-index <n>` 或 `--window-title <exact-title>`
- 标签页：`--tab <id>`、`--tab-index <n>` 或 `--tab-title <exact-title>`
- 窗格：`--pane <id>` 或 `--pane-index <n>`
- 会话：`--session <id>`

需要精确目标时，使用 `list`、`inspect` 或 `app active` 返回的 ID。如果省略 selector，大多数 scoped action 会作用于活动目标。当多个目标可能合理匹配用户请求时，优先使用显式 selector。

在 walkthrough 或多步骤 UI 工作流前使用 `surface list`。它会用稳定名称报告可用和不可用的 destination 及原因。直接的 `surface ... open` 命令是幂等的；当最终状态必须为打开时，使用它们而不是 toggle 命令。`surface list` 接受 `--instance` 或 `--pid` 选择进程，但拒绝 window、tab、pane 和 session selector。

## 安全与限制

- 只有当用户明确要求关闭某个对象时，才调用 close 动作。close 动作会经过 Warp 的正常关闭流程，并可能触发现有应用警告。
- `input insert` 和 `input replace` 只会暂存文本。Warp Control 有意不提供提交或运行输入的动作。
- 不要编造不支持的命令。先使用匹配分组的 `help`，只有在没有专用分组匹配时，才使用 `action list` 或 `action inspect`。
- Warp Control 只影响同一用户拥有的、正在运行的本地 Warp 应用。它不能控制远程或云端 Warp 实例。
- 每个渠道专属的 Warp Control CLI 只会列出并操作同一渠道的 Warp 实例。
- 在 Windows 上，本地控制发布会保持禁用，直到支持已认证的 broker transport。

## 手动设置和故障排查

Warp Control 可用性取决于构建渠道和 **Settings > Scripting** 开关。本地控制模式在内部 dogfood 构建（例如 WarpDev）默认启用，在公开渠道（Stable、Preview、OSS）默认禁用。任何渠道上的最终门禁都是 **Settings > Scripting** 开关。已安装的 `{{warpctrl_binary_name}}` wrapper 会调用匹配渠道的 Warp 可执行文件。

如果 `{{warpctrl_binary_name}} instance list` 为空，确认兼容的同渠道 Warp 应用正在运行，并且 Scripting 已启用。如果命令报告多个实例，用 `--instance <instance_id>` 重新运行。

如果符号链接不在 `PATH` 上，按 **如何调用 Warp Control** 中的确认门控流程处理，或直接使用 `{{warpctrl_wrapper_path}}`。
