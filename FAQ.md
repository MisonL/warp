# 常见问题

本 FAQ 覆盖我们最常听到的问题：如何为 Warp 客户端做贡献、如何在本仓库中使用 agent，以及本仓库与 Warp 产品之间的关系。完整贡献流程见 [CONTRIBUTING.md](CONTRIBUTING.md)。工程细节，包括构建环境、代码风格和测试，见 [AGENTS.md](AGENTS.md)。

## 贡献

### 我该如何贡献？

请从 GitHub issue 开始。Bug 报告在被分拣为可行动后，可以直接进入代码 PR；功能请求在写任何代码之前，需要先经过一个简短的 spec PR。完整流程，包括准备状态标签、spec PR、代码 PR 和审查，记录在 [CONTRIBUTING.md](CONTRIBUTING.md) 中。

### 如何提交高质量 bug 报告或功能请求？

请使用 [issue 模板](https://github.com/warpdotdev/warp/issues/new/choose)。对 bug，请包含复现步骤、预期行为与实际行为、你的 Warp 版本（`Settings → About`）和操作系统。对功能，请先描述面向用户的问题，再提出实现方案。

如果你已经在运行 Warp，`/feedback` 命令会自动提交一个 issue，并附上日志和环境详情。

### 准备状态标签是什么意思？

- **`ready-to-spec`** - 问题已经理解，设计仍开放。下一步是 spec PR。
- **`ready-to-implement`** - issue 已准备好进入代码 PR。对 bug 来说，这表示报告已经足够可复现或可行动。
- **`needs-mocks`** - 开始实现前需要设计 mock。

任何人都可以接手带标签的 issue。如果某个 issue 需要分拣或重新评估准备状态，请提及 **@oss-maintainers**。

### 为什么功能在写代码前需要 spec PR？

Spec 可以让范围、行为和架构在独立层面接受审查，避免有人先写出后续可能需要丢弃的代码。每个 spec PR 都会在 `specs/GH<issue-number>/` 下添加一个 `product.md`（期望行为）和一个 `tech.md`（实现计划）。每份文档应包含哪些内容，请看[打开 Spec PR](CONTRIBUTING.md#打开-spec-pr)。

### 如何从源码构建并运行 Warp？

```bash
./script/bootstrap   # 平台相关环境准备
cargo run            # 构建并运行 Warp
./script/presubmit   # fmt、clippy 和测试
```

macOS、Linux 和 Windows 都受支持。平台相关环境准备由 `./script/bootstrap` 处理。完整工程指南见 [AGENTS.md](AGENTS.md)。

### 我的 PR 会由真人审查，还是由 agent 审查？

两者都会。打开 PR 后，Oz 会自动被指派并产出初始审查。Oz 批准后，会自动请求 Warp 团队领域专家进行后续审查。你不需要手动指派 reviewer。

### 我的 PR 一直没人审查，该怎么办？

推送处理 Oz 反馈的变更后，请在 PR 中评论 `/oz-review` 请求重新审查。每个 PR 最多可以这样做三次。如果流程卡住，或你已经用完重新审查次数，请提及 **@oss-maintainers** 升级给团队处理。

### contributor 和 collaborator 有什么区别？

**Contributor** 是任何为项目做贡献的人，包括提交 issue、打开 PR、帮助分拣或参与讨论。大多数参与帮忙的人都是 contributor。你不需要任何权限或身份；只要提交 issue 或打开 PR 即可。

**Collaborator** 是一个正式 GitHub 角色，会授予在本仓库中有合并 PR 记录的贡献者。Collaborator 拥有更多权限：应用和管理 issue 标签、在任何 ready issue 上直接用 `@oz` 调度 Oz，以及为本仓库工作使用赠送的 Oz credits。

### 如何成为 collaborator？

有多个 PR 被合并的 contributor 可能会被邀请成为 collaborator。没有正式申请流程；持续贡献即可，维护者会主动联系。

## 在本仓库中使用 agent

### 我可以使用自己的编码 agent 来贡献吗？

可以。你可以使用任何你喜欢的工具，例如 Warp 内置 agent、Claude Code、Codex、Gemini CLI、Cursor 或其他工具，也可以完全不用 agent。本仓库提供了 agent 可读上下文，包括 [`.agents/skills/`](.agents/skills/) 下的 skill、[`specs/`](specs/) 下的 spec，以及 [`AGENTS.md`](AGENTS.md)。任何支持这些格式的 harness 都可以读取它们。

### 我可以在 Warp 中使用已有订阅的 Codex 或 Claude 模型，或提交 PR 添加这种能力吗？

目前不可以。Warp 内置 agent harness 在服务端运行，目前没有在本仓库开放。

不过，我们计划在 Warp 中支持 [ACP（agent client protocol）](https://agentclientprotocol.com/)，这样你就可以直接连接其他模型或订阅，并为你选择的编码 agent 获得原生 Warp 体验。

[这项工作已在 roadmap 中跟踪](https://github.com/warpdotdev/warp/issues/9233)，后续探索时我们会向社区更新进展。

### 如何让 Oz 帮我实现某个 issue？

在任何带准备状态标签的 issue 中提及 **@oss-maintainers** 并提出请求。获批请求会使用**赠送的 Oz credits** 运行；你不需要设置自己的 Oz 账户，也不需要支付计算费用。

成为 collaborator 后，你可以在任何 ready issue 上直接提及 `@oz` 进行调度，无需等待维护者。

### 在这里贡献需要付费吗？

不需要。无论手写贡献还是使用自己的 agent 贡献都是免费的。对本仓库中的获批请求，Oz 会使用 Warp 的 credits 运行；对回馈本仓库的 collaborator 也是免费的。

### Agent 生成的 PR 和人工 PR 执行同样标准吗？

是。无论代码由谁或什么工具编写，同样的 Oz + SME 审查、同样的测试，以及同样的 `./script/format` / `cargo clippy` / presubmit 检查都适用。PR 是手写还是 agent 生成，不会改变质量门槛，只会改变你迭代到满足门槛的速度。

### 我的 issue、评论或代码会被用于训练模型吗？

不会。Warp 不会使用对本仓库的贡献，或围绕这些贡献的讨论，来训练模型。

## 什么是开源的，什么不是

### Warp 是完全开源的吗？

Warp **客户端**是开源的：应用和大多数 crate 使用 [AGPL v3](LICENSE-AGPL) 许可，UI 框架 crate（`warpui_core`、`warpui`）使用 [MIT](LICENSE-MIT) 许可。**服务端**、**Warp Drive 后端**和 **Oz**（我们的 agent 编排层）不在本仓库中，目前仍是专有组件。

### 本仓库包含什么，不包含什么？

**本仓库包含：** Warp 客户端应用、WarpUI 框架、集成测试、agent skill 和功能 spec。

**本仓库不包含：** 服务端、Drive 后端、托管认证和 Oz 编排。

### 我能否在不登录或不使用 Warp 云服务的情况下运行 Warp？

部分功能完全在本地可用；其他功能（Drive 同步、托管模型 agent、团队功能）需要 Warp 后端。我们正在努力让可本地运行的范围逐步更清晰，包括在 onboarding 中提供更明确的控制项。

### 服务端或 Oz 将来会开源吗？

我们尚未承诺具体日期，也不想过度承诺。以 AGPL 开放客户端是一个单向门，开放服务端也会是类似承诺；如果以及当我们这样做时，会明确说明。

## 许可

### 为什么选择这个许可：应用用 AGPL，UI crate 用 MIT？

我们希望代码库中不同部分实现不同目标，所以选择了两种不同许可。

对**客户端应用**，我们选择 [AGPL v3](LICENSE-AGPL)，因为我们希望修改保持开放。像 MIT 或 Apache 2.0 这样的宽松许可会允许他人 fork 客户端、做出修改，再把闭源产品交付给用户；这是我们见过让面向最终用户的开源项目受损的模式，也不是我们希望培育的生态。AGPL 关闭了 GPL 留下的网络使用漏洞，因此托管形式的客户端衍生版本也会被覆盖。代价是 AGPL 比一些公司愿意嵌入专有产品的许可更严格，我们接受这一点；客户端不是我们预期被那样复用的层。

对 **UI 框架 crate**（`warpui_core`、`warpui`），我们选择 [MIT](LICENSE-MIT)，因为它们是通用基础设施，在 Warp 之外也有用。我们希望用 Rust 构建无关应用的人可以在不受 AGPL 摩擦影响的情况下采用它们。保持这一层宽松许可，有利于框架的覆盖范围，也有利于上游贡献回流。

简而言之：希望衍生保持开放的地方用 AGPL，希望最大化复用的地方用 MIT。

### 我可以在公司里按 AGPL 使用 Warp 吗？

可以。把 Warp 当作终端或开发环境使用，不会触发 AGPL 的网络或分发义务。AGPL 适用于你修改客户端，且向他人分发或托管该修改版本的情况。

### 为什么有 CLA？

CLA 授予 Warp 在本项目许可（AGPL 和 MIT）下重新分发贡献所需的权利，并用于处理未来许可和合规需求。它不会改变贡献到本仓库的代码许可。

### 其他人可以 fork Warp 吗？

可以，这正是 AGPL 的用途。该许可防止完全专有的重新发布；开放的衍生版本是受欢迎的。

## 帮助与安全

### 去哪里获取帮助？

- 使用产品相关问题可查阅 [Warp 文档](https://docs.warp.dev/)。
- Bug 报告和功能请求可使用 [GitHub Issues](https://github.com/warpdotdev/warp/issues)。
- 一般问题和讨论可加入 [Slack 社区](https://go.warp.dev/join-preview)；贡献者会在 [`#oss-contributors`](https://warpcommunity.slack.com/archives/C0B0LM8N4DB) 中与彼此和 Warp 团队交流。
- 如需升级给团队处理，请在 issue 或 PR 中提及 **@oss-maintainers**。

### 如何报告安全漏洞？

请不要打开公开 GitHub issue。请查看 [SECURITY.md](SECURITY.md)：通过 [security@warp.dev](mailto:security@warp.dev) 报告，或打开私有 [GitHub Security Advisory](https://github.com/warpdotdev/Warp/security/advisories/new)。
