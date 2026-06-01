# MacOS 用户通知

我们用于支持通知的 Apple framework 是 `UNUserNotifications`：https://developer.apple.com/documentation/usernotifications?language=objc

## 本地开发通知

该 framework 需要 signed app，才能向 Apple Notification Center 请求授权并安排通知。因此，只执行 `cargo build && cargo run` 并不够。可选方式有以下几种：

### 1. Bundle 应用

这比选项 2 更耗时，但更稳定，也更接近用户最终体验。

1. 运行 `script/user_notifications --nouniversal --open`

如果要专门测试授权流程，需要：
1. 删除本地安装的所有 `WarpDev` app
2. 在 System Preferences 的 `Notifications` 中检查显示的 app，确保 `WarpDev` 不是 Notification Center 中的 app。
3. 登出*
4. 重新登录，并再次 bundle&run 应用。

### 2. 对本地 build 做 nosign（script）

这比选项 1 不稳定，如果要测试授权流程，不建议使用。

1. 确保已经安装 WarpDev app，且它位于 Applications folder 中。该 app 必须位于 Applications 中，否则 Apple 在测试通知时找不到该 app。
2. 运行 `script/local_build_and_sign`

如果要专门测试授权流程，需要：
1. 删除本地安装的所有 `WarpDev` app，包括 Applications folder 中的那个。
2. 在 System Preferences 的 `Notifications` 中检查显示的 app，确保 `WarpDev` 不是 Notification Center 中的 app。
3. 登出*并重新登录。
4. 将 `WarpDev` 从 `Bin` 移到 `Applications`（也就是安装 `WarpDev`）。
5. 再次运行脚本以 nosign 并运行 app：`script/local_build_and_sign`。

*注意：如果你已经拥有发送通知的全部权限，且不是在测试授权流程，可以执行以下步骤替代登出再登录：
1. 删除所有 `WarpDev` app。
2. 运行 `sudo lsof | grep usernoted | grep db2`，找到 Notification Center 使用的 database 路径。
3. 运行 `killall usernoted && killall NotificationCenter`
4. 运行 `rm <path-to-notification-center-db>`
5. 重新构建并运行 app（如果不 bundle，仍然必须将 `WarpDev` 移回 `Applications` folder）。

## 调试通知

调试错误或行为不符合预期时，一些有用方法包括：
- 检查 Notification Center，确认 `WarpDev` 是否为已注册 app，并尝试不同设置（例如打开/关闭、启用声音等）
- 在本地 build 中使用 `NSLog` 打印 debug statement
- 使用 Console app 查看更有帮助的 framework error。当你自己的日志中看不到任何 error 时，这尤其有用。用 `NotificationCenter`、`usernoted` 或 `dev` 过滤消息
- 如果不确定，删除所有 `WarpDev` app 并重启电脑。有时 Notification Center 需要轻推一下
