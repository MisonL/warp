#!/usr/bin/env python3
"""
Quick validation script for skills - minimal version
"""

import sys
import os
import re
from pathlib import Path

try:
    import yaml
except ModuleNotFoundError:
    yaml = None

try:
    from scripts.i18n import text as localized_text
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from i18n import text as localized_text


def validate_skill(skill_path):
    """Basic validation of a skill"""
    skill_path = Path(skill_path)

    # Check SKILL.md exists
    skill_md = skill_path / 'SKILL.md'
    if not skill_md.exists():
        return False, localized_text("SKILL.md not found", "未找到 SKILL.md")

    # Read and validate frontmatter
    content = skill_md.read_text()
    if not content.startswith('---'):
        return False, localized_text("No YAML frontmatter found", "未找到 YAML frontmatter")

    # Extract frontmatter
    match = re.match(r'^---\n(.*?)\n---', content, re.DOTALL)
    if not match:
        return False, localized_text("Invalid frontmatter format", "frontmatter 格式无效")

    frontmatter_text = match.group(1)

    # Parse YAML frontmatter
    if yaml is None:
        return False, localized_text(
            "PyYAML is required to validate SKILL.md frontmatter. Install it with: python -m pip install PyYAML",
            "需要安装 PyYAML 才能校验 SKILL.md frontmatter。安装命令：python -m pip install PyYAML",
        )

    try:
        frontmatter = yaml.safe_load(frontmatter_text)
        if not isinstance(frontmatter, dict):
            return False, localized_text("Frontmatter must be a YAML dictionary", "frontmatter 必须是 YAML 字典")
    except yaml.YAMLError as e:
        return False, localized_text(f"Invalid YAML in frontmatter: {e}", f"frontmatter 中的 YAML 无效：{e}")

    # Define allowed properties
    ALLOWED_PROPERTIES = {
        'name',
        'description',
        'description_zh_CN',
        'license',
        'allowed-tools',
        'metadata',
        'compatibility',
    }

    # Check for unexpected properties (excluding nested keys under metadata)
    unexpected_keys = set(frontmatter.keys()) - ALLOWED_PROPERTIES
    if unexpected_keys:
        return False, (
            localized_text(
                f"Unexpected key(s) in SKILL.md frontmatter: {', '.join(sorted(unexpected_keys))}. "
                f"Allowed properties are: {', '.join(sorted(ALLOWED_PROPERTIES))}",
                f"SKILL.md frontmatter 中存在非预期字段：{', '.join(sorted(unexpected_keys))}。"
                f"允许的字段为：{', '.join(sorted(ALLOWED_PROPERTIES))}",
            )
        )

    # Check required fields
    if 'name' not in frontmatter:
        return False, localized_text("Missing 'name' in frontmatter", "frontmatter 缺少 'name'")
    if 'description' not in frontmatter:
        return False, localized_text("Missing 'description' in frontmatter", "frontmatter 缺少 'description'")

    # Extract name for validation
    name = frontmatter.get('name', '')
    if not isinstance(name, str):
        return False, localized_text(f"Name must be a string, got {type(name).__name__}", f"Name 必须是字符串，实际为 {type(name).__name__}")
    name = name.strip()
    if name:
        # Check naming convention (kebab-case: lowercase with hyphens)
        if not re.match(r'^[a-z0-9-]+$', name):
            return False, localized_text(
                f"Name '{name}' should be kebab-case (lowercase letters, digits, and hyphens only)",
                f"Name '{name}' 应使用 kebab-case（仅包含小写字母、数字和连字符）",
            )
        if name.startswith('-') or name.endswith('-') or '--' in name:
            return False, localized_text(
                f"Name '{name}' cannot start/end with hyphen or contain consecutive hyphens",
                f"Name '{name}' 不能以连字符开头或结尾，也不能包含连续连字符",
            )
        # Check name length (max 64 characters per spec)
        if len(name) > 64:
            return False, localized_text(
                f"Name is too long ({len(name)} characters). Maximum is 64 characters.",
                f"Name 过长（{len(name)} 个字符）。最大长度为 64 个字符。",
            )

    # Extract and validate description
    description = frontmatter.get('description', '')
    if not isinstance(description, str):
        return False, localized_text(
            f"Description must be a string, got {type(description).__name__}",
            f"Description 必须是字符串，实际为 {type(description).__name__}",
        )
    description = description.strip()
    if description:
        # Check for angle brackets
        if '<' in description or '>' in description:
            return False, localized_text("Description cannot contain angle brackets (< or >)", "Description 不能包含尖括号（< 或 >）")
        # Check description length (max 1024 characters per spec)
        if len(description) > 1024:
            return False, localized_text(
                f"Description is too long ({len(description)} characters). Maximum is 1024 characters.",
                f"Description 过长（{len(description)} 个字符）。最大长度为 1024 个字符。",
            )

    # Validate compatibility field if present (optional)
    compatibility = frontmatter.get('compatibility', '')
    if compatibility:
        if not isinstance(compatibility, str):
            return False, localized_text(
                f"Compatibility must be a string, got {type(compatibility).__name__}",
                f"Compatibility 必须是字符串，实际为 {type(compatibility).__name__}",
            )
        if len(compatibility) > 500:
            return False, localized_text(
                f"Compatibility is too long ({len(compatibility)} characters). Maximum is 500 characters.",
                f"Compatibility 过长（{len(compatibility)} 个字符）。最大长度为 500 个字符。",
            )

    return True, localized_text("Skill is valid!", "Skill 校验通过。")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(localized_text("Usage: python quick_validate.py <skill_directory>", "用法：python quick_validate.py <skill_directory>"))
        sys.exit(1)
    
    valid, message = validate_skill(sys.argv[1])
    print(message)
    sys.exit(0 if valid else 1)
