#!/usr/bin/env python3

import yaml
import csv
import urllib.request
import os
from datetime import datetime

# Results directory
results_dir = "eval/results"
os.makedirs(results_dir, exist_ok=True)

# Output CSV file
csv_file = f"{results_dir}/url_verification_{datetime.now().strftime('%Y%m%d_%H%M%S')}.csv"

# Process all suite files
suites_dir = "eval/suites"
all_results = []

print("RipTide URL Verification")
print("=" * 40)

for suite_file in sorted(os.listdir(suites_dir)):
    if suite_file.endswith('.yml'):
        suite_path = os.path.join(suites_dir, suite_file)
        suite_name = suite_file.replace('.yml', '')

        print(f"\nSuite: {suite_name}")
        print("-" * 30)

        with open(suite_path, 'r') as f:
            data = yaml.safe_load(f)

        for target in data.get('targets', []):
            name = target.get('name', '')
            url = target.get('url', '')
            target_type = target.get('type', '')

            print(f"  Checking: {name[:40]}...", end=" ")

            try:
                req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
                response = urllib.request.urlopen(req, timeout=10)
                status_code = response.code
                status = "SUCCESS"
                print(f"✓ [{status_code}]")
            except Exception as e:
                status_code = 0
                status = "FAILED"
                print(f"✗ [{str(e)[:30]}]")

            all_results.append({
                'Suite': suite_name,
                'Name': name,
                'URL': url,
                'Type': target_type,
                'HTTP_Code': status_code,
                'Status': status
            })

# Write to CSV
with open(csv_file, 'w', newline='', encoding='utf-8') as f:
    fieldnames = ['Suite', 'Name', 'URL', 'Type', 'HTTP_Code', 'Status']
    writer = csv.DictWriter(f, fieldnames=fieldnames)
    writer.writeheader()
    writer.writerows(all_results)

# Summary
total = len(all_results)
success = sum(1 for r in all_results if r['Status'] == 'SUCCESS')
failed = total - success

print("\n" + "=" * 40)
print("Summary")
print("=" * 40)
print(f"Total URLs: {total}")
print(f"Successful: {success}")
print(f"Failed: {failed}")
print(f"Success Rate: {success*100//total if total > 0 else 0}%")
print(f"\nResults saved to: {csv_file}")
