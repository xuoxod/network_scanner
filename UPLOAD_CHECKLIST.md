
# Upload checklist for rmediatech (quick)

1. Run the full test suite

   ```bash
   cargo test --workspace
   ```

2. Build release artifacts for crates you intend to publish

   ```bash
   cargo build --workspace --release
   ```

3. Optional linters / tidying

   - Fix remaining non-fatal warnings (dead_code, unused_imports) if desired.
   - Run `cargo clippy --workspace` and address high-confidence suggestions.

4. Documentation

   - Ensure `crates/discovery/README.md` documents the new `--out-legacy` and `--out-target` behavior (done).
   - Confirm top-level README mentions build/test instructions.

5. Metadata

   - Ensure `Cargo.toml` version fields are correct for crates you will publish.
   - Ensure LICENSE is present and correct.

6. Commit & tag

   ```bash
   git add .
   git commit -m "Prepare release: add legacy exporter and companion JSONs"
   git tag -a v0.1.0 -m "release v0.1.0"
   git push --follow-tags
   ```

7. Upload / publish

   - For crates.io publish, run `cargo publish -p <crate>` after ensuring API token is configured.
   - Or push the repo to your hosting provider (GitHub/GitLab) as appropriate.

Notes

- This project currently passes unit/integration tests for `crates/io` and `crates/discovery`. Remaining non-fatal warnings are documented in the repository; fixing them is optional for an initial upload.
