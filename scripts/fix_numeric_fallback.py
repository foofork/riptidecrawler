#!/usr/bin/env python3
"""
Automated fixer for Rust default_numeric_fallback clippy warnings.
Intelligently adds type suffixes to numeric literals based on context.
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple


class NumericFallbackFixer:
    def __init__(self):
        # Patterns for different numeric contexts
        self.patterns = [
            # Duration constructors (need u64)
            (r'Duration::from_millis\((\d+)\)', r'Duration::from_millis(\1_u64)'),
            (r'Duration::from_secs\((\d+)\)', r'Duration::from_secs(\1_u64)'),
            (r'Duration::from_nanos\((\d+)\)', r'Duration::from_nanos(\1_u64)'),
            (r'Duration::from_micros\((\d+)\)', r'Duration::from_micros(\1_u64)'),

            # Floating point literals in expressions (need _f64)
            # Standalone floats in assignments, comparisons
            (r'([=<>!+\-*/\s(,])([\d]+\.[\d]+)([^_\da-zA-Z])', r'\1\2_f64\3'),

            # Memory sizes (typically u64)
            (r'(\d+)\s*\*\s*1024\s*\*\s*1024(?!_)', r'\1_u64 * 1024_u64 * 1024_u64'),
            (r'(\d+)\s*\*\s*1024(?!_)', r'\1_u64 * 1024_u64'),

            # Array indexing (usize)
            (r'\.len\(\)\s*-\s*(\d+)(?!_)', r'.len() - \1_usize'),
            (r'\[(\d+)\](?!_)', r'[\1_usize]'),
            (r'get\((\d+)\)', r'get(\1_usize)'),

            # Port numbers (u16)
            (r':(\d{4,5})(?!_)', r':\1_u16'),

            # Percentage calculations (f64)
            (r'(\d+)\s*as\s*f64\s*/\s*100\.0(?!_)', r'\1 as f64 / 100.0_f64'),

            # Common numeric literals in conditionals and comparisons
            # Integers without type suffix in comparisons
            (r'([<>=!]\s*)(\d+)([;\s)])', self._classify_integer),

            # Function call arguments that are standalone integers
            (r',\s*(\d+)\s*([,)])', self._classify_arg),
        ]

    def _classify_integer(self, match):
        """Classify integer based on context - conservative approach"""
        prefix = match.group(1)
        num = match.group(2)
        suffix = match.group(3)

        # For comparisons, try to infer type from magnitude
        value = int(num)
        if value <= 100:
            return f"{prefix}{num}_usize{suffix}"  # Likely count/index
        elif value < 1000:
            return f"{prefix}{num}_u32{suffix}"    # Medium numbers
        else:
            return f"{prefix}{num}_u64{suffix}"    # Large numbers

    def _classify_arg(self, match):
        """Classify function argument integers"""
        num = match.group(1)
        suffix = match.group(2)

        value = int(num)
        if value < 256:
            return f", {num}_u32{suffix}"
        else:
            return f", {num}_u64{suffix}"

    def fix_file(self, filepath: Path) -> Tuple[bool, int]:
        """Fix numeric fallback issues in a single file"""
        try:
            content = filepath.read_text()
            original = content
            changes = 0

            # Apply each pattern
            for pattern, replacement in self.patterns:
                if callable(replacement):
                    new_content = re.sub(pattern, replacement, content)
                else:
                    new_content = re.sub(pattern, replacement, content)

                if new_content != content:
                    changes += content.count(pattern)
                    content = new_content

            # Only write if changes were made
            if content != original:
                filepath.write_text(content)
                return True, changes

            return False, 0

        except Exception as e:
            print(f"Error processing {filepath}: {e}", file=sys.stderr)
            return False, 0

    def fix_directory(self, directory: Path, include_tests: bool = False) -> dict:
        """Fix all Rust files in a directory"""
        results = {
            'files_processed': 0,
            'files_modified': 0,
            'total_changes': 0,
        }

        # Find all .rs files
        pattern = "**/*.rs"
        for filepath in directory.rglob(pattern):
            # Skip test files if requested
            if not include_tests and ('test' in str(filepath) or filepath.name.startswith('test_')):
                continue

            results['files_processed'] += 1
            modified, changes = self.fix_file(filepath)

            if modified:
                results['files_modified'] += 1
                results['total_changes'] += changes
                print(f"✓ Fixed {filepath} ({changes} patterns)")

        return results


def main():
    if len(sys.argv) < 2:
        print("Usage: fix_numeric_fallback.py <directory> [--include-tests]")
        sys.exit(1)

    directory = Path(sys.argv[1])
    include_tests = '--include-tests' in sys.argv

    if not directory.exists():
        print(f"Directory not found: {directory}", file=sys.stderr)
        sys.exit(1)

    fixer = NumericFallbackFixer()
    results = fixer.fix_directory(directory, include_tests)

    print(f"\n{'='*60}")
    print(f"Files processed: {results['files_processed']}")
    print(f"Files modified: {results['files_modified']}")
    print(f"Total changes: {results['total_changes']}")
    print(f"{'='*60}")


if __name__ == '__main__':
    main()
