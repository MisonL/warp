# Warp 简体中文汉化版

本项目为上游 [Warp](https://github.com/warpdotdev/warp) 的汉化版本，会尽量遵循上游功能改动，同时在本仓库中添加国际化适配与简体中文支持。本仓库按独立项目演进，不代表上游官方发布版本。

<a href="https://www.warp.dev">
    <img width="1024" alt="Warp 智能开发环境产品预览" src="https://github.com/user-attachments/assets/9976b2da-2edd-4604-a36c-8fd53719c6d4" />
</a>
&nbsp;
<p align="center">
  <a href="https://www.warp.dev"><img height="20" alt="使用 Warp 构建" src="https://raw.githubusercontent.com/warpdotdev/brand-assets/main/Github/Built-With-Warp-Export@2x.png" /></a>
  &nbsp;
  <a href="https://oz.warp.dev"><img height="20" alt="由 Oz 提供支持" src="https://raw.githubusercontent.com/warpdotdev/brand-assets/main/Github/Powered-By-Oz-Export@2x.png" /></a>
</p>

<p align="center">
  <a href="https://www.warp.dev">官网</a>
  ·
  <a href="https://www.warp.dev/code">代码</a>
  ·
  <a href="https://www.warp.dev/agents">智能体</a>
  ·
  <a href="https://www.warp.dev/terminal">终端</a>
  ·
  <a href="https://www.warp.dev/drive">云盘</a>
  ·
  <a href="https://docs.warp.dev">文档</a>
  ·
  <a href="https://www.warp.dev/blog/how-warp-works">Warp 工作原理</a>
</p>

> [!NOTE]
> OpenAI 是新的开源 Warp 仓库的创始赞助方，新的智能管理工作流由 GPT 模型提供支持。

## 项目定位

本仓库以 Warp 上游开源代码为基础，重点维护简体中文本地化能力。目标是在尽量跟随上游功能变化的同时，让应用的用户界面具备可翻译性，并提供简体中文文案目录、语言设置、静态扫描和视觉验证证据。

本地化适配主要由以下内容维护：

- `app/assets/bundled/locales/en-US.json`
- `app/assets/bundled/locales/zh-CN.json`
- `crates/localization/`

## 关于 Warp

[Warp](https://www.warp.dev) 是从终端演进而来的智能开发环境。你可以使用 Warp 内置的编码智能体，也可以接入自己的命令行智能体，例如 Claude Code、Codex、Gemini CLI 等。

## 安装

你可以从 [Warp 下载页](https://www.warp.dev/download)获取安装包，也可以阅读 [Warp 官方文档](https://docs.warp.dev/)了解各平台安装说明。

本仓库是汉化适配版本；实际打包、发布和分发流程以本项目后续独立演进规则为准。

## Warp 贡献概览面板

可以访问 [build.warp.dev](https://build.warp.dev) 查看：

- Oz 智能体如何分拣 issue、编写规格、实现变更和审查 PR
- 主要贡献者和正在进行的功能
- 使用 GitHub 登录后跟踪自己的 issue
- 在 Web 编译版 Warp 终端中进入活跃智能体会话

## Oz for OSS

如果你维护一个受欢迎的开源项目，可以[申请 Oz 额度](https://tally.so/r/LZWxqG) 了解 [Oz for OSS](https://github.com/warpdotdev/oz-for-oss)。

Oz for OSS 是 Warp 的合作伙伴计划，用于把本仓库中使用的智能开源管理工作流带到精选合作仓库中。Warp 团队会与维护者直接协作，按项目实际情况落地 issue 分拣、PR 审查、社区管理和贡献者协调等工作流。

## 许可证

Warp 的 UI 框架，也就是 `warpui_core` 和 `warpui` crate，使用 [MIT 许可证](LICENSE-MIT)。

本仓库其余代码使用 [AGPL v3](LICENSE-AGPL)。

## 开源与贡献

Warp 客户端代码是开源的，并托管在本仓库中。上游欢迎社区贡献，并提供了轻量级流程帮助新贡献者开始参与。完整贡献流程可阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

本汉化版本按独立项目演进；是否向上游提交贡献、何时同步上游、如何发布汉化版本，以本项目维护策略为准。

> [!TIP]
> 可以在 [`#oss-contributors`](https://warpcommunity.slack.com/archives/C0B0LM8N4DB) Slack 频道与贡献者和 Warp 团队交流，这里适合临时问题、设计讨论和与维护者协作。新用户需要先加入 [Warp Slack 社区](https://go.warp.dev/join-preview)，再进入 `#oss-contributors`。

### 从 issue 到 PR

提交前，请先[搜索现有 issue](https://github.com/warpdotdev/warp/issues?q=is%3Aissue+is%3Aopen+sort%3Areactions-%2B1-desc)，确认是否已有相同 bug 或功能请求。如果没有，可以使用模板[提交 issue](https://github.com/warpdotdev/warp/issues/new/choose)。安全漏洞应按 [CONTRIBUTING.md](CONTRIBUTING.md#reporting-security-issues) 中的说明私下报告。

issue 创建后，上游 Warp 维护者可能会添加准备状态标签：[`ready-to-spec`](https://github.com/warpdotdev/warp/issues?q=is%3Aissue+is%3Aopen+label%3Aready-to-spec) 表示可以开始设计规格，[`ready-to-implement`](https://github.com/warpdotdev/warp/issues?q=is%3Aissue+is%3Aopen+label%3Aready-to-implement) 表示设计已确定，可以提交代码 PR。任何人都可以领取带标签的 issue。如果希望某个 issue 被评估是否可进入准备状态，可以在 issue 中提及 **@oss-maintainers**。

## 本地构建

从源码构建并运行 Warp：

```bash
./script/bootstrap   # 平台相关环境准备
./script/run         # 构建并运行 Warp
./script/presubmit   # 格式化、clippy 和测试
```

完整工程指南见 [WARP.md](WARP.md)，其中包括编码风格、测试和平台相关说明。

## 加入团队

如果你希望加入 Warp 团队，可以查看[开放职位](https://www.warp.dev/careers)。

## 支持与问题

1. 阅读 [Warp 官方文档](https://docs.warp.dev/)了解功能说明。
2. 加入 [Slack 社区](https://go.warp.dev/join-preview)，与其他用户交流并从 Warp 团队获得帮助；贡献者通常在 [`#oss-contributors`](https://warpcommunity.slack.com/archives/C0B0LM8N4DB)。
3. 试用 [Preview 构建](https://www.warp.dev/download-preview)，体验最新实验性功能。
4. 如需升级处理某个 issue，可以提及 **@oss-maintainers**，例如遇到自动化 agent 问题时。

## 行为准则

请保持尊重和同理心。Warp 遵循 [行为准则](CODE_OF_CONDUCT.md)。如果需要报告违规行为，请发送邮件至 warp-coc at warp.dev。

## 开源依赖

以下开源依赖帮助 Warp 从零开始构建：

- [Tokio](https://github.com/tokio-rs/tokio)
- [NuShell](https://github.com/nushell/nushell)
- [Fig Completion Specs](https://github.com/withfig/autocomplete)
- [Warp Server Framework](https://github.com/seanmonstar/warp)
- [Alacritty](https://github.com/alacritty/alacritty)
- [Hyper HTTP library](https://github.com/hyperium/hyper)
- [FontKit](https://github.com/servo/font-kit)
- [Core-foundation](https://github.com/servo/core-foundation-rs)
- [Smol](https://github.com/smol-rs/smol)
