#!/usr/bin/env python3
"""Generate an HTML report from run_loop.py output.

Takes the JSON output from run_loop.py and generates a visual HTML report
showing each description attempt with check/x for each test case.
Distinguishes between train and test queries.
"""

import argparse
import html
import json
import sys
from pathlib import Path

try:
    from scripts.i18n import configure_argparse, text as localized_text
except ModuleNotFoundError:
    from i18n import configure_argparse, text as localized_text


def generate_html(data: dict, auto_refresh: bool = False, skill_name: str = "") -> str:
    """Generate HTML report from loop output data. If auto_refresh is True, adds a meta refresh tag."""
    history = data.get("history", [])
    holdout = data.get("holdout", 0)
    title_prefix = html.escape(skill_name + " - ") if skill_name else ""
    page_title = localized_text("Skill Description Optimization", "Skill 描述优化")
    explainer_title = localized_text(
        "Optimizing your skill's description.",
        "正在优化 skill 描述。",
    )
    explainer_body = localized_text(
        "This page updates automatically as the agent tests different versions of your skill's description. "
        "Each row is an iteration - a new description attempt. The columns show test queries: green PASS labels "
        "mean the skill triggered correctly or correctly did not trigger, and red FAIL labels mean it got the "
        "decision wrong. The Train score shows performance on queries used to improve the description; the Test "
        "score shows performance on held-out queries the optimizer has not seen. When it is done, the agent will "
        "apply the best-performing description to your skill.",
        "当 agent 测试不同版本的 skill 描述时，此页面会自动更新。每一行代表一次迭代，也就是一次新的描述尝试。"
        "列中展示测试查询：绿色“通过”表示 skill 正确触发或正确未触发，红色“失败”表示判断错误。训练分数展示用于改进描述的查询表现；"
        "测试分数展示优化器未见过的保留查询表现。完成后，agent 会将表现最好的描述应用到你的 skill。",
    )
    labels = {
        "original": localized_text("Original", "原始描述"),
        "best": localized_text("Best", "最佳描述"),
        "best_score": localized_text("Best Score", "最佳分数"),
        "iterations": localized_text("Iterations", "迭代次数"),
        "train": localized_text("Train", "训练"),
        "test": localized_text("Test", "测试"),
        "query_columns": localized_text("Query columns", "查询列"),
        "should_trigger": localized_text("Should trigger", "应触发"),
        "should_not_trigger": localized_text("Should NOT trigger", "不应触发"),
        "iter": localized_text("Iter", "轮次"),
        "description": localized_text("Description", "描述"),
        "pass": localized_text("PASS", "通过"),
        "fail": localized_text("FAIL", "失败"),
        "test_context": localized_text("(test)", "（测试）"),
        "train_context": localized_text("(train)", "（训练）"),
    }

    # Get all unique queries from train and test sets, with should_trigger info
    train_queries: list[dict] = []
    test_queries: list[dict] = []
    if history:
        for r in history[0].get("train_results", history[0].get("results", [])):
            train_queries.append({"query": r["query"], "should_trigger": r.get("should_trigger", True)})
        if history[0].get("test_results"):
            for r in history[0].get("test_results", []):
                test_queries.append({"query": r["query"], "should_trigger": r.get("should_trigger", True)})

    refresh_tag = '    <meta http-equiv="refresh" content="5">\n' if auto_refresh else ""

    html_parts = ["""<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
""" + refresh_tag + """    <title>""" + title_prefix + html.escape(page_title) + """</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@500;600&family=Lora:wght@400;500&display=swap" rel="stylesheet">
    <style>
        body {
            font-family: 'Lora', Georgia, serif;
            max-width: 100%;
            margin: 0 auto;
            padding: 20px;
            background: #faf9f5;
            color: #141413;
        }
        h1 { font-family: 'Poppins', sans-serif; color: #141413; }
        .explainer {
            background: white;
            padding: 15px;
            border-radius: 6px;
            margin-bottom: 20px;
            border: 1px solid #e8e6dc;
            color: #b0aea5;
            font-size: 0.875rem;
            line-height: 1.6;
        }
        .summary {
            background: white;
            padding: 15px;
            border-radius: 6px;
            margin-bottom: 20px;
            border: 1px solid #e8e6dc;
        }
        .summary p { margin: 5px 0; }
        .best { color: #788c5d; font-weight: bold; }
        .table-container {
            overflow-x: auto;
            width: 100%;
        }
        table {
            border-collapse: collapse;
            background: white;
            border: 1px solid #e8e6dc;
            border-radius: 6px;
            font-size: 12px;
            min-width: 100%;
        }
        th, td {
            padding: 8px;
            text-align: left;
            border: 1px solid #e8e6dc;
            white-space: normal;
            word-wrap: break-word;
        }
        th {
            font-family: 'Poppins', sans-serif;
            background: #141413;
            color: #faf9f5;
            font-weight: 500;
        }
        th.test-col {
            background: #6a9bcc;
        }
        th.query-col { min-width: 200px; }
        td.description {
            font-family: monospace;
            font-size: 11px;
            word-wrap: break-word;
            max-width: 400px;
        }
        td.result {
            text-align: center;
            font-size: 16px;
            min-width: 40px;
        }
        td.test-result {
            background: #f0f6fc;
        }
        .pass { color: #788c5d; }
        .fail { color: #c44; }
        .rate {
            font-size: 9px;
            color: #b0aea5;
            display: block;
        }
        tr:hover { background: #faf9f5; }
        .score {
            display: inline-block;
            padding: 2px 6px;
            border-radius: 4px;
            font-weight: bold;
            font-size: 11px;
        }
        .score-good { background: #eef2e8; color: #788c5d; }
        .score-ok { background: #fef3c7; color: #d97706; }
        .score-bad { background: #fceaea; color: #c44; }
        .train-label { color: #b0aea5; font-size: 10px; }
        .test-label { color: #6a9bcc; font-size: 10px; font-weight: bold; }
        .best-row { background: #f5f8f2; }
        th.positive-col { border-bottom: 3px solid #788c5d; }
        th.negative-col { border-bottom: 3px solid #c44; }
        th.test-col.positive-col { border-bottom: 3px solid #788c5d; }
        th.test-col.negative-col { border-bottom: 3px solid #c44; }
        .legend { font-family: 'Poppins', sans-serif; display: flex; gap: 20px; margin-bottom: 10px; font-size: 13px; align-items: center; }
        .legend-item { display: flex; align-items: center; gap: 6px; }
        .legend-swatch { width: 16px; height: 16px; border-radius: 3px; display: inline-block; }
        .swatch-positive { background: #141413; border-bottom: 3px solid #788c5d; }
        .swatch-negative { background: #141413; border-bottom: 3px solid #c44; }
        .swatch-test { background: #6a9bcc; }
        .swatch-train { background: #141413; }
    </style>
</head>
<body>
    <h1>""" + title_prefix + html.escape(page_title) + """</h1>
    <div class="explainer">
        <strong>""" + html.escape(explainer_title) + """</strong> """ + html.escape(explainer_body) + """
    </div>
"""]

    # Summary section
    best_test_score = data.get('best_test_score')
    best_train_score = data.get('best_train_score')
    html_parts.append(f"""
    <div class="summary">
        <p><strong>{labels['original']}:</strong> {html.escape(data.get('original_description', 'N/A'))}</p>
        <p class="best"><strong>{labels['best']}:</strong> {html.escape(data.get('best_description', 'N/A'))}</p>
        <p><strong>{labels['best_score']}:</strong> {data.get('best_score', 'N/A')} {labels['test_context'] if best_test_score else labels['train_context']}</p>
        <p><strong>{labels['iterations']}:</strong> {data.get('iterations_run', 0)} | <strong>{labels['train']}:</strong> {data.get('train_size', '?')} | <strong>{labels['test']}:</strong> {data.get('test_size', '?')}</p>
    </div>
""")

    # Legend
    html_parts.append("""
    <div class="legend">
        <span style="font-weight:600">""" + labels["query_columns"] + """:</span>
        <span class="legend-item"><span class="legend-swatch swatch-positive"></span> """ + labels["should_trigger"] + """</span>
        <span class="legend-item"><span class="legend-swatch swatch-negative"></span> """ + labels["should_not_trigger"] + """</span>
        <span class="legend-item"><span class="legend-swatch swatch-train"></span> """ + labels["train"] + """</span>
        <span class="legend-item"><span class="legend-swatch swatch-test"></span> """ + labels["test"] + """</span>
    </div>
""")

    # Table header
    html_parts.append("""
    <div class="table-container">
    <table>
        <thead>
            <tr>
                <th>""" + labels["iter"] + """</th>
                <th>""" + labels["train"] + """</th>
                <th>""" + labels["test"] + """</th>
                <th class="query-col">""" + labels["description"] + """</th>
""")

    # Add column headers for train queries
    for qinfo in train_queries:
        polarity = "positive-col" if qinfo["should_trigger"] else "negative-col"
        html_parts.append(f'                <th class="{polarity}">{html.escape(qinfo["query"])}</th>\n')

    # Add column headers for test queries (different color)
    for qinfo in test_queries:
        polarity = "positive-col" if qinfo["should_trigger"] else "negative-col"
        html_parts.append(f'                <th class="test-col {polarity}">{html.escape(qinfo["query"])}</th>\n')

    html_parts.append("""            </tr>
        </thead>
        <tbody>
""")

    # Find best iteration for highlighting
    if test_queries:
        best_iter = max(history, key=lambda h: h.get("test_passed") or 0).get("iteration")
    else:
        best_iter = max(history, key=lambda h: h.get("train_passed", h.get("passed", 0))).get("iteration")

    # Add rows for each iteration
    for h in history:
        iteration = h.get("iteration", "?")
        train_passed = h.get("train_passed", h.get("passed", 0))
        train_total = h.get("train_total", h.get("total", 0))
        test_passed = h.get("test_passed")
        test_total = h.get("test_total")
        description = h.get("description", "")
        train_results = h.get("train_results", h.get("results", []))
        test_results = h.get("test_results", [])

        # Create lookups for results by query
        train_by_query = {r["query"]: r for r in train_results}
        test_by_query = {r["query"]: r for r in test_results} if test_results else {}

        # Compute aggregate correct/total runs across all retries
        def aggregate_runs(results: list[dict]) -> tuple[int, int]:
            correct = 0
            total = 0
            for r in results:
                runs = r.get("runs", 0)
                triggers = r.get("triggers", 0)
                total += runs
                if r.get("should_trigger", True):
                    correct += triggers
                else:
                    correct += runs - triggers
            return correct, total

        train_correct, train_runs = aggregate_runs(train_results)
        test_correct, test_runs = aggregate_runs(test_results)

        # Determine score classes
        def score_class(correct: int, total: int) -> str:
            if total > 0:
                ratio = correct / total
                if ratio >= 0.8:
                    return "score-good"
                elif ratio >= 0.5:
                    return "score-ok"
            return "score-bad"

        train_class = score_class(train_correct, train_runs)
        test_class = score_class(test_correct, test_runs)

        row_class = "best-row" if iteration == best_iter else ""

        html_parts.append(f"""            <tr class="{row_class}">
                <td>{iteration}</td>
                <td><span class="score {train_class}">{train_correct}/{train_runs}</span></td>
                <td><span class="score {test_class}">{test_correct}/{test_runs}</span></td>
                <td class="description">{html.escape(description)}</td>
""")

        # Add result for each train query
        for qinfo in train_queries:
            r = train_by_query.get(qinfo["query"], {})
            did_pass = r.get("pass", False)
            triggers = r.get("triggers", 0)
            runs = r.get("runs", 0)

            status_label = labels["pass"] if did_pass else labels["fail"]
            css_class = "pass" if did_pass else "fail"

            html_parts.append(f'                <td class="result {css_class}">{status_label}<span class="rate">{triggers}/{runs}</span></td>\n')

        # Add result for each test query (with different background)
        for qinfo in test_queries:
            r = test_by_query.get(qinfo["query"], {})
            did_pass = r.get("pass", False)
            triggers = r.get("triggers", 0)
            runs = r.get("runs", 0)

            status_label = labels["pass"] if did_pass else labels["fail"]
            css_class = "pass" if did_pass else "fail"

            html_parts.append(f'                <td class="result test-result {css_class}">{status_label}<span class="rate">{triggers}/{runs}</span></td>\n')

        html_parts.append("            </tr>\n")

    html_parts.append("""        </tbody>
    </table>
    </div>
""")

    html_parts.append("""
</body>
</html>
""")

    return "".join(html_parts)


def main():
    configure_argparse(argparse)
    parser = argparse.ArgumentParser(
        description=localized_text("Generate HTML report from run_loop output", "根据 run_loop 输出生成 HTML 报告")
    )
    parser.add_argument("input", help=localized_text("Path to JSON output from run_loop.py (or - for stdin)", "run_loop.py JSON 输出路径（或使用 - 从 stdin 读取）"))
    parser.add_argument("-o", "--output", default=None, help=localized_text("Output HTML file (default: stdout)", "输出 HTML 文件（默认：stdout）"))
    parser.add_argument("--skill-name", default="", help=localized_text("Skill name to include in the report title", "要包含在报告标题中的 skill 名称"))
    args = parser.parse_args()

    if args.input == "-":
        data = json.load(sys.stdin)
    else:
        data = json.loads(Path(args.input).read_text())

    html_output = generate_html(data, skill_name=args.skill_name)

    if args.output:
        Path(args.output).write_text(html_output)
        print(localized_text(f"Report written to {args.output}", f"报告已写入 {args.output}"), file=sys.stderr)
    else:
        print(html_output)


if __name__ == "__main__":
    main()
