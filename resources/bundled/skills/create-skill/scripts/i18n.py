#!/usr/bin/env python3
"""Small locale helpers for bundled create-skill scripts."""

import locale
import os


def _normalized_locale(value: str) -> str:
    return value.strip().lower().replace("_", "-").split(".", 1)[0].split("@", 1)[0]


def _is_simplified_chinese(value: str) -> bool:
    normalized = _normalized_locale(value)
    return normalized in {"zh", "zh-cn", "zh-sg", "zh-hans"} or normalized.startswith(
        "zh-hans-"
    )


def _is_english(value: str) -> bool:
    normalized = _normalized_locale(value)
    return normalized == "en" or normalized.startswith("en-")


def _language_for_value(value: str, split_candidates: bool) -> str:
    candidates = value.split(":") if split_candidates else [value]
    for candidate in candidates:
        if _is_simplified_chinese(candidate):
            return "zh"
        if _is_english(candidate):
            return "en"
    return "en"


def current_language() -> str:
    for key in ("WARP_LOCALE", "LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"):
        value = os.environ.get(key)
        if not value:
            continue
        return _language_for_value(value, key == "LANGUAGE")

    return "zh" if _is_simplified_chinese(locale.getlocale()[0] or "") else "en"


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
