此 PR 模板用于确保我们在发布新功能时，妥善完成沟通、跟踪、文档、可访问性等工作。

## PRD 检查清单

- [ ] 已计划如何用指标量化衡量成功

## 编码检查清单

- [ ] 在 dev 中测试一周
- [ ] 代码中包含 telemetry
- [ ] A11y（如适用，更多信息见[测试 a11y 指南](https://docs.google.com/document/d/1-H0bWss5Qw18ZpIYg-RUvN7_db1MVdWfOb5UF_GLxNc/edit?usp=sharing)）
- [ ] 添加到 Command Palette（如适用）
- [ ] 将 toggle setting 添加到 command palette（如适用）
- [ ] 添加到 Mac Menu（如适用）
- [ ] 添加 keybinding（如适用），可参考 [actions audit](https://docs.google.com/spreadsheets/d/1C56ZIqDGjJi873-HAPdnT2DofC3Z6G-aJMYeQeERHx4/edit#gid=0) 获取灵感
- [ ] 在应用内做 sanity check，确认它不会与其他 keybinding 冲突
- [ ] 日志中没有敏感信息
- [ ] dev 中没有与该功能相关的崩溃
- [ ] dev 中没有性能回归。见 [dashboard](https://warp.metabaseapp.com/dashboard/1519-dev-performance-by-version?shell=zsh)
- [ ] 该功能在 SSH 下工作正常，且没有回归。如何获取 VM 见[说明](https://github.com/warpdotdev/warp-internal/tree/master/app/tests/ssh/README.md)。
- [ ] 我们是否已经明确头脑风暴过开发者将如何发现此功能？
- [ ] 链接到 Figma mock
- [ ] 已在多个主题中测试（深色和浅色）
- [ ] 如果要发布的功能依赖某个 server API，该 server API 是否已经在 production 上稳定运行至少一个完整 server release 周期？更多详情见[这里](https://www.notion.so/warpdev/How-to-add-a-new-full-stack-feature-8412cede405a4ec194b32bdd4b951035?pvs=4#73b202f939834b97ab1fbdf7fc82cd53)。


## 内容检查清单

- [ ] 帮助内容
- [ ] Changelog 条目（在下方添加条目）
- [ ] [Telemetry 条目](https://docs.warp.dev/getting-started/privacy#exhaustive-telemetry-table)（如适用）
- [ ] Metabase 中的指标 dashboard
- [ ] Tweet（如适合）
- [ ] Blog post（如适合）

## Changelog

CHANGELOG-NEW-FEATURE: {{在此插入 changelog 条目}}
