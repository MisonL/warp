# Markdown 表格测试用例

## 01_simple_2x2

| 表头 1 | 表头 2 |
| --- | --- |
| 单元格 1 | 单元格 2 |

---

## 02_three_columns

| 姓名 | 年龄 | 城市 |
| --- | --- | --- |
| Alice | 30 | NYC |
| Bob | 25 | LA |

---

## 03_multiple_rows

| ID | 值 |
| --- | --- |
| 1 | Apple |
| 2 | Banana |
| 3 | Cherry |
| 4 | Date |
| 5 | Elderberry |

---

## 04_left_aligned

| 左 1 | 左 2 |
| :--- | :--- |
| 短文本 | 文本 |
| 长得多的文本 | 另一个 |

---

## 05_right_aligned

| 右 1 | 右 2 |
| ---: | ---: |
| 短文本 | 文本 |
| 长得多的文本 | 另一个 |

---

## 06_center_aligned

| 居中 1 | 居中 2 |
| :---: | :---: |
| 短文本 | 文本 |
| 长得多的文本 | 另一个 |

---

## 07_mixed_alignment

| 左 | 中 | 右 |
| :--- | :---: | ---: |
| L | C | R |
| 左对齐 | 居中 | 右对齐 |

---

## 08_bold

| 表头 | 值 |
| --- | --- |
| **粗体** | 普通 |
| 文本 | **也是粗体** |

---

## 09_italic

| 表头 | 值 |
| --- | --- |
| *斜体* | 普通 |
| 文本 | *也是斜体* |

---

## 10_inline_code

| 函数 | 返回 |
| --- | --- |
| `foo()` | `String` |
| `bar()` | `i32` |

---

## 11_links

| 站点 | URL |
| --- | --- |
| Google | [链接](https://google.com) |
| GitHub | [链接](https://github.com) |

---

## 12_strikethrough

| 项目 | 状态 |
| --- | --- |
| ~~已弃用~~ | 旧 |
| 活跃 | 当前 |

---

## 13_mixed_formatting

| 功能 | 描述 |
| --- | --- |
| **粗体** 加 *斜体* | 混合 |
| `code` 和 **粗体** | 组合 |
| ~~删除线~~ 和 *斜体* | 更多 |

---

## 14_empty_cells

| A | B | C |
| --- | --- | --- |
|  | 已填充 |  |
| 已填充 |  | 已填充 |

---

## 15_whitespace_cells

| A | B |
| --- | --- |
|   | 空格 |
| tab	 | 文本 |

---

## 16_escaped_pipes

| 表达式 | 结果 |
| --- | --- |
| A \| B | OR 操作 |
| X \| Y \| Z | 多个 |

---

## 17_long_content

| 短 | 很长的内容 |
| --- | --- |
| A | 这是一个包含大量文本的很长单元格，应当换行或截断 |
| B | 另一个包含大量内容的单元格 |

---

## 18_html_entities

| 符号 | 代码 |
| --- | --- |
| &lt; | 小于 |
| &gt; | 大于 |
| &amp; | Ampersand |

---

## 19_unicode_emoji

| 图标 | 名称 |
| --- | --- |
| 🚀 | Rocket |
| ⭐ | Star |
| 🎉 | Party |

---

## 20_wide_table

| Build Identifier | Release Channel | Feature Flag State | Workspace Session Token | Active Pane Title | Suggested Command Preview | Generated File Path | Git Branch Name | Pull Request Status | Reviewer Assignment | Telemetry Event Name | Render Mode | Table Layout Strategy | Horizontal Overflow Sentinel | Unbroken Content Sample | Final Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| build_2026_04_08_very_long_identifier_alpha | dogfood_internal_preview_rollout_candidate | markdown_table_horizontal_scroll_enabled | ws_session_token_01_ABCDEFGHIJKLMNOPQRSTUVWXYZ | agent_mode_diff_review_surface_with_extra_context | cargo_nextest_run_no_fail_fast_workspace_markdown_parser | crates/markdown_parser/examples/table-demo/all_test_cases.md | zach/wide-markdown-table-scroll | awaiting_follow_up_visual_regression_check | reviewer_assignment_pending_product_design | markdown_table_rendered_in_example_viewport | constrained_width_preview_panel | preserve_column_intrinsic_widths_before_wrapping | horizontal_scroll_should_be_required_here | SUPERLONGUNBROKENTEXTVALUE0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ | first_row_designed_to_force_width |
| build_2026_04_08_very_long_identifier_beta | stable_candidate_post_validation | markdown_table_horizontal_scroll_enabled | ws_session_token_02_ZYXWVUTSRQPONMLKJIHGFEDCBA | markdown_parser_demo_showing_extreme_width_case | cargo_run_features_with_local_server_markdown_demo | crates/markdown_parser/examples/table-demo/render_snapshot_reference.png | zach/wide-markdown-table-scroll | local_only_validation_before_pr | reviewer_assignment_not_requested_yet | markdown_table_horizontal_scroll_exercised | embedded_example_renderer | keep_headers_verbose_and_cells_intentionally_wide | overflow_region_should_extend_far_past_viewport | ANOTHEREXTREMELYLONGUNBROKENVALUE_for_horizontal_scroll_testing_only | second_row_keeps_pressure_on_layout |
| build_2026_04_08_very_long_identifier_gamma | canary_rollout_with_extra_observability | markdown_table_horizontal_scroll_enabled | ws_session_token_03_0123456789_repeat_repeat | full_width_table_case_for_manual_agent_testing | cargo_clippy_workspace_all_targets_all_features_tests | app/src/features/markdown/table_renderer/visual_debug_reference.rs | zach/wide-markdown-table-scroll | no_pr_needed_for_manual_local_test | reviewer_assignment_not_applicable | markdown_table_scroll_behavior_verified_manually | split_pane_code_review_view | avoid_collapsing_columns_even_with_dense_content | viewport_must_scroll_horizontally_to_reveal_tail_columns | YETANOTHERLONGUNBROKENCONTENTBLOCK_THAT_SHOULD_NOT_WRAP_EASILY | third_row_confirms_consistent_behavior |

---

## 21_deep_table

| ID | 值 |
| --- | --- |
| 1 | 行 1 |
| 2 | 行 2 |
| 3 | 行 3 |
| 4 | 行 4 |
| 5 | 行 5 |
| 6 | 行 6 |
| 7 | 行 7 |
| 8 | 行 8 |
| 9 | 行 9 |
| 10 | 行 10 |
| 11 | 行 11 |
| 12 | 行 12 |
| 13 | 行 13 |
| 14 | 行 14 |
| 15 | 行 15 |
| 16 | 行 16 |
| 17 | 行 17 |
| 18 | 行 18 |
| 19 | 行 19 |
| 20 | 行 20 |

---

## 22_large_grid

| C1 | C2 | C3 | C4 | C5 | C6 |
| --- | --- | --- | --- | --- | --- |
| R1C1 | R1C2 | R1C3 | R1C4 | R1C5 | R1C6 |
| R2C1 | R2C2 | R2C3 | R2C4 | R2C5 | R2C6 |
| R3C1 | R3C2 | R3C3 | R3C4 | R3C5 | R3C6 |
| R4C1 | R4C2 | R4C3 | R4C4 | R4C5 | R4C6 |
| R5C1 | R5C2 | R5C3 | R5C4 | R5C5 | R5C6 |
| R6C1 | R6C2 | R6C3 | R6C4 | R6C5 | R6C6 |
| R7C1 | R7C2 | R7C3 | R7C4 | R7C5 | R7C6 |
| R8C1 | R8C2 | R8C3 | R8C4 | R8C5 | R8C6 |
