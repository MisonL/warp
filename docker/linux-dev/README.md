# 用于 Linux 开发的 Dockerfile

本目录中的 Dockerfile 定义了一个 container，其中安装了快速开始在 Linux 上构建 Warp 所需的全部工具。

该 container 基于 Debian Sid，也就是 Debian 的 unstable 分支。它确保你运行的是 `mesa` 等组件的最新版本（`mesa` 是一个开源 3D 图形库，提供 OpenGL 和 Vulkan 实现）。

## 前置条件

你需要安装：
* Docker（例如 Docker Desktop）
* XQuartz（从[这里](https://www.xquartz.org/)下载）
  * 为了正确渲染，你需要运行 `defaults write org.xquartz.X11 enable_iglx -bool true` 启用 iGLX（indirect GL extensions）；可以在安装 XQuartz 前执行。
  * 安装 XQuartz 后，运行它，并在设置的 Security 标签中启用 "Allow connections from network clients"。完成此变更后，需要退出并重新启动 XQuartz。

## 设置

以下所有命令都应从仓库根目录运行。

首先，构建 docker container image：

```
CONTAINER_NAME="warp-client-linux-dev"
docker build -t $CONTAINER_NAME docker/linux-dev
```

接着，运行 container：

```
# 要挂载到 container 中的源代码目录路径。它可以是 `warp`
# 仓库，也可以是你选择的某个父目录。
LOCAL_PATH="/Users/$USER/src"

# 将 image 作为 container 运行，为 SSH 连接桥接端口 22，
# 将给定目录挂载到 container 的 `/src`，将你的 SSH key
# 目录挂载到 container 中（这样不需要创建新的 GitHub SSH key），
# 并挂载 gcloud 配置（以及认证信息，以便运行 SSH 集成测试）。
docker run -dp 127.0.0.1:22:22/tcp -v $LOCAL_PATH:/src -v $HOME/.ssh:/home/dev/.ssh -v $HOME/.config/gcloud:/home/dev/.config/gcloud $CONTAINER_NAME
```

## 用法

每次启动 XQuartz 后，都需要运行一次以下命令，以便 container 中运行的程序可以连接到它：

```
xhost +localhost
```

你应该可以 SSH 到 container 中，并在无需额外设置的情况下构建和运行 Warp（dev 账户密码是 "password"）：

```
ssh dev@localhost
cd /src
cargo run --features fast_dev
```

尝试编译 Warp 时，可能会遇到一些奇怪错误；如果遇到，只需继续重新运行 cargo 命令，最终应该可以成功。
