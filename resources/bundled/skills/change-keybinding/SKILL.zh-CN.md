---
name: change-keybinding
description: 通过编辑用户的 keybindings.yaml 文件来自定义 Warp 键盘快捷键。适用于用户想重映射按键组合、重新绑定操作、修改快捷键或移除默认按键绑定时。
---

# change-keybinding

当用户想重映射、重新绑定或移除 Warp 键盘快捷键时使用此技能。

## Keybindings 文件

用户自定义内容位于：

```sh
{{keybindings_file_path}}
```

这是 Warp 启动时读取的精确路径，且与平台和 channel 相关。例如 macOS 可能位于 `~/.warp*/`，Linux 可能位于 `~/.config/warp-terminal/` 等 XDG config 目录，Windows 可能位于 `%LocalAppData%`。必须原样使用该路径，不要根据用户 home 目录结构推断其他路径。如果文件不存在，创建文件及缺失的父目录。

## 文件格式

文件是 `action_name` 到 `key_trigger` 的扁平 YAML map。Action name 含冒号，因此**必须加引号**：

```yaml
"workspace:toggle_ai_assistant": ctrl-s
"editor_view:delete_all_left": cmd-shift-A
"workspace:toggle_command_palette": none
```

## 按键编码规则

Trigger 使用 Warp 的标准化格式。必须完全正确，否则绑定会静默加载失败。

- **修饰键**：组合时按此顺序：`ctrl-alt-shift-cmd-meta-`。跨平台 alias：`cmdorctrl-`，macOS 上变成 `cmd`，其他平台变成 `ctrl`。
- **字母大小写**：仅适用于单字母按键。没有 `shift` 时字母小写，例如 `ctrl-s`。有 `shift` 时字母必须大写，例如 `shift-A`，不能写 `shift-a`。
- **特殊按键**：`space`、`enter`、`escape`、`tab`、`backspace`、`delete`、`insert`、`up`、`down`、`left`、`right`、`home`、`end`、`pageup`、`pagedown`、`f1` 到 `f20`、`numpadenter`。即使带 `shift` 也始终小写，例如 `ctrl-shift-space`、`shift-tab`。空格使用字面词 `space`，不要使用 `" "`。
- **标点**：直接使用字符，例如 `cmd-=`、`cmd-,`、`cmdorctrl-/`。
- **移除默认绑定**：把值设为字面字符串 `none`。该 action 会变为未绑定。

把用户说法转换为此格式：`Ctrl+S` 转为 `ctrl-s`，`Cmd+Shift+P` 转为 `cmd-shift-P`，`Ctrl+Space` 转为 `ctrl-space`。

## 识别 action

默认绑定编译进 Warp，不能从磁盘上的 keybindings 文件发现。没有可供 agent 查询的目录能把描述或当前快捷键映射到 action name。根据用户描述选择策略：

1. **用户给出 action name**，例如 “set workspace:toggle_command_palette to cmd-p”：直接写入。
2. **用户只给描述或当前按键组合**，例如 “rebind the command palette to cmd-p” 或 “change ctrl+space to ctrl+s”：你没有 action name，不能可靠猜测。不要编造。引导用户打开**快捷键编辑器**（`workspace:show_keybinding_settings`，macOS 默认 `cmd-ctrl-k`；其他平台为 Settings -> Keyboard Shortcuts）。用户可按描述或当前快捷键搜索并直接编辑，或提供规范的 `namespace:action_name` 后再由你写文件。

## 工作流

1. 确定要重映射的 action 和新 trigger，参考“识别 action”。
2. 如果 `{{keybindings_file_path}}` 已存在，先读取它。**保留所有已有条目**，只新增或更新目标条目。
3. 写入文件，必要时创建父目录。确保 action key 加引号，value 使用标准化按键格式。
4. 告诉用户 **Warp 必须重启**后改动才生效。keybindings 文件只在 app 启动时加载，不像 `settings.toml` 会热重载。用户可用 `cmd-Q`（macOS）或对应平台方式退出并重新打开 Warp。

## 示例

按旧 trigger 修改已有自定义绑定：

```yaml
# 修改前
"workspace:toggle_ai_assistant": ctrl-space
# 修改后
"workspace:toggle_ai_assistant": ctrl-s
```

移除默认快捷键：

```yaml
"workspace:toggle_keybindings_page": none
```

`shift` 与特殊键组合时，特殊键保持小写：

```yaml
"workspace:toggle_ai_assistant": ctrl-shift-space
```

使用 `cmdorctrl-` alias 的跨平台绑定：

```yaml
"workspace:toggle_command_palette": cmdorctrl-shift-P
```
