# Inno Setup installer 脚本

## 什么是 `windows-installer.iss`？

在 Windows 上，程序通常使用 installer 安装，也叫安装向导。
Installer 是一个单独的可执行文件，负责：
* 创建用于存放程序文件的目录
* 下载 asset
* 初始化 registry entry
* 创建桌面图标
* 根据应用需求执行更多操作

`windows-installer.iss` 是一个 **Inno Setup script**：
用于构建 Warp installer 的配置文件。
Inno Setup Compiler 会读取脚本文件并生成 installer 可执行文件。
这大致相当于 MacOS 上的 bundling 流程。

## 如何编辑 installer

请参考 Inno Setup 文档：[Inno Setup Help](https://jrsoftware.org/ishelp/)。
可以使用任何代码编辑器手动编辑该脚本。
不过，它需要 Inno Setup compiler 才能转换成 `.exe` 文件。

## 如何编译此 installer

首先，确保你已经设置好环境。
* 下载并安装 [Inno Setup Compiler](https://jrsoftware.org/isdl.php)。
* 运行 `cargo build`，确保 installer 使用最新版本的 Warp。

### 选项 1：使用 CLI

1. 将 Inno Setup Command-line Compiler 可执行文件加入 shell path。
默认情况下，它位于 `C:\Program Files (x86)\Inno Setup 6\ISCC.exe`。
2. 编译 installer：
```shell
iscc .\script\windows\windows-installer.iss
```
3. 运行生成的可执行文件：
```shell
.\script\windows\Output\Warp-Windows-Setup.exe
```

该脚本以一组 preprocessor definition 开头。
从命令行使用 `/D` flag 可以模拟 preprocessor definition
并覆盖硬编码默认值。
用法：`iscc <script path> /D<name>[=<value>]`

可覆盖以下常量：
* `MyAppVersion`（默认：`0.1.0`）
* `MyAppExeName`（默认：`warp.exe`）
* `ReleaseChannel`（默认：`dev`）
* `TargetProfileDir`（默认：`debug`）

### 选项 2：使用 GUI

1. 打开 Inno Setup 应用并选择此脚本。
2. 点击 "compile" 按钮。这会在与该脚本同级的 `Output` 目录中生成 installer 可执行文件。
2. 要运行 installer，请点击 Inno Setup 中的 "run" 按钮。

## 使用图标

Windows 有自己的 icon 文件格式，可以将多个 icon size 打包在一起。
App icon 位于 `app/channels/<channel_name>/icon/no-padding`。
`.ico` 文件使用 imagemagick 生成：

```shell
convert 16x16.png 32x32.png 48x48.png 64x64.png 256x256.png icon.ico
```

注意，不支持超过 256x256 的尺寸。
请参见 [Inno Setup 文档](https://jrsoftware.org/ishelp/index.php?topic=setup_setupiconfile)。
