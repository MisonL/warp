为 Warp 实现 dock tile plugin。

该 plugin 用于在应用图标变更时更新 dock tile icon，并允许应用图标变更在应用重启后保持。没有该 plugin 时，应用第一次退出后图标状态会恢复为默认值（不过出于某些原因，第一次退出之后图标状态又会被保留）。

该 plugin 使用 Objective-C 实现，并通过 `clang` 编译器配合 `-bundle` flag 构建。更多详情见 Makefile。

该 plugin 会安装到 app bundle 的 `Contents/PlugIns/WarpDockTilePlugin.docktileplugin`，并通过 script/mac/bundle 脚本打包。它会为 arm64 和 x86_64 构建 universal binary。

该 plugin 是一个简单的 Objective-C 程序，会监听 main application 在应用图标变更时发出的 notification。收到 notification 后，它会更新 dock tile icon。

更多详情见 Mac 文档：
https://developer.apple.com/documentation/appkit/nsdocktileplugin?language=objc

注意，在开发期间，MacOS 对变更后重新加载 plugin 的支持并不好。

重建后的建议流程是：
1. 从 dock 中移除图标。
2. 运行 `killall Dock && killall SystemUIServer`

另外，[MacDockTileSample](https://github.com/CartBlanche/MacDockTileSample) 中有一个示例 plugin，对安装和迭代很有帮助。

另一个技巧是添加基于文件的 debug 日志，而不是使用 NSLog，因为无法找到 NSLog 被写到哪里。
