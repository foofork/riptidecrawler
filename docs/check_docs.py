#!/usr/bin/env python3
"""
Check for missing doc comments on public Rust items.
"""

import os
import re
from pathlib import Path
from typing import List, Tuple
from collections import defaultdict

# Pattern to match public items
PUBLIC_PATTERN = re.compile(r'^\s*pub\s+(fn|struct|enum|trait|type|const|static|mod|use)\s+(\w+)')

# Pattern to match doc comments
DOC_COMMENT_PATTERN = re.compile(r'^\s*///')

# Patterns to ignore
IGNORE_PATTERNS = [
    re.compile(r'mod\s+tests'),
    re.compile(r'#\[test\]'),
    re.compile(r'pub\s+use'),  # Re-exports typically don't need their own docs
    re.compile(r'#\[cfg\(test\)\]'),
]


def should_ignore_line(line: str) -> bool:
    """Check if a line should be ignored."""
    return any(pattern.search(line) for pattern in IGNORE_PATTERNS)


def has_doc_comment(lines: List[str], line_idx: int) -> bool:
    """Check if a public item has a doc comment before it."""
    # Check up to 10 lines above for doc comments or attributes
    for i in range(max(0, line_idx - 10), line_idx):
        line = lines[i].rstrip()

        # If we hit a doc comment, we're good
        if DOC_COMMENT_PATTERN.match(line):
            return True

        # Skip empty lines and attributes
        if not line or line.strip().startswith('#['):
            continue

        # If we hit actual code, stop looking
        if line.strip() and not line.strip().startswith('//'):
            break

    return False


def check_file(file_path: Path) -> List[Tuple[int, str, str]]:
    """
    Check a single file for missing doc comments.
    Returns list of (line_number, item_type, item_name) tuples.
    """
    missing = []

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return missing

    for idx, line in enumerate(lines):
        # Skip if line should be ignored
        if should_ignore_line(line):
            continue

        # Check for public items
        match = PUBLIC_PATTERN.match(line)
        if match:
            item_type = match.group(1)
            item_name = match.group(2)

            # Check if it has a doc comment
            if not has_doc_comment(lines, idx):
                missing.append((idx + 1, item_type, item_name))

    return missing


def main():
    """Main function to check all Rust files."""
    crates_dir = Path('/workspaces/eventmesh/crates')

    # Focus on riptide-api crate first
    api_dir = crates_dir / 'riptide-api' / 'src'

    if not api_dir.exists():
        print(f"Directory not found: {api_dir}")
        return

    # Stats
    total_public_items = 0
    total_missing = 0
    files_with_issues = defaultdict(list)

    # Find all .rs files
    for rs_file in api_dir.rglob('*.rs'):
        # Skip test files
        if '/tests/' in str(rs_file) or rs_file.name.startswith('test_'):
            continue

        missing = check_file(rs_file)
        if missing:
            total_missing += len(missing)
            files_with_issues[rs_file] = missing

        # Count total public items in file
        try:
            with open(rs_file, 'r', encoding='utf-8') as f:
                content = f.read()
                # Count all public items (not just undocumented ones)
                total_public_items += len(PUBLIC_PATTERN.findall(content))
        except Exception:
            pass

    # Print report
    print("=" * 80)
    print("PUBLIC ITEM DOCUMENTATION ANALYSIS - riptide-api crate")
    print("=" * 80)
    print()

    if files_with_issues:
        print(f"⚠️  Found {total_missing} public items without doc comments\n")

        # Group by item type
        by_type = defaultdict(list)
        for file_path, items in files_with_issues.items():
            for line_num, item_type, item_name in items:
                by_type[item_type].append((file_path, line_num, item_name))

        # Print by type
        for item_type in sorted(by_type.keys()):
            items = by_type[item_type]
            print(f"\n{item_type.upper()} ({len(items)} items):")
            print("-" * 80)
            for file_path, line_num, item_name in items[:20]:  # Show first 20
                rel_path = file_path.relative_to(Path('/workspaces/eventmesh'))
                print(f"  ❌ {rel_path}:{line_num} - {item_type} {item_name}")

            if len(items) > 20:
                print(f"  ... and {len(items) - 20} more")
    else:
        print("✅ All public items have doc comments!")

    # Calculate coverage
    documented = total_public_items - total_missing
    coverage = (documented / total_public_items * 100) if total_public_items > 0 else 100

    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"Total public items:     {total_public_items}")
    print(f"Documented items:       {documented}")
    print(f"Missing documentation:  {total_missing}")
    print(f"Documentation coverage: {coverage:.1f}%")
    print()

    # Now check other important crates
    print("\n" + "=" * 80)
    print("CHECKING OTHER CRATES")
    print("=" * 80)

    important_crates = [
        'riptide-types',
        'riptide-extraction',
        'riptide-spider',
        'riptide-reliability',
    ]

    for crate_name in important_crates:
        crate_dir = crates_dir / crate_name / 'src'
        if not crate_dir.exists():
            continue

        crate_missing = 0
        crate_total = 0

        for rs_file in crate_dir.rglob('*.rs'):
            if '/tests/' in str(rs_file) or rs_file.name.startswith('test_'):
                continue

            missing = check_file(rs_file)
            crate_missing += len(missing)

            try:
                with open(rs_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    crate_total += len(PUBLIC_PATTERN.findall(content))
            except Exception:
                pass

        crate_coverage = ((crate_total - crate_missing) / crate_total * 100) if crate_total > 0 else 100
        status = "✅" if crate_missing == 0 else "⚠️"
        print(f"{status} {crate_name:30s} - {crate_missing:3d}/{crate_total:3d} missing ({crate_coverage:.1f}% coverage)")


if __name__ == '__main__':
    main()
