use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug)]
struct Args {
    root: PathBuf,
    dry: bool,
    verbose: bool,
}

fn print_usage(program: &str) {
    eprintln!(
        "rust-clean-targets — strip Cargo target/ caches, keep compiled binaries\n\n\
         Usage:\n  \
         {program} [OPTIONS] <PATH>\n\n\
         Options:\n  \
         -n, --dry            Show what would be deleted without deleting.\n      \
                          Lists directories and root-level files of target/.\n      \
                          Combine with --verbose to also list every nested file.\n      \
                          Alias: --dry-run\n  \
         -v, --verbose    Print every path acted on (including files inside\n      \
                          cache subdirectories)\n  \
         -h, --help       Show this help"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut iter = env::args();
    let program = iter.next().unwrap_or_else(|| "rust-clean-targets".into());
    let mut root: Option<PathBuf> = None;
    let mut dry = false;
    let mut verbose = false;

    for arg in iter {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage(&program);
                std::process::exit(0);
            }
            "-n" | "--dry" | "--dry-run" => dry = true,
            "-v" | "--verbose" => verbose = true,
            s if s.starts_with('-') => return Err(format!("unknown flag: {s}")),
            _ => {
                if root.is_some() {
                    return Err("only one PATH argument is supported".into());
                }
                root = Some(PathBuf::from(arg));
            }
        }
    }

    let root = root.ok_or_else(|| {
        print_usage(&program);
        "missing <PATH> argument".to_string()
    })?;

    Ok(Args {
        root,
        dry,
        verbose,
    })
}

fn is_cargo_target_dir(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    if dir.file_name().and_then(|s| s.to_str()) != Some("target") {
        return false;
    }
    // Strong signals that this is a Cargo target dir.
    dir.join("CACHEDIR.TAG").exists() || dir.join(".rustc_info.json").exists()
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut total: u64 = 0;
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(0);
    }
    if meta.is_file() {
        return Ok(meta.len());
    }
    if meta.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            match dir_size(&p) {
                Ok(n) => total += n,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
    }
    Ok(total)
}

fn human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", v, UNITS[i])
    }
}

fn looks_like_binary_artifact(path: &Path) -> bool {
    // Files at the root of a profile dir produced by Cargo as final outputs.
    // We keep:
    //   - executables (no extension or `.exe`)
    //   - dynamic libs: .so, .dylib, .dll
    //   - static libs : .a, .lib, .rlib
    //   - WebAssembly: .wasm
    //   - macOS framework dirs are uncommon at root; skip them.
    // We DROP:
    //   - .d (dep-info), .rmeta, .pdb, .o, .obj
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    if name.starts_with('.') {
        return false; // .cargo-lock, .fingerprint sentinels
    }
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    match ext {
        "" | "exe" | "so" | "dylib" | "dll" | "a" | "lib" | "rlib" | "wasm" => true,
        _ => false,
    }
}

#[derive(Default, Debug)]
struct Stats {
    targets_seen: u64,
    bytes_freed: u64,
    paths_removed: u64,
    hidden_paths: u64,
}

/// `prominent` = this path is a directory, OR a direct child of `target/`.
/// Default `--dry` only prints prominent paths; `--verbose` prints all.
fn remove_path(path: &Path, prominent: bool, args: &Args, stats: &mut Stats) -> io::Result<()> {
    let size = dir_size(path).unwrap_or(0);
    let should_print = args.verbose || (args.dry && prominent);
    if should_print {
        let kind = if path.is_dir() { "dir " } else { "file" };
        println!(
            "  {} {} {}  ({})",
            if args.dry { "would rm" } else { "rm     " },
            kind,
            path.display(),
            human(size)
        );
    }
    if !args.dry {
        let meta = fs::symlink_metadata(path)?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    stats.bytes_freed += size;
    stats.paths_removed += 1;
    if !prominent {
        stats.hidden_paths += 1;
    }
    Ok(())
}

/// Inside a profile dir (e.g. `target/release`), drop the build cache and
/// keep only top-level final artifacts plus `examples/` final artifacts.
fn clean_profile_dir(profile: &Path, args: &Args, stats: &mut Stats) -> io::Result<()> {
    if !profile.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(profile)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            match name.as_ref() {
                // Pure caches — always remove. Directories are prominent.
                "build" | "deps" | "incremental" | ".fingerprint" => {
                    remove_path(&path, true, args, stats)?;
                }
                // Examples can contain final binaries; clean their internal cache
                // but keep top-level files.
                "examples" => {
                    clean_examples_dir(&path, args, stats)?;
                }
                _ => {
                    // Unknown subdir (could be a workspace member's nested
                    // artifact dir). Be conservative — leave it alone.
                }
            }
        } else if file_type.is_file() {
            // Drop dep-info / metadata files at the top of the profile dir.
            let drop_ext = matches!(
                path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                "d" | "rmeta" | "pdb" | "o" | "obj"
            );
            if drop_ext || !looks_like_binary_artifact(&path) {
                if name == ".cargo-lock" {
                    // Tiny lock file, leave it so cargo doesn't re-create noise.
                    continue;
                }
                // Files inside a profile dir are not prominent — only --verbose lists them.
                remove_path(&path, false, args, stats)?;
            }
        }
    }
    Ok(())
}

fn clean_examples_dir(examples: &Path, args: &Args, stats: &mut Stats) -> io::Result<()> {
    for entry in fs::read_dir(examples)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            // examples/build, examples/deps, etc. — caches. Dirs are prominent.
            if matches!(name.as_ref(), "build" | "deps" | "incremental" | ".fingerprint") {
                remove_path(&path, true, args, stats)?;
            }
        } else if file_type.is_file() {
            let drop_ext = matches!(
                path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                "d" | "rmeta" | "pdb" | "o" | "obj"
            );
            if drop_ext || !looks_like_binary_artifact(&path) {
                remove_path(&path, false, args, stats)?;
            }
        }
    }
    Ok(())
}

fn clean_target_dir(target: &Path, args: &Args, stats: &mut Stats) -> io::Result<()> {
    println!("target: {}", target.display());
    stats.targets_seen += 1;

    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            match name.as_ref() {
                // Always-cache directories at the top of target/. Prominent.
                "doc" | "package" | "tmp" | ".rustdoc_fingerprint" => {
                    remove_path(&path, true, args, stats)?;
                }
                // Profile dirs: debug, release, plus any custom profile.
                _ => {
                    clean_profile_dir(&path, args, stats)?;
                }
            }
        } else if file_type.is_file() {
            // Top-level files in target/ are all metadata; keep them.
            // (CACHEDIR.TAG, .rustc_info.json, .package-cache)
        }
    }
    Ok(())
}

fn walk(root: &Path, args: &Args, stats: &mut Stats) -> io::Result<()> {
    let meta = match fs::symlink_metadata(root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("skip {}: {}", root.display(), e);
            return Ok(());
        }
    };
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Ok(());
    }

    if is_cargo_target_dir(root) {
        clean_target_dir(root, args, stats)?;
        // Don't descend into a cleaned target/ — its subtrees are profile dirs.
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("skip entry under {}: {}", root.display(), e);
                continue;
            }
        };
        let ft = match entry.file_type() {
            Ok(f) => f,
            Err(_) => continue,
        };
        if ft.is_dir() {
            walk(&entry.path(), args, stats)?;
        }
    }
    Ok(())
}

fn run() -> Result<(Args, Stats), String> {
    let args = parse_args()?;
    if !args.root.exists() {
        return Err(format!("path does not exist: {}", args.root.display()));
    }
    let mut stats = Stats::default();
    walk(&args.root, &args, &mut stats).map_err(|e| format!("walk failed: {e}"))?;
    Ok((args, stats))
}

fn main() -> ExitCode {
    match run() {
        Ok((args, stats)) => {
            let hidden_hint = if args.dry && !args.verbose && stats.hidden_paths > 0 {
                format!(
                    " (+{} file(s) not shown — pass --verbose to list)",
                    stats.hidden_paths
                )
            } else {
                String::new()
            };
            println!(
                "\ndone: {} target dir(s), {} path(s) removed, {} freed{}{}",
                stats.targets_seen,
                stats.paths_removed,
                human(stats.bytes_freed),
                if args.dry { " (dry)" } else { "" },
                hidden_hint,
            );
            println!(
                "\nThanks for using rust-clean-targets!\n— François-Guillaume Ribreau ( https://fgribreau.com )"
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
