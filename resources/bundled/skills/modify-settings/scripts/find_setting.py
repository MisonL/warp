"""
Find a setting's full dotted path and properties in the Warp settings JSON schema.

Usage:
    python3 find_setting.py <settings_schema_path> <key_name>

Example:
    python3 find_setting.py /path/to/settings_schema.json input_mode
"""

import json
import locale
import os
import sys


def _normalized_locale(value):
    return value.strip().lower().replace("_", "-").split(".", 1)[0].split("@", 1)[0]


def _is_simplified_chinese(value):
    normalized = _normalized_locale(value)
    return normalized in {"zh", "zh-cn", "zh-sg", "zh-hans"} or normalized.startswith(
        "zh-hans-"
    )


def _is_english(value):
    normalized = _normalized_locale(value)
    return normalized == "en" or normalized.startswith("en-")


def _language_for_value(value, split_candidates):
    candidates = value.split(":") if split_candidates else [value]
    for candidate in candidates:
        if _is_simplified_chinese(candidate):
            return "zh"
        if _is_english(candidate):
            return "en"
    return "en"


def current_language():
    for key in ("WARP_LOCALE", "LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"):
        value = os.environ.get(key)
        if value:
            return _language_for_value(value, key == "LANGUAGE")

    return "zh" if _is_simplified_chinese(locale.getlocale()[0] or "") else "en"


def localized_text(en, zh):
    return zh if current_language() == "zh" else en


def find_key(obj, target, path=""):
    found = False
    if isinstance(obj, dict):
        for k, v in obj.items():
            new_path = f"{path}.{k}" if path else k
            if k == target:
                print(localized_text(f"Path: {new_path}", f"路径：{new_path}"))
                print(json.dumps(v, indent=2))
                print()
                found = True
            if find_key(v, target, new_path):
                found = True
    return found


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(
            localized_text(
                f"Usage: {sys.argv[0]} <settings_schema_path> <key_name>",
                f"用法：{sys.argv[0]} <settings_schema_path> <key_name>",
            )
        )
        sys.exit(1)

    schema_path, target_key = sys.argv[1], sys.argv[2]

    with open(schema_path) as f:
        schema = json.load(f)

    if not find_key(schema, target_key):
        print(
            localized_text(
                f"No setting found matching key: {target_key}",
                f"未找到匹配此 key 的设置：{target_key}",
            ),
            file=sys.stderr,
        )
        sys.exit(1)
