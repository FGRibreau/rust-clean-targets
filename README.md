<div align="center">

# rust-clean-targets

**Strip Cargo `target/` caches across a whole tree, keep only the compiled binaries.**

<br/>

<img src="assets/banner.svg" alt="rust-clean-targets — walk, detect, prune Cargo target directories" width="780"/>

<br/>

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](#)

</div>

---

## What is this?

A tiny CLI that walks a directory, finds every Cargo `target/` it can prove is one (`CACHEDIR.TAG` or `.rustc_info.json`), and removes the cache subtrees — `deps/`, `build/`, `incremental/`, `.fingerprint/`, `doc/`, dep-info `.d` files — while leaving the actually-compiled binaries (`release/app`, `debug/app`, examples, dylibs, rlibs, wasm) untouched.

Useful when you have a `~/code` folder full of half-built Rust projects eating tens of GB and you don't want to nuke `target/` whole because you want yesterday's release build to still run.

## Features

- **Walks recursively** — point it at any folder, it finds every Cargo target inside
- **Keeps the binaries** — executables, dylibs, rlibs, wasm stay; only cache goes
- **Safe detection** — only touches dirs marked by Cargo (`CACHEDIR.TAG` / `.rustc_info.json`)
- **Dry mode** — `--dry` previews directories that would be removed before doing anything
- **Verbose mode** — `--dry --verbose` lists every individual file too
- **Reclaim report** — final summary shows target dirs touched, paths removed, bytes freed
- **Single static binary** — no runtime, no config, no dependencies

---

## Quick Start

```bash
git clone https://github.com/FGRibreau/rust-clean-targets.git
cd rust-clean-targets
cargo build --release

# preview (recommended first run)
./target/release/rust-clean-targets --dry ~/code

# do it
./target/release/rust-clean-targets ~/code
```

---

## Usage

```
rust-clean-targets [OPTIONS] <PATH>
```

### Options

| Flag | Description |
|------|-------------|
| `-n`, `--dry` | Preview only — list directories (and root files of `target/`) that would be removed. Alias: `--dry-run` |
| `-v`, `--verbose` | Print every path acted on, including individual files inside cache subdirs |
| `-h`, `--help` | Show usage |

### Example

```bash
$ rust-clean-targets --dry ~/code
target: /Users/me/code/myproj/target
  would rm dir  /Users/me/code/myproj/target/release/.fingerprint  (12.04 MiB)
  would rm dir  /Users/me/code/myproj/target/release/incremental   (48.21 MiB)
  would rm dir  /Users/me/code/myproj/target/release/deps          (612.80 MiB)
  would rm dir  /Users/me/code/myproj/target/release/build         (4.10 MiB)

done: 1 target dir(s), 5 path(s) removed, 677.82 MiB freed (dry) (+1 file(s) not shown — pass --verbose to list)

Thanks for using rust-clean-targets!
— François-Guillaume Ribreau ( https://fgribreau.com )
```

---

## What it keeps vs removes

| Inside `target/<profile>/` | Action |
|------|--------|
| Top-level executables (no ext, `.exe`) | **kept** |
| Dynamic libs (`.so`, `.dylib`, `.dll`) | **kept** |
| Static libs (`.a`, `.lib`, `.rlib`) | **kept** |
| WebAssembly (`.wasm`) | **kept** |
| `examples/` final binaries | **kept** |
| `deps/`, `build/`, `incremental/`, `.fingerprint/` | removed |
| `examples/{deps,build,incremental,.fingerprint}/` | removed |
| Top-level `.d`, `.rmeta`, `.pdb`, `.o`, `.obj` | removed |

| Inside `target/` | Action |
|------|--------|
| Profile dirs (`debug/`, `release/`, custom) | descended into |
| `doc/`, `package/`, `tmp/`, `.rustdoc_fingerprint/` | removed |
| `CACHEDIR.TAG`, `.rustc_info.json` | kept |

A directory is treated as a Cargo target only if it contains `CACHEDIR.TAG` or `.rustc_info.json`. A folder named `target/` that isn't actually one (e.g. some unrelated project's data dir) is left alone.

---

## Development

```bash
cargo build --release
cargo test
```

The binary is a single `src/main.rs` with no external dependencies — just `std`.

---

## License

[MIT](LICENSE) © François-Guillaume Ribreau ( https://fgribreau.com )
