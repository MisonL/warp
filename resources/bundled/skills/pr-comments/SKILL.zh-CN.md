---
name: pr-comments
description: "获取并展示当前分支对应 GitHub PR 的审查评论。"
---

# 获取 PR 评论

获取当前分支对应 GitHub PR 的所有 review comments，并通过 `insert_code_review_comments` 展示。

## 流程

1. 运行内置脚本。必须位于一个 git 仓库内，且当前分支有打开的 PR。运行 shell 命令时使用 `do_not_summarize_output: true`，避免 JSON 输出被截断。

   ```bash
   python3 <skill_dir>/scripts/fetch_github_review_comments.py
   ```

   脚本会向 stdout 输出 JSON。如果脚本无法获取评论，改用后面的 `gh` fallback 命令。

2. 使用 JSON 输出中的三个顶层字段调用 `insert_code_review_comments`：
   - `local_repository_path`
   - `base_branch`
   - `comments`

3. 停下来等待用户。展示每批评论后，必须询问用户希望如何继续。除非用户明确要求，否则不要继续行动。不要在用户没有要求时根据评论改代码。不要冒充用户提交 review 回复。你的角色只是获取并展示评论，然后等待指示。

## 脚本处理的内容

- 通过 `gh api --paginate` 获取 issue comments、diff comments 和 reviews
- 将较大的 diff hunks 裁剪到评论行附近的窗口
- 给回复评论设置 `reply_metadata`
- 给顶层 diff 评论设置 `location_metadata`，包括 filepath、裁剪后的 diff hunk、line 和 side
- PR 级评论（issue comments 和 reviews）没有 location 或 reply metadata

## fallback 命令

如果脚本获取评论失败，按以下步骤直接从 GitHub API 获取：

1. 使用 GitHub CLI 找到当前分支对应的 PR number、owner、repo 和 base branch。
2. 使用 GitHub `/repos/{owner_login}/{repo_name}/issues/{pr_number}/comments` endpoint 获取 PR 级评论。
3. 使用 GitHub `/repos/{owner_login}/{repo_name}/pulls/{pr_number}/comments` endpoint 获取行级和文件级 review comments。对 thread replies 移除 location metadata 和 diff hunks。
4. 使用 GitHub `/repos/{owner_login}/{repo_name}/pulls/{pr_number}/reviews` endpoint，并通过 filter 获取带评论正文的 code reviews。
5. 调用 `insert_code_review_comments` 工具把所有 PR 级、review 级、文件级和行级评论发送给用户。如果 PR 没有评论，也用该工具返回空列表。不要绕过该工具直接朗读评论内容。

禁用 pager，避免输出被分页。例如 macOS zsh：

```sh
GH_PAGER="" gh pr view --json number,headRepository,headRepositoryOwner,baseRefName
GH_PAGER="" gh api /repos/{owner_login}/{repo_name}/issues/{pr_number}/comments --jq '.[] | {id, html_url, user_login: .user.login, body, created_at, updated_at}'
GH_PAGER="" gh api /repos/{owner_login}/{repo_name}/pulls/{pr_number}/comments --jq '.[] | {id, html_url, diff_hunk, path, user_login: .user.login, body, created_at, updated_at, start_line, original_start_line, start_side, line, original_line, side, in_reply_to_id, subject_type} | if .in_reply_to_id != null then del(.diff_hunk, .path, .line, .original_line, .start_line, .original_start_line, .side, .start_side, .subject_type) else . end'
GH_PAGER="" gh api /repos/{owner_login}/{repo_name}/pulls/{pr_number}/reviews --jq '.[] | {id, html_url, user_login: .user.login, body, created_at, updated_at} | select(.body != "" and .body != null)'
```

根据用户的操作系统和 shell 调整以上命令，然后调用 `insert_code_review_comments`。

6. 展示评论后，执行上面流程第 3 步：停下来询问用户希望如何继续。没有明确指示时不要对评论采取行动。

## 要求

- `gh` CLI 已认证且有 repo 访问权限
- 当前分支有打开的 pull request
