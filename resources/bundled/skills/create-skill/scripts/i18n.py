#!/usr/bin/env python3
"""Small locale helpers for bundled create-skill scripts."""

import locale
import os


def current_language() -> str:
    locale_value = (
        os.environ.get("WARP_LOCALE")
        or os.environ.get("LANGUAGE")
        or os.environ.get("LC_ALL")
        or os.environ.get("LC_MESSAGES")
        or os.environ.get("LANG")
        or locale.getlocale()[0]
        or ""
    )
    return "zh" if locale_value.lower().replace("_", "-").startswith("zh") else "en"


def text(en: str, zh: str) -> str:
    return zh if current_language() == "zh" else en


def configure_argparse(argparse_module) -> None:
    if current_language() != "zh":
        return

    translations = {
        "usage: ": "用法：",
        "positional arguments": "位置参数",
        "options": "选项",
        "optional arguments": "选项",
        "show this help message and exit": "显示此帮助消息并退出",
        "the following arguments are required: %s": "缺少必要参数：%s",
        "unrecognized arguments: %s": "无法识别的参数：%s",
        "invalid choice: %(value)r (choose from %(choices)s)": "无效选项：%(value)r（可选：%(choices)s）",
    }
    argparse_module._ = lambda message: translations.get(message, message)
