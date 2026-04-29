# Contributing

## Development

```bash
cargo build --release
cargo test
```

The binary is a single `src/main.rs` with no external dependencies — just `std`.

---

## Release pipeline

Releases are fully automated via three GitHub Actions workflows:

| Workflow | Trigger | Tool | Purpose |
|----------|---------|------|---------|
| `release-plz.yml` | push to `main` | [`release-plz`](https://github.com/release-plz/release-plz) | Open a release PR (version bump + changelog). Merging it tags + creates a GitHub Release + `cargo publish`. |
| `release.yml` | push of `v*` tag | [`taiki-e/upload-rust-binary-action`](https://github.com/taiki-e/upload-rust-binary-action) | Cross-compile 8 targets and upload tarballs/zips to the GitHub Release. |
| `homebrew-bump.yml` | release published | [`dawidd6/action-homebrew-bump-formula`](https://github.com/dawidd6/action-homebrew-bump-formula) | Open a PR on [`FGRibreau/homebrew-tap`](https://github.com/FGRibreau/homebrew-tap) bumping the formula. |

### Required GitHub Environment

All publish secrets live in a GitHub Environment named **`release`** (Settings → Environments → New environment). The `release-plz` and `homebrew-bump` jobs reference it via `environment: release`. The matrix-build `release.yml` only uses `GITHUB_TOKEN` and needs no environment.

Recommended protection rules on the `release` environment:

- **Required reviewers**: yourself — every crates.io publish then waits for an approval click in the Actions UI
- **Deployment branches and tags**: `main` + tag pattern `v*`

### Required secrets (inside the `release` environment)

| Secret | Used by | How to get it |
|--------|---------|---------------|
| `CARGO_REGISTRY_TOKEN` | release-plz | https://crates.io/settings/tokens — scope: `publish-new`, `publish-update` |
| `RELEASE_PLZ_TOKEN` | release-plz | A GitHub PAT (classic) with `repo` scope, **not** the default `GITHUB_TOKEN` (so the tag push it creates triggers `release.yml`). |
| `HOMEBREW_TAP_TOKEN` | homebrew-bump | A GitHub PAT (classic) with `public_repo` scope on `FGRibreau/homebrew-tap`. |

### First-time bootstrap

The Homebrew formula doesn't exist yet in the tap — `dawidd6/action-homebrew-bump-formula` only **bumps** an existing formula. To bootstrap:

1. Cut the first release (push a `v0.1.0` tag, or merge the release-plz PR).
2. Wait for `release.yml` to upload the source tarball asset.
3. Generate the initial formula and push it to the tap:
   ```bash
   brew tap FGRibreau/tap
   brew create https://github.com/FGRibreau/cargo-clean-targets/archive/refs/tags/v0.1.0.tar.gz \
     --tap FGRibreau/tap --set-name cargo-clean-targets
   # edit the generated Formula/cargo-clean-targets.rb, then commit + push
   ```
4. Subsequent releases will be bumped automatically by `homebrew-bump.yml`.
