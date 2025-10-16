#!/usr/bin/env python3
"""
Parse product data from extracted HTML files and convert to CSV.
"""

import json
import re
import csv
import sys
from pathlib import Path


def extract_product_json(html_content):
    """Extract JSON-LD product data from HTML."""
    # Find JSON-LD script tag with Product schema
    pattern = r'<script type="application/ld\+json">(\{"@context":"https://schema\.org","@type":"Product".*?\})</script>'
    match = re.search(pattern, html_content, re.DOTALL)

    if match:
        try:
            return json.loads(match.group(1))
        except json.JSONDecodeError as e:
            print(f"Error parsing JSON: {e}", file=sys.stderr)
            return None
    return None


def extract_product_fields(product_data):
    """Extract relevant fields from product JSON."""
    if not product_data:
        return None

    fields = {
        'url': product_data.get('url', ''),
        'name': product_data.get('name', ''),
        'sku': product_data.get('sku', ''),
        'brand': product_data.get('brand', {}).get('name', '') if isinstance(product_data.get('brand'), dict) else '',
        'price': '',
        'currency': '',
        'availability': '',
        'description': product_data.get('description', ''),
        'rating': '',
        'review_count': ''
    }

    # Extract offer data
    offers = product_data.get('offers', {})
    if isinstance(offers, dict):
        fields['price'] = str(offers.get('price', ''))
        fields['currency'] = offers.get('priceCurrency', '')
        availability = offers.get('availability', '')
        # Simplify availability to just the status
        if 'OutOfStock' in availability:
            fields['availability'] = 'Out of Stock'
        elif 'InStock' in availability:
            fields['availability'] = 'In Stock'
        else:
            fields['availability'] = availability

    # Extract rating data
    rating_data = product_data.get('aggregateRating', {})
    if isinstance(rating_data, dict):
        fields['rating'] = str(rating_data.get('ratingValue', ''))
        fields['review_count'] = str(rating_data.get('reviewCount', ''))

    return fields


def process_html_files(input_files, output_file):
    """Process HTML files and write to CSV."""
    results = []

    for html_file in input_files:
        html_path = Path(html_file)
        if not html_path.exists():
            print(f"Warning: File not found: {html_file}", file=sys.stderr)
            continue

        print(f"Processing: {html_file}")
        with open(html_path, 'r', encoding='utf-8') as f:
            html_content = f.read()

        product_data = extract_product_json(html_content)
        if product_data:
            fields = extract_product_fields(product_data)
            if fields:
                results.append(fields)
                print(f"  ✓ Extracted: {fields['name']}")
            else:
                print(f"  ✗ Failed to extract fields", file=sys.stderr)
        else:
            print(f"  ✗ No product JSON found", file=sys.stderr)

    # Write to CSV
    if results:
        fieldnames = ['url', 'name', 'sku', 'brand', 'price', 'currency',
                     'availability', 'description', 'rating', 'review_count']

        with open(output_file, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(results)

        print(f"\n✓ Wrote {len(results)} products to {output_file}")
        return True
    else:
        print("\n✗ No products extracted", file=sys.stderr)
        return False


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: parse_product_data.py <output.csv> <input1.html> [input2.html] ...")
        sys.exit(1)

    output_file = sys.argv[1]
    input_files = sys.argv[2:]

    success = process_html_files(input_files, output_file)
    sys.exit(0 if success else 1)
