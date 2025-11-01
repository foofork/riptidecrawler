#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

OUT="$ROOT/unused_audit.csv"
TMP="$(mktemp -d)"

# Collect messages across typical build surfaces
echo "[audit] collecting diagnostics…"
cargo check  --workspace --all-targets --message-format=json >"$TMP/base.json" || true
cargo check  --workspace --all-targets --no-default-features --message-format=json >"$TMP/no_feat.json" || true
cargo check  --workspace --all-targets --all-features --message-format=json >"$TMP/all_feat.json" || true
cargo test   --no-run --message-format=json >"$TMP/tests.json" || true
# wasm is optional; uncomment if you target it:
# cargo check --target wasm32-unknown-unknown --all-features --message-format=json >"$TMP/wasm.json" || true

# Merge & normalize
jq -s '
  map(.[] | select(.reason=="compiler-message" and .message.code!=null))
  | map({
      code: .message.code.code,
      level: .message.level,
      msg: .message.message,
      file: (.message.spans[0].file_name // ""),
      line: (.message.spans[0].line_start // 0),
      rendered: (.message.rendered // "")
    })
  | map(select(.code|test("^(unused_|dead_code|unused_imports|unused_must_use)")))
' "$TMP/base.json" "$TMP/no_feat.json" "$TMP/all_feat.json" "$TMP/tests.json" ${TMP}/wasm.json 2>/dev/null \
> "$TMP/all.json"

# Best-effort symbol extraction from rendered msg
jq -r '
  ["code","level","file","line","symbol","message","configs_seen","decision","action"],
  ( group_by(.code + "|" + .file + "|" + (.line|tostring) + "|" + .msg)
    | .[]
    | .[0] as $x
    | $x.rendered as $r
    | ($r|capture("`(?<sym>[^`]+)`")?.sym // "") as $sym
    | [
        $x.code,
        $x.level,
        $x.file,
        ($x.line|tostring),
        $sym,
        ($x.msg|gsub("[\\r\\n]"; " ")),
        "base/no_feat/all_feat/tests",  # quick note of where we looked
        "",  # decision (KEEP / GATE / WIRE / REMOVE)
        ""   # action (e.g., #[cfg(feature="wasm-metrics")] / call fn X / rename to _var)
      ] | @csv )
' "$TMP/all.json" > "$OUT"

echo "[audit] wrote $OUT"
