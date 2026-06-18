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


def current_language():
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
