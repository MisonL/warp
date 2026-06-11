# Changelog Draft
**Channel:** stable
**Range:** v0.2026.04.29.08.56.stable_00 → v0.2026.05.06.09.12.stable_00
**Generated:** 2026-05-06T19:00:00Z
**Total PRs in range:** 211 | **Explicit markers:** 57 | **Unmarked:** 154

---

## New Features
- 现在可以像 Chrome 一样，将 tab 从一个 window 拖出到独立 window，或在 window 之间拖动。([#9275](https://github.com/warpdotdev/warp/pull/9275))
- 新增 `/set-tab-color` slash command，可从 input bar 设置或清除当前 tab 的颜色。([#9305](https://github.com/warpdotdev/warp/pull/9305))

## Improvements
- 新增 tab context menu action，可在可用时复制可见 tab 和 pane metadata。([#10120](https://github.com/warpdotdev/warp/pull/10120))
- Conversation details panel 现在可以通过可配置 keyboard shortcut 打开和关闭。([#9837](https://github.com/warpdotdev/warp/pull/9837))
- Conversation details side panel 现在也可用于本地 Warp Agent conversation，不再只用于 cloud Oz run。点击 pane header 中的 info button，即可为任何 active AI conversation 打开它。([#9493](https://github.com/warpdotdev/warp/pull/9493))
- 在 conversation streaming 期间，降低 agent runs management view 的内存占用和 CPU 工作量。([#9866](https://github.com/warpdotdev/warp/pull/9866))
- 新增将 image file 拖放到 active CLI agent session（例如 Claude Code）的支持。([#9553](https://github.com/warpdotdev/warp/pull/9553))
- Warp 现在会在 agent block output 中渲染 inline local image 和 Mermaid diagram。([#9993](https://github.com/warpdotdev/warp/pull/9993))
- 当 remote host 上预构建 remote-server binary 不兼容（例如 glibc < 2.31）时，Warp 现在会静默 fallback 到常规 SSH session，而不是尝试会在运行时失败的安装。([#9681](https://github.com/warpdotdev/warp/pull/9681))
- 使用 .htm extension 的 HTML 文件现在会在 Warp editor 中以 HTML syntax highlighting 打开。([#9360](https://github.com/warpdotdev/warp/pull/9360))
- 识别 Block 的 `goose` CLI agent：运行 `goose` 现在会像其他已识别的第三方 agent 一样，激活 CLI-agent toolbar、status、brand color 和 icon。([#9497](https://github.com/warpdotdev/warp/pull/9497))
- 新增 `/continue-locally` slash command，可在本地继续 cloud conversation。([#9500](https://github.com/warpdotdev/warp/pull/9500))
- 点击检测到的 file link 时出现的 tooltip 中，新增 "Show in Finder"（macOS）/ "Show containing folder"（Linux/Windows）选项。([#9475](https://github.com/warpdotdev/warp/pull/9475))
- 收紧 orchestration event subscription scope，使 SSE 只为 active parent 和 child agent run 运行。([#9273](https://github.com/warpdotdev/warp/pull/9273))
- 修复 macOS IME candidate popup 在 code editor pane 中的位置，使其锚定到 editor caret，而不是过期 terminal/input 位置。([#9555](https://github.com/warpdotdev/warp/pull/9555))

## Bug Fixes
- 修复 packaged build 中 /feedback recording 显示 "Unknown" 而不是已安装 Warp 版本的问题。([#10219](https://github.com/warpdotdev/warp/pull/10219))
- 修复新输出流入 active block 时，find（cmd+f）selection 跳到不同匹配项的问题。([#10057](https://github.com/warpdotdev/warp/pull/10057))
- 修复 macOS 上日语 IME 在短语正好结束于标点前时丢失最后一个字符的问题。([#9730](https://github.com/warpdotdev/warp/pull/9730))
- 修复连接到 SSH session 时本地 file tree 闪烁/重排的问题。([#10184](https://github.com/warpdotdev/warp/pull/10184))
- 修复 terminal text selection 在拖出边界时不会自动滚动的问题。([#9448](https://github.com/warpdotdev/warp/pull/9448))
- 修复 Linux 上 editor 聚焦时 Ctrl-G 无法关闭 CLI agent rich input 的问题。([#10030](https://github.com/warpdotdev/warp/pull/10030))
- 当 buffer 为空时，在 agent view 中按 backspace 不再重置 conversation。([#10114](https://github.com/warpdotdev/warp/pull/10114))
- 修复系统睡眠后 remote SSH session 不必要的重连尝试，减少错误噪声。([#10096](https://github.com/warpdotdev/warp/pull/10096))
- 修复 terminal pane resize 时 CLI agent 重复 TUI redraw 的问题。([#9877](https://github.com/warpdotdev/warp/pull/9877))
- 修复 Tabs Panel 位于 header toolbar 右侧时 new-session "+" dropdown 的对齐问题。([#9492](https://github.com/warpdotdev/warp/pull/9492))
- 当 input 和 block 同时有 selection 时，copy keybinding 现在会优先复制 input 中选中的文本。([#9491](https://github.com/warpdotdev/warp/pull/9491))
- [Windows] 修复 hotkey window。([#9891](https://github.com/warpdotdev/warp/pull/9891))
- [Windows] 修复 symlink traversal。([#9863](https://github.com/warpdotdev/warp/pull/9863))
- 修复将 Web conversation hand off 到 native client 时 Windows 上的 crash。([#9987](https://github.com/warpdotdev/warp/pull/9987))
- 修复多个 'open skill' button 共享 hover state 的 bug。([#9437](https://github.com/warpdotdev/warp/pull/9437))
- 修复 OSS Linux desktop entry，使 WarpOss 通过 packaged `warp-terminal-oss` command 启动。([#9424](https://github.com/warpdotdev/warp/pull/9424))
- 修复 Windows 上启用非拉丁 keyboard layout 时 Ctrl/Cmd shortcut（例如 copy、paste）失败的问题。([#9476](https://github.com/warpdotdev/warp/pull/9476))
- 修复 alt screen program（例如 delta、diff-so-fancy）中的 background colour bleed。([#9852](https://github.com/warpdotdev/warp/pull/9852))
- 在窄 pane 上将 warping indicator 的 action chip 裁切到新行，而不是溢出。([#9297](https://github.com/warpdotdev/warp/pull/9297))
- Agent block output 中的 inline `.bmp`、`.tiff` / `.tif` 和 `.ico` image 现在可正确渲染。([#9397](https://github.com/warpdotdev/warp/pull/9397))
- 如果用户在 block input 中附加 image，应锁定 agent mode，而不运行 NLD classifier。([#9366](https://github.com/warpdotdev/warp/pull/9366))
- 当 staging-directory cleanup 命中 race 时，remote-server install 不再失败。([#9681](https://github.com/warpdotdev/warp/pull/9681))
- `.command` shell script 现在会在 Warp editor 中以 shell syntax highlighting 打开。([#9345](https://github.com/warpdotdev/warp/pull/9345))
- 修复存在 untracked file 时 git diff chip 在 tracked-only 和 all-files count 之间闪烁的问题。([#9244](https://github.com/warpdotdev/warp/pull/9244))
- `Open File → Default App` 现在会在正在运行的 Warp channel 中打开文件，而不是路由到另一个已安装的 Warp。([#9285](https://github.com/warpdotdev/warp/pull/9285))
- 修复 vertical tabs settings popup item 无法点击的问题。([#9540](https://github.com/warpdotdev/warp/pull/9540))
- 修复 Warp 枚举系统字体或构建 font fallback chain 时发生的 macOS memory leak。([#9665](https://github.com/warpdotdev/warp/pull/9665))
- 从 `file://` URL 打开的 executable shell script 现在会在 terminal 中运行，而不是在 editor 中打开。([#9503](https://github.com/warpdotdev/warp/pull/9503))
- 修复 Option+Enter、Option+Tab 和 Option+Escape 发送 literal key name 而不是正确 escape sequence 的问题。([#9514](https://github.com/warpdotdev/warp/pull/9514))
- 修复 LLM 请求超出文件末尾的 line range 时 read_files tool 显示空 box 的问题。([#9326](https://github.com/warpdotdev/warp/pull/9326))
- 防止 Warp 在长 block output 中识别 filepath 时消耗过多内存。([#9617](https://github.com/warpdotdev/warp/pull/9617))
- 不要在 Warp 以 headless SDK/CLI mode 运行时触发 agent onboarding tutorial。([#9590](https://github.com/warpdotdev/warp/pull/9590))
- 在 Oz CLI 中新增 `--version` flag 支持。([#9252](https://github.com/warpdotdev/warp/pull/9252))
- 修复切换到 SSH remote session 时 file tree 闪烁的问题。([#9320](https://github.com/warpdotdev/warp/pull/9320))
- 修复 input 聚焦时 selected block 的 scroll-to-start/end keybinding 不工作的问题。([#9332](https://github.com/warpdotdev/warp/pull/9332))
- 修复带 background image 或自定义 opacity 的 horizontal tabs mode 中 terminal pane background 过暗的问题。([#9474](https://github.com/warpdotdev/warp/pull/9474))
- 标记为 `vue`、`xml`、`dockerfile`、`jsx`、`tsx` 等的 AI code block 现在会带 syntax highlighting 渲染。([#9471](https://github.com/warpdotdev/warp/pull/9471))
- Reopen Closed Session 现在可以从 Linux 和 Windows 的 new-session menu 访问。([#9347](https://github.com/warpdotdev/warp/pull/9347))
- 修复使用 `.hpp`、`.hxx` 或 `.H` extension 的 C++ header file 缺少 syntax highlighting 的问题。([#9388](https://github.com/warpdotdev/warp/pull/9388))
- 修复 `/open-file` 对 relative WSL path 的处理，使 Unix separator 得以保留。([#9322](https://github.com/warpdotdev/warp/pull/9322))

## Oz Updates
- 添加 Codex 作为 local child agent 的 supported harness。([#10176](https://github.com/warpdotdev/warp/pull/10176))
- 每个 profile 可配置 max context window。([#9352](https://github.com/warpdotdev/warp/pull/9352))

---

## Community
### Contributors
- @Abdalla-Eldoumani ✨
- @Akeuuh — [#9655](https://github.com/warpdotdev/warp/pull/9655) ✨
- @AntonVishal ✨
- @BennyWaitWhat ✨
- @Faizanq ✨
- @JamieMcMillan ✨
- @R3flector ✨
- @amriksingh0786 ✨
- @princepal9120 ✨
- @webdevtodayjason ✨
- @zerone0x ✨

### Issue Reporters
感谢报告本 release 中已修复 issue 的社区成员：
- @user123 — [#5678](https://github.com/warpdotdev/warp/issues/5678) "Crash when opening large file"

---

*此 draft 由 `changelog-draft` Oz skill 生成。Needs Review 和 Skipped PRs 可在 JSON audit artifact 中查看。*
