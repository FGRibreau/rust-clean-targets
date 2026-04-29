<div align="center">

# cargo-clean-targets

**Strip Cargo `target/` caches across a whole tree, keep only the compiled binaries.**

<br/>

<img src="assets/banner.svg" alt="cargo-clean-targets — walk, detect, prune Cargo target directories" width="780"/>

<br/>

[![Crates.io](https://img.shields.io/crates/v/cargo-clean-targets.svg)](https://crates.io/crates/cargo-clean-targets)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](#)
[![Release](https://github.com/FGRibreau/cargo-clean-targets/actions/workflows/release.yml/badge.svg)](https://github.com/FGRibreau/cargo-clean-targets/actions/workflows/release.yml)

</div>

---

## Sponsors

<table>
  <tr>
    <td align="center" width="200">
        <a href="https://getnatalia.com/">
        <img src="assets/sponsors/natalia.svg" height="60" alt="Natalia"/><br/>
        <b>Natalia</b>
        </a><br/>
        <sub>24/7 AI voice and whatsapp agent for customer services</sub>
    </td>
    <td align="center" width="200">
      <a href="https://nobullshitconseil.com/">
        <img src="assets/sponsors/nobullshitconseil.svg" height="60" alt="NoBullshitConseil"/><br/>
        <b>NoBullshitConseil</b>
      </a><br/>
      <sub>360° tech consulting</sub>
    </td>
    <td align="center" width="200">
      <a href="https://www.hook0.com/">
        <img src="assets/sponsors/hook0.png" height="60" alt="Hook0"/><br/>
        <b>Hook0</b>
      </a><br/>
      <sub>Open-Source Webhooks-as-a-Service</sub>
    </td>
    <td align="center" width="200">
      <a href="https://france-nuage.fr/">
        <img src="assets/sponsors/france-nuage.png" height="60" alt="France-Nuage"/><br/>
        <b>France-Nuage</b>
      </a><br/>
      <sub>Sovereign cloud hosting in France</sub>
    </td>
  </tr>
</table>

> **Interested in sponsoring?** [Get in touch](mailto:rust@fgribreau.com)

---

## What is this?

A tiny Cargo subcommand that walks a directory, finds every Cargo `target/` it can prove is one (`CACHEDIR.TAG` or `.rustc_info.json`), and removes the cache subtrees — `deps/`, `build/`, `incremental/`, `.fingerprint/`, `doc/`, dep-info `.d` files — while leaving the actually-compiled binaries (`release/app`, `debug/app`, examples, dylibs, rlibs, wasm) untouched.

Useful when you have a `~/code` folder full of half-built Rust projects eating tens of GB and you don't want to nuke `target/` whole because you want yesterday's release build to still run.

---

## Why I built this

I had a hunch Cargo's `target/` directories were quietly eating my Mac's disk. I'd been chasing disk hogs with [dumap](https://github.com/jrobhoward/dumap) for a while — Docker volumes, `node_modules`, old Xcode runtimes were the obvious offenders, but Rust caches scattered across years of side projects were the ones I kept missing.

So on the morning of April 29, 2026, I sat down and wrote this. Walk every `target/` on the machine, prune the cache, keep the binaries. First run:

```
done: 127 target dir(s), 4199 path(s) removed, 773.17 GiB freed
```

**773 GiB!** I had to re-read that line three times. Almost a full terabyte of dead weight on a single laptop! `deps/`, `build/`, `incremental/`, `.fingerprint/` had quietly piled up to nearly 700 GB across 127 target dirs without me ever noticing!

If I had this much sitting on my disk, I can't be the only one. So I open-sourced it the same morning.

— François-Guillaume Ribreau ([fgribreau.com](https://fgribreau.com))

---

## Features

- **Walks recursively** — point it at any folder, it finds every Cargo target inside
- **Keeps the binaries** — executables, dylibs, rlibs, wasm stay; only cache goes
- **Safe detection** — only touches dirs marked by Cargo (`CACHEDIR.TAG` / `.rustc_info.json`)
- **Dry mode** — `--dry` previews directories that would be removed before doing anything
- **Verbose mode** — `--dry --verbose` lists every individual file too
- **Reclaim report** — final summary shows target dirs touched, paths removed, bytes freed
- **Cargo-native** — invokable as `cargo clean-targets` (single static binary, no runtime, no deps)

---

## Install

### Cargo

```bash
cargo install cargo-clean-targets
```

Then use it as a Cargo subcommand:

```bash
cargo clean-targets ~/code
```

### Homebrew (macOS / Linux)

```bash
brew tap FGRibreau/tap
brew install cargo-clean-targets
```

### `cargo binstall` (prebuilt binary, no compile)

```bash
cargo binstall cargo-clean-targets
```

### Prebuilt binaries

Grab a tarball/zip for your platform from [Releases](https://github.com/FGRibreau/cargo-clean-targets/releases). Targets shipped:

`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`.

### From source

```bash
git clone https://github.com/FGRibreau/cargo-clean-targets.git
cd cargo-clean-targets
cargo build --release
./target/release/cargo-clean-targets --dry ~/code
```

---

## Quick Start

```bash
# preview (recommended first run)
cargo clean-targets --dry ~/code

# do it
cargo clean-targets ~/code
```

You can also run the binary directly without going through Cargo:

```bash
cargo-clean-targets --dry ~/code
```

---

## Usage

```
cargo clean-targets [OPTIONS] <PATH>
cargo-clean-targets [OPTIONS] <PATH>
```

### Options

| Flag | Description |
|------|-------------|
| `-n`, `--dry` | Preview only — list directories (and root files of `target/`) that would be removed. Alias: `--dry-run` |
| `-v`, `--verbose` | Print every path acted on, including individual files inside cache subdirs |
| `-h`, `--help` | Show usage |

### Example

```bash
$ cargo clean-targets --dry ~/code
target: /Users/me/code/myproj/target
  would rm dir  /Users/me/code/myproj/target/release/.fingerprint  (12.04 MiB)
  would rm dir  /Users/me/code/myproj/target/release/incremental   (48.21 MiB)
  would rm dir  /Users/me/code/myproj/target/release/deps          (612.80 MiB)
  would rm dir  /Users/me/code/myproj/target/release/build         (4.10 MiB)

done: 1 target dir(s), 5 path(s) removed, 677.82 MiB freed (dry) (+1 file(s) not shown — pass --verbose to list)

Thanks for using cargo-clean-targets!
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
| Target-triple wrappers (`x86_64-unknown-linux-gnu/`, `aarch64-apple-darwin/`, …) | descended into, then each profile inside is cleaned |
| `doc/`, `package/`, `tmp/`, `.rustdoc_fingerprint/` | removed |
| `sqlx-tmp/` (from `cargo sqlx prepare`) | removed |
| `cargo-timings/` (from `cargo build --timings`) | removed |
| `CACHEDIR.TAG`, `.rustc_info.json` | kept |

A directory is treated as a Cargo target only if it contains `CACHEDIR.TAG` or `.rustc_info.json`. A folder named `target/` that isn't actually one (e.g. some unrelated project's data dir) is left alone.

Inside a target-triple wrapper (`target/<triple>/`), the same profile-cleaning logic applies — `target/<triple>/release/myapp` is kept, the surrounding `deps/`, `build/`, `incremental/`, `.fingerprint/` are removed.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions and the release pipeline.

---

## License

[MIT](LICENSE) © François-Guillaume Ribreau ( https://fgribreau.com )
