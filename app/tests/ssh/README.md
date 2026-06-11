# 通过 SSH 测试 Warp 功能

## 前置条件：安装 Docker

1. `brew install --cask docker`
2. 打开 Docker desktop 应用。这一步是必要的，因为它会创建让 `docker` CLI 可用的符号链接。

## 通过 SSH 运行 Warp

本仓库中有一个名为 "Build image and start container for SSH testing" 的 workflow。执行后，你可以通过运行 bash@0.0.0.0 或 zsh@0.0.0.0 进行 SSH 登录。它会提示输入密码，这些 VM 的密码是 `password`。

构建 image 后，你可以直接使用 workflow 中的第二个命令再次启动 container。

注意，系统中只能有一个占用端口 22 的 docker container。

## 更高级的用法

有时 SSH 用户的问题来自 SSH server 配置，通常是 `/etc/ssh/sshd_config`（本仓库 Dockerfile 使用的 Ubuntu 中也是如此）。如果需要编辑它，之后必须用 `sudo service ssh restart` 重启 SSH daemon 才能让变更生效。这会导致 container 停止，因此你需要在 Docker Desktop 应用中重启它（点击播放按钮），以便用新配置重新启动。
