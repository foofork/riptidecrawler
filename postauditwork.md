🧭 Post-Audit Action Plan (Concise)
1️⃣ Categorize every finding

Go through the hygiene report and mark each as one of four types:

Code type	Meaning	Action
Develop	Needed feature or logic not finished yet	Add to roadmap / backlog
Gate	Valid only under certain features or targets	Add #[cfg(feature="…")] or similar
Keep	Intentional placeholder / trait stub	Add _var or #[allow(dead_code)] // TODO
Remove	Obsolete or redundant code	Delete safely
2️⃣ Build the roadmap (for “Develop”)

For every “Develop” item:

Create a short task entry like:
✅ pipeline.rs – implement convert_extracted_content() for normalized output

Group by crate or subsystem (api, pipeline, cli, etc.).

Track them in your roadmap or project board (GitHub issues, Notion, etc.).

Tag them feature:incomplete or wire-up.

Example roadmap section:

### Development Roadmap
- [ ] Finish convert_extracted_content() → pipeline.rs
- [ ] Wire update_wasm_memory_metrics() → metrics.rs
- [ ] Add CLI output for final_url and extraction_time

3️⃣ Apply quick code hygiene fixes

Prefix unused locals with _var.

Add #[allow(dead_code)] // TODO(<ticket>) above kept placeholders.

Gate unused feature code with #[cfg(feature="…")].

Remove obsolete functions, structs, or imports.

4️⃣ Verify and lock in
cargo check  --workspace --all-targets
cargo clippy --workspace --all-targets -D warnings
cargo test


Ensure the workspace compiles cleanly with minimal remaining warnings.

5️⃣ Deliver a summary

Add to code_hygiene_report.md:

## Summary
Develop: 6  |  Gate: 4  |  Keep: 10  |  Remove: 5

## Roadmap (Develop)
- [ ] implement convert_extracted_content()
- [ ] integrate wasm metrics
- [ ] log final_url / extraction_time

## Gated
- wasm metrics → feature `wasm-metrics`

## Kept
- reliable_extractor field → TODO(eventmesh-130)

## Removed
- old LegacyParser module


✅ In short:

Turn “dead code” into a decision.

Feed unfinished items into the development roadmap.

Gate or clean the rest.

Re-check until the build is clean.