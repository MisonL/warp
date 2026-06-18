---
name: triage-vulnerabilities
description: Triage and remediate security vulnerabilities across Warp infrastructure. Checks Dependabot alerts (GitHub), GCP Artifact Registry container scanning, and Docker Scout for public images. Use when the user asks to check for vulnerabilities, triage CVEs, fix dependency issues, update base images, or remediate security alerts.
description_zh_CN: 排查并修复 Warp 基础设施中的安全漏洞。检查 GitHub Dependabot 警报、GCP Artifact Registry 容器扫描和 Docker Scout 公共镜像；适用于检查漏洞、分诊 CVE、修复依赖、更新基础镜像或处理安全告警。
---

# triage-vulnerabilities

跨四类来源分诊并修复安全漏洞：GitHub Dependabot、GCP container registry scanning、Docker Scout、Linear security issues。

## 漏洞来源

### 1. Dependabot (GitHub)

启用 Dependabot 的 repo：`warp-internal`、`warp-server`、`warp-terraform`、`session-sharing-server`。

拉取 open alerts：

```bash
gh api /repos/warpdotdev/<repo>/dependabot/alerts?state=open

gh api /repos/warpdotdev/<repo>/dependabot/alerts?state=open \
  --jq '.[] | [.number, .security_advisory.cve_id // .security_advisory.ghsa_id, .dependency.package.name, .security_advisory.severity, (.security_vulnerability.first_patched_version.identifier // "no fix"), .dependency.manifest_path] | @tsv'
```

关键字段：alert number、state、URL、package、ecosystem、manifest path、CVE/GHSA、summary、severity、first patched version。

### 2. GCP Artifact Registry Scanning

`us-east4` 中的内部 service image：

- Production project `astral-field-294621`: `warp-server`、`warp-server-jobs`、`warp-server-migrations`、`session-sharing-server`、`pgbouncer-rtc`
- Staging project `warp-server-staging`: 同上，另含 `cloud-run-source-deploy`

扫描步骤：

```bash
gcloud artifacts docker images list \
  us-east4-docker.pkg.dev/<project>/<repo> \
  --include-tags --sort-by=~create_time --limit=5

gcloud artifacts vulnerabilities list \
  "us-east4-docker.pkg.dev/<project>/<repo>/<image>@sha256:<digest>" \
  --format=json
```

只扫描每个 repo 最新、最近打 tag 的 image。旧 image 通常不可操作。

### 3. Docker Scout

Docker Hub 上 `warpdotdev/` org 下的公开 image。目前已接入的 repo 包括 `dev-base`，也可能有其他 repo，先用 `docker scout repo list --org warpdotdev` 检查。

```bash
docker scout cves warpdotdev/<image> --only-severity critical,high
docker scout recommendations warpdotdev/<image>
```

### 4. Linear Security Issues

Linear 中带 Security label 的 issue 用于追踪自动工具未覆盖的漏洞，例如内部发现、人工分诊或组织安全事项。Linear issue 不会因为 Dependabot PR 合并而自动关闭，因此可能与其他来源重复。

搜索 open security issue 时，使用 Linear MCP 查询 Security label，并排除 Done、Cancelled。记录 issue number、title、URL、status、description。

## 分诊流程

### 第 1 步：收集漏洞

使用 TODO 跟踪每个来源。查询四类来源，并把结果写到临时文件：

- `dependabot_alerts.tsv`: CVE、package、severity、repo、fix version
- `gcp_vulns.tsv`: CVE、severity、package、image、fix available
- `scout_vulns.tsv`: CVE、severity、package、image
- `linear_security.tsv`: Issue ID、CVE/title、status、URL

### 第 2 步：去重

同一个 CVE 可能出现在多个来源，例如 base image 漏洞同时被 GCP scanning 和 Docker Scout 报告，或 Dependabot alert 被 Linear issue 重复追踪。按 CVE ID 分组，并记录所有 affected source/image。Linear issue 若只是重复追踪已有 Dependabot 漏洞，应说明底层修复合并后即可视为解决。

为每个唯一漏洞添加 TODO。

### 第 3 步：一次修复一个漏洞

先确认所有来源都查完并去重，再开始修复。按严重程度排序：critical、high、medium、low。

#### a. 检查上游修复

- Dependabot alerts: 查看 `security_vulnerability.first_patched_version`。
- GCP scanning: 查看 `FIX_AVAILABLE`。
- Docker Scout: 查看 `docker scout recommendations`。
- 必要时查外部来源：
  - NVD: `https://nvd.nist.gov/vuln/detail/<CVE-ID>`
  - GitHub Advisory Database: `https://github.com/advisories`
  - Distroless issues: `https://github.com/GoogleContainerTools/distroless/issues`

若没有上游修复，报告漏洞并继续下一个。不要尝试 deactivate 或 dismiss alert，只有人类可以做。

#### b. 应用修复

Dependabot 自动修复：

```bash
gh pr list --repo warpdotdev/<repo> --author app/dependabot --state open --json title,url
```

如果已有 PR，review 并 approve。没有 PR 时可能需要手动处理。

依赖升级：当 Dependabot 不能自动修复时，手动更新相应 manifest，例如 `Cargo.toml`、`go.mod`、`package.json`，运行测试并提交 PR。必要时可以同时更新相关依赖。优先更新 direct dependency，而不是添加 override。若 direct dependency 没有可用版本能拉入修复后的 transitive dependency，且 override 不直接，按严重程度和暴露面判断是否等待。

基础设施变更：容器漏洞可能需要更新 Dockerfile base image、sidecar image 版本，或等待 distroless base image 更新。等待上游时报告并跳过。

提交修复 PR 时：

- PR 标题包含 CVE ID。
- PR 描述包含 advisory 链接。
- 使用 `create-pr` 技能创建 PR。

#### c. 报告无法解决的漏洞

无可用修复时报告：

- CVE ID 和 severity。
- 受影响 package/image。
- 为什么无法修复。
- 上游修复追踪位置。

可以建议人类暂时 deactivate alert。不要自己 dismiss 或 deactivate。

## 重要约束

- 永远不要 dismiss 或 deactivate vulnerability alert。
- 一次只处理一个漏洞，修复、验证后再处理下一个。
- 按 severity 优先级处理：critical、high、medium、low。
- 先检查 production project `astral-field-294621`，再检查 staging。
- distroless image 的修复依赖上游 base image。
- 判断依赖实际使用方式。构建工具漏洞通常低于生产运行路径中的漏洞。
