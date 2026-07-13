---
name: modify-settings
description: 借助内置 JSON Schema 查看或修改 Warp 应用设置。
---

# modify-settings

当用户想查看、修改或排查 Warp 应用设置时使用此技能。

## Settings Schema

描述所有可用设置的 JSON Schema 内置在：

```sh
{{settings_schema_path}}
```

该 Schema 遵循 JSON Schema draft 2020-12，设置按 `properties` 分层组织。每个设置包含：

- **`description`**：设置控制的内容
- **`type`**：值类型，例如 `string`、`boolean`、`integer`
- **`default`**：默认值
- **`enum`** 或 **`oneOf`**：有约束时的有效值

### 查找设置

先用 `grep` 对候选 key 做宽泛搜索：

```sh
grep -i "font" {{settings_schema_path}}
```

找到候选键后，运行内置脚本获取完整点分路径、设置属性和父级上下文。这一点很关键，因为 Schema 中可能有多个相似键，例如多个 `input`。不要只根据 grep 输出推断嵌套层级。

```sh
python3 <skill_dir>/scripts/find_setting.py {{settings_schema_path}} <key_name>
```

输出会给出无歧义的完整路径，例如 `properties.appearance.properties.input.properties.input_mode`，以及该设置的完整定义和有效值。

## Settings File

用户设置存储在 TOML 文件中：

```sh
{{settings_file_path}}
```

设置使用点分 TOML 段落标题，对应 Schema 层级。始终从 Schema 追踪完整嵌套路径到 TOML；每个中间 `properties` 都对应一层段落。例如：

`properties.appearance.properties.font_size` 对应：

```toml
[appearance]
font_size = 14
```

`properties.appearance.properties.themes.properties.theme` 对应：

```toml
[appearance.themes]
theme = "light"
```

常见错误是少算一层。写 TOML 段落标题前必须数清完整深度。

如果设置文件还不存在，则创建它。Warp 会热重载此文件，改动会立即生效。

## 工作流

1. **查找设置**：用 `grep` 找候选键，再运行 Python 路径追踪脚本获取完整点分路径和有效值。不要只依赖 grep 输出推断嵌套。
2. **读取当前值**：检查 settings 文件中该设置是否已配置。
3. **应用改动**：用 Schema 中的有效值在 TOML 文件中新增或更新设置。
