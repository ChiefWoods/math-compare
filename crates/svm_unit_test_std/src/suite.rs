//! Per-(file, test) discovery + lazy build dispatch.
//!
//! The macro hands us `file!()` (typically workspace-root-relative) plus the
//! specific test `name` and the package's cargo env. We:
//!   1. find the file by walking up from `CARGO_MANIFEST_DIR` until
//!      `<dir>/<file>` resolves;
//!   2. parse it once per process to extract every `#[svm_test]` sibling and
//!      the surrounding `use`s/helpers (memoized per file);
//!   3. trigger [`crate::build_one_test`] for *just this test* (memoized per
//!      `(file, name)` pair so parallel sibling tests in the same process
//!      don't double-build).
//!
//! This means `cargo test add_mod_n` only ever compiles `add_mod_n.so`; its
//! sibling tests in the same file are never built unless they're also run.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use quote::ToTokens;

struct ParsedFile {
    cleaned_source: String,
}

/// Build (or return the cached path of) the SBPF program for the
/// `#[svm_test]` named `name` defined in `file`. Returns the absolute path
/// to its `.so`.
///
/// Thread-safe: a per-`(file, name)` `OnceLock` ensures the build happens
/// exactly once per process even when many sibling `#[test]`s race in
/// parallel.
pub fn ensure_test_built(
    file: &'static str,
    name: &'static str,
    manifest_dir: &str,
    pkg_name: &str,
    target_tmpdir: Option<&str>,
) -> &'static Path {
    type Cell = OnceLock<&'static Path>;
    type Reg = HashMap<(&'static str, &'static str), Arc<Cell>>;
    static REGISTRY: OnceLock<Mutex<Reg>> = OnceLock::new();
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));

    let cell: Arc<Cell> = {
        let mut guard = registry.lock().expect("svm_test registry poisoned");
        guard
            .entry((file, name))
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone()
    };

    cell.get_or_init(|| {
        let parsed = parse_file_cached(file, manifest_dir);
        let path = crate::builder::build_one_test(
            &parsed.cleaned_source,
            name,
            file,
            target_tmpdir,
            manifest_dir,
            pkg_name,
        );
        Box::leak(path.into_boxed_path())
    })
}

/// Parse a test file once per process. Multiple `#[svm_test]` siblings in
/// the same file all share the result.
fn parse_file_cached(file: &'static str, manifest_dir: &str) -> &'static ParsedFile {
    type Cell = OnceLock<&'static ParsedFile>;
    type Reg = HashMap<&'static str, Arc<Cell>>;
    static CACHE: OnceLock<Mutex<Reg>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let cell: Arc<Cell> = {
        let mut guard = cache.lock().expect("svm_test parse cache poisoned");
        guard
            .entry(file)
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone()
    };

    cell.get_or_init(|| {
        let abs = resolve_source_path(manifest_dir, file);
        let source = fs::read_to_string(&abs)
            .unwrap_or_else(|e| panic!("svm_test: read {}: {e}", abs.display()));
        let cleaned_source = parse_suite(&source)
            .unwrap_or_else(|e| panic!("svm_test: parse {}: {e}", abs.display()));
        Box::leak(Box::new(ParsedFile { cleaned_source }))
    })
}

/// Walk up from `CARGO_MANIFEST_DIR` until `<dir>/<file>` exists.
fn resolve_source_path(start_dir: &str, file: &str) -> PathBuf {
    let file_path = Path::new(file);
    if file_path.is_absolute() {
        if file_path.exists() {
            return file_path.to_path_buf();
        }
        panic!("svm_test: absolute file path `{file}` does not exist");
    }

    let mut dir = PathBuf::from(start_dir);
    loop {
        let candidate = dir.join(file);
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            panic!(
                "svm_test: could not locate source file `{file}` starting from `{start_dir}` (walked up to filesystem root)"
            );
        }
    }
}

/// Build the source string handed to `cargo build-sbf`: same items, with
/// `#[svm_test]` attributes stripped from each test fn and any
/// `use svm_unit_test::*;` removed (the SBPF crate doesn't depend on us).
fn parse_suite(source: &str) -> Result<String, syn::Error> {
    let file: syn::File = syn::parse_str(source)?;
    let mut out_items: Vec<String> = Vec::new();

    for item in &file.items {
        match item {
            syn::Item::Use(u) => {
                if !is_crate_use(u) {
                    out_items.push(u.to_token_stream().to_string());
                }
            }
            syn::Item::Fn(f) => {
                let mut clone = f.clone();
                if let Some(idx) = clone.attrs.iter().position(is_svm_test_attr) {
                    clone.attrs.remove(idx);
                }
                out_items.push(clone.to_token_stream().to_string());
            }
            other => out_items.push(other.to_token_stream().to_string()),
        }
    }

    Ok(out_items.join("\n"))
}

fn is_svm_test_attr(a: &syn::Attribute) -> bool {
    let p = a.path();
    if p.is_ident("svm_test") {
        return true;
    }
    p.segments
        .last()
        .map(|s| s.ident == "svm_test")
        .unwrap_or(false)
}

/// Filter `use svm_unit_test::...;` (and similar) out of the SBPF source —
/// the generated SBPF crate doesn't depend on `svm-unit-test`. Match by the
/// crate name (the first ident of the use tree), not the macro name.
fn is_crate_use(u: &syn::ItemUse) -> bool {
    fn first_ident(tree: &syn::UseTree) -> Option<String> {
        match tree {
            syn::UseTree::Path(p) => Some(p.ident.to_string()),
            syn::UseTree::Name(n) => Some(n.ident.to_string()),
            syn::UseTree::Rename(r) => Some(r.ident.to_string()),
            _ => None,
        }
    }
    matches!(first_ident(&u.tree).as_deref(), Some("svm_unit_test"))
}
