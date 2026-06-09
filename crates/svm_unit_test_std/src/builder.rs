//! Lazy SBPF compilation triggered by `#[svm_test]`.
//!
//! Each test gets its own `#![no_std]` cdylib crate generated under
//! `target/tmp/suite-<hash-of-file-path>/build/<test-name>/`. Before
//! shelling out to `cargo build-sbf` we stat-compare the existing `.so`
//! against the generated lib.rs/Cargo.toml plus the user's package source
//! tree — if nothing is newer than the `.so`, the spawn is skipped entirely.
//! This keeps the no-op path on the order of milliseconds rather than
//! hundreds.

use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

/// Build (if needed) and return the path to `<name>.so` for the given
/// `#[svm_test]` fn. Idempotent and concurrency-safe; the caller (suite.rs)
/// further memoizes per `(file, name)` so this is invoked at most once per
/// test per process.
pub fn build_one_test(
    source: &str,
    name: &str,
    file: &str,
    target_tmpdir: Option<&str>,
    manifest_dir: &str,
    pkg_name: &str,
) -> PathBuf {
    // Hash the *file path*, not the source — the suite directory needs to
    // be stable across cargo runs so cargo's incremental fingerprinting
    // inside it stays warm.
    let mut hasher = DefaultHasher::new();
    file.hash(&mut hasher);
    let suite_id = format!("{:016x}", hasher.finish());

    let workspace_root = match target_tmpdir {
        Some(t) => PathBuf::from(t),
        None => PathBuf::from(manifest_dir)
            .join("target")
            .join("svm-unit-tests"),
    };
    let work = workspace_root.join(format!("suite-{suite_id}"));
    let so_dir = work.join("so");
    let build_dir = work.join("build");
    fs::create_dir_all(&so_dir).expect("create so dir");

    let pkg_path = PathBuf::from(manifest_dir);
    build_one(&build_dir, &so_dir, name, source, &pkg_path, pkg_name);

    so_dir.join(format!("{name}.so"))
}

fn build_one(
    build_dir: &Path,
    so_dir: &Path,
    name: &str,
    source: &str,
    pkg_path: &Path,
    pkg_name: &str,
) {
    let crate_name = format!("svm_test_{name}");
    let crate_dir = build_dir.join(name);
    fs::create_dir_all(crate_dir.join("src")).expect("create per-test crate dir");

    let rel_pkg = pathdiff::diff_paths(pkg_path, &crate_dir)
        .expect("relative path to user package")
        .display()
        .to_string()
        .replace('\\', "/");

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
{pkg_name} = {{ path = "{rel_pkg}" }}
solana-program-error = {{ version = "3", default-features = false }}

[workspace]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
overflow-checks = false
panic = "abort"
strip = "symbols"
"#
    );
    write_if_changed(&crate_dir.join("Cargo.toml"), &cargo_toml);

    let lib_rs = format!(
        r#"#![allow(unused_imports, dead_code)]

// Lets a #[svm_test] body return `()` (→ success, code 0), a `u64` program
// return code, or a `Result<T, E: Into<ProgramError>>` — so an
// `#[svm_test(error = ...)]` body can just be the fallible op itself (`Ok` →
// success, `Err(e)` → its `ProgramError` code, which `error = <ProgramError>`
// matches). The `Into<ProgramError>` bound means any idiomatic Solana error
// type works with no extra impls.
trait _SvmTestReturnCode {{ fn _svm_test_return_code(self) -> u64; }}
impl _SvmTestReturnCode for () {{ fn _svm_test_return_code(self) -> u64 {{ 0 }} }}
impl _SvmTestReturnCode for u64 {{ fn _svm_test_return_code(self) -> u64 {{ self }} }}
impl<T, E: Into<solana_program_error::ProgramError>> _SvmTestReturnCode for Result<T, E> {{
    fn _svm_test_return_code(self) -> u64 {{
        match self {{
            Ok(_) => 0,
            Err(e) => {{
                let p: solana_program_error::ProgramError = e.into();
                p.into()
            }}
        }}
    }}
}}

{source}

#[unsafe(no_mangle)]
pub extern "C" fn entrypoint(_input: *mut u8) -> u64 {{
    _SvmTestReturnCode::_svm_test_return_code({name}())
}}
"#
    );
    write_if_changed(&crate_dir.join("src").join("lib.rs"), &lib_rs);

    let so_path = so_dir.join(format!("{name}.so"));
    if !needs_rebuild(&so_path, &crate_dir, pkg_path) {
        return;
    }

    let manifest = crate_dir.join("Cargo.toml");
    let target_dir = crate_dir.join("target");

    let status = Command::new("cargo")
        .arg("build-sbf")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--sbf-out-dir")
        .arg(so_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .status()
        .expect("spawn `cargo build-sbf`");

    assert!(
        status.success(),
        "`cargo build-sbf` failed for svm_test `{name}` (manifest: {})",
        manifest.display(),
    );

    // build-sbf emits `<crate_name>.so`; the macro expects `<name>.so`.
    // fs::rename replaces the destination on Unix, which is what we want
    // — the new build supersedes any stale prior artifact.
    let produced = so_dir.join(format!("{crate_name}.so"));
    if produced != so_path && produced.exists() {
        fs::rename(&produced, &so_path).unwrap_or_else(|e| {
            if so_path.exists() {
                // Lost a race with a parallel cargo-test process; drop the
                // duplicate and use the existing one.
                let _ = fs::remove_file(&produced);
            } else {
                panic!(
                    "rename {} -> {}: {e}",
                    produced.display(),
                    so_path.display(),
                );
            }
        });
    }
    assert!(
        so_path.exists(),
        "expected `{}` to exist after build-sbf",
        so_path.display(),
    );
}

/// Decide whether to invoke `cargo build-sbf` at all. We approximate cargo's
/// own fingerprint check with mtimes:
///   * the `.so` must exist;
///   * the generated Cargo.toml + lib.rs must not be newer than it;
///   * nothing in the user's package source tree (Cargo.toml + src/**) may
///     be newer than it.
///
/// Misses transitive dep changes that cargo would catch (e.g. a Cargo.lock
/// bump in another workspace member). For those, run `cargo clean` or just
/// touch the test file. The trade-off pays for itself: the no-op path drops
/// from a multi-hundred-ms `cargo build-sbf` spawn to a few stat() calls.
fn needs_rebuild(so_path: &Path, crate_dir: &Path, pkg_path: &Path) -> bool {
    let Some(so_mtime) = mtime(so_path) else {
        return true;
    };

    for f in [
        crate_dir.join("Cargo.toml"),
        crate_dir.join("src").join("lib.rs"),
    ] {
        if mtime(&f).is_some_and(|m| m > so_mtime) {
            return true;
        }
    }

    if mtime(&pkg_path.join("Cargo.toml")).is_some_and(|m| m > so_mtime) {
        return true;
    }

    let src = pkg_path.join("src");
    if newest_mtime_under(&src).is_some_and(|m| m > so_mtime) {
        return true;
    }

    false
}

fn mtime(p: &Path) -> Option<SystemTime> {
    p.metadata().ok()?.modified().ok()
}

fn newest_mtime_under(dir: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    walk_files(dir, &mut |path| {
        if let Some(m) = mtime(path) {
            newest = Some(newest.map_or(m, |n| n.max(m)));
        }
    });
    newest
}

fn walk_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, cb);
        } else {
            cb(&path);
        }
    }
}

/// Write `contents` to `path` only if it would change. Avoids bumping
/// mtime on no-op runs (which would defeat our `needs_rebuild` check).
fn write_if_changed(path: &Path, contents: &str) {
    if let Ok(existing) = fs::read_to_string(path)
        && existing == contents
    {
        return;
    }
    fs::write(path, contents).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}
