
# Rust Code Hygiene Audit — Definitive Instructions

## Goal

Find and triage **unused variables/imports**, **dead/unreachable code**, **TODO/FIXME debt**, **broad suppressions**, **unused dependencies**, and **cfg-gated stragglers**. For each finding, decide: **WIRE / GATE / KEEP / REMOVE**. Produce a short report.

---

## 0) Run from repo root (collect signals)

```bash
set -e

# A. Core compiler + Clippy (json)
cargo check  --workspace --all-targets --message-format=json > .check.json || true
cargo clippy --workspace --all-targets --message-format=json > .clippy.json || true

# B. Feature/target sweeps (to detect cfg-only usage)
cargo check --workspace --all-targets --no-default-features --message-format=json > .check_nofeat.json || true
cargo check --workspace --all-targets --all-features        --message-format=json > .check_allfeat.json || true
cargo test  --no-run --message-format=json > .check_tests.json || true
# Example If relevant to the repo:
# rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
# cargo check --target wasm32-unknown-unknown --all-features --message-format=json > .check_wasm.json || true

# C. Extract “unused/dead/unreachable” from all runs
jq -s '
  map(.[] | select(.reason=="compiler-message" and .message.code!=null))
  | map({
      code: .message.code.code,
      level: .message.level,
      file: (.message.spans[0].file_name // ""),
      line: (.message.spans[0].line_start // 0),
      msg: .message.message,
      rendered: (.message.rendered // "")
    })
  | map(select(.code|test("^(unused_|dead_code|unused_imports|unused_must_use|unreachable_code)$")))
' .check.json .clippy.json .check_nofeat.json .check_allfeat.json .check_tests.json 2>/dev/null \
> .unused_all.json
```

---

## 1) Scan for TODO/FIXME and risky suppressions

```bash
# TODO/FIXME/XXX
rg -n --no-ignore -S 'TODO|FIXME|XXX' --glob '!target' > .todos.txt || true

# Commented-out code heuristic (optional signal)
rg -n --no-ignore -S '^\s*//.*' --glob '!target' | rg -n 'fn|struct|impl|unsafe|#\[' > .commented_code.txt || true

# Broad crate-wide or local allows/denies (can hide issues)
rg -n --no-ignore -S '#!\[(allow|deny|warn)\(' --glob '!target' > .crate_attrs.txt || true
rg -n --no-ignore -S '#\[allow\(' --glob '!target' > .local_allows.txt || true
```

---

## 2) Unused dependencies

```bash
cargo install cargo-udeps >/dev/null 2>&1 || true
cargo udeps --workspace --all-targets > .udeps.txt || true
```

---

## 3) For each finding, apply the decision tree

Open each entry from `.unused_all.json` (use the `file` and `line`), then:

1. **Search usage**:

```bash
rg -n "\b<SYMBOL>\b" --glob '!target'
```

2. **Check configs**: If the warning disappears under any of these,
   it’s config-specific (so **GATE** it):

```bash
cargo check --workspace --all-targets --no-default-features
cargo check --workspace --all-targets --all-features
cargo test --no-run
# and optionally wasm target if applicable
```

3. **Classify & act** (pick one):

* **WIRE** – it should be used.

  * Use/log/propagate (`?`, `.await`, include in output), or call the helper.
* **GATE** – valid only for certain features/targets.

  * Add precise `#[cfg(feature="…")]`, `#[cfg(test)]`, `#[cfg(target_arch="…")]`.
* **KEEP** – intentionally unused for future work or trait signatures.

  * Locals: `let var` → `let _var`.
  * Functions/fields: `#[allow(dead_code)] // TODO:<ticket/ref>`.
  * Keep allows **narrow and local** (avoid crate-wide).
* **REMOVE** – obsolete.

  * Delete and re-check builds; fix any references.

> Notes:
>
> * For parameters required by a trait/handler, rename to `_param`.
> * For `#[must_use]` results, either use (`?`) or explicitly ignore: `let _ = call();` or `drop(call());`.

---

## 4) Apply edits (minimal & reversible)

* Prefer small, local changes (underscore, narrow `#[allow]`, small `#[cfg]`).
* Avoid blanket `#![allow(unused)]` at crate root.
* When gating, consider moving feature-specific chunks into a `mod` behind `#[cfg(feature="…")]`.

---

## 5) Re-verify (clean build gate)

```bash
cargo check  --workspace --all-targets
cargo clippy --workspace --all-targets -D warnings
cargo test
```

---

## 6) Dependencies cleanup

From `.udeps.txt`:

* If truly unused → remove from `Cargo.toml`.
* If only used under a feature/target → move under `[features]` and gate uses in code.

---

## 7) Deliverables

Create `code_hygiene_report.md` (short, actionable):

* Date/time
* Counts: total findings, and numbers **WIRE / GATE / KEEP / REMOVE**
* Top files touched
* Items deferred with TODOs (include ticket/owner)
* Unused dependencies removed (list)

Example:

```bash
echo "# Code Hygiene Report" > code_hygiene_report.md
echo "- Date: $(date -Iseconds)" >> code_hygiene_report.md
echo "- Findings: $(jq 'length' .unused_all.json) (unused/dead/unreachable)" >> code_hygiene_report.md
echo "- TODOs found: $(wc -l < .todos.txt 2>/dev/null || echo 0)" >> code_hygiene_report.md
echo "- Unused deps (udeps): $(rg -c '^warning: ' .udeps.txt 2>/dev/null || echo 0)" >> code_hygiene_report.md
```

---

## Commit policy

* Bundle mechanical edits by crate/module (small PRs).
* PR title tags: `[WIRE]`, `[GATE]`, `[KEEP]`, `[REMOVE]` as relevant.
* CI: enforce `cargo clippy --workspace --all-targets -D warnings` on main branch.

---