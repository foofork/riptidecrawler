# Legacy Crates

This directory contains crates that are no longer actively maintained or used in the OSS Riptide project but are preserved for reference purposes.

## Contents

### riptide-schemas
Schema definitions and validation logic that has been moved to the product layer. See its README for details.

## Policy

Crates in this directory:
- Are **not** built as part of the workspace
- Are **not** included in releases
- Are **not** maintained or updated
- Are kept **only** for historical reference

If you need functionality from a legacy crate, consider:
1. Implementing it at the product layer
2. Creating a new, properly architected crate
3. Extracting and modernizing the relevant code
