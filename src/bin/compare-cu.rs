use std::{
    collections::{BTreeSet, HashMap},
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitCode},
};

struct Library {
    canonical: &'static str,
    package: &'static str,
    test_target: &'static str,
}

const LIBRARIES: &[(&str, Library)] = &[
    (
        "bigdecimal",
        Library {
            canonical: "bigdecimal",
            package: "cu_bigdecimal",
            test_target: "bigdecimal",
        },
    ),
    (
        "fixed",
        Library {
            canonical: "fixed",
            package: "cu_fixed",
            test_target: "fixed",
        },
    ),
    (
        "rust-decimal",
        Library {
            canonical: "rust_decimal",
            package: "cu_rust_decimal",
            test_target: "rust_decimal",
        },
    ),
    (
        "rust_decimal",
        Library {
            canonical: "rust_decimal",
            package: "cu_rust_decimal",
            test_target: "rust_decimal",
        },
    ),
    (
        "hylo-fix",
        Library {
            canonical: "hylo_fix",
            package: "cu_hylo_fix",
            test_target: "hylo_fix",
        },
    ),
    (
        "hylo_fix",
        Library {
            canonical: "hylo_fix",
            package: "cu_hylo_fix",
            test_target: "hylo_fix",
        },
    ),
    (
        "spl-math",
        Library {
            canonical: "spl_math",
            package: "cu_spl_math",
            test_target: "spl_math",
        },
    ),
    (
        "spl_math",
        Library {
            canonical: "spl_math",
            package: "cu_spl_math",
            test_target: "spl_math",
        },
    ),
];

struct Args {
    raw: bool,
    save: Option<PathBuf>,
    libraries: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let libraries = canonical_libraries(&args.libraries)?;

    if args.raw {
        for library in &libraries {
            let (_, lines) = run_library(library)?;
            for line in lines {
                println!("{line}");
            }
        }
        return Ok(());
    }

    if libraries.len() < 2 {
        return Err("compare requires at least two distinct libraries".to_string());
    }

    let mut results = Vec::new();
    for library in &libraries {
        results.push((library.clone(), run_library(library)?.0));
    }

    println!("{}", render_table(&results));

    if let Some(path) = args.save {
        save_csv(&path, &results).map_err(|error| format!("save {}: {error}", path.display()))?;
        println!("\nsaved CSV to {}", path.display());
    }

    Ok(())
}

fn parse_args() -> Result<Args, String> {
    let mut raw = false;
    let mut save = None;
    let mut libraries = Vec::new();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--raw" => raw = true,
            "--save" => {
                let path = args
                    .next()
                    .ok_or_else(|| "--save requires a path".to_string())?;
                save = Some(PathBuf::from(path));
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option `{arg}`")),
            _ => libraries.push(arg),
        }
    }

    if libraries.is_empty() {
        return Err("usage: compare-cu [--raw] [--save PATH] <libraries...>".to_string());
    }

    Ok(Args {
        raw,
        save,
        libraries,
    })
}

fn print_help() {
    println!("Compare svm-unit-test CU output across math libraries.");
    println!();
    println!("Usage: compare-cu [--raw] [--save PATH] <libraries...>");
    println!();
    println!("Libraries: bigdecimal, fixed, rust-decimal, hylo-fix, spl-math");
}

fn canonical_libraries(names: &[String]) -> Result<Vec<String>, String> {
    let mut libraries = Vec::new();
    for name in names {
        let library = library_for(name).ok_or_else(|| {
            format!(
                "unknown library `{name}`. Choose from: bigdecimal, fixed, rust-decimal, hylo-fix, spl-math"
            )
        })?;
        if !libraries.iter().any(|item| item == library.canonical) {
            libraries.push(library.canonical.to_string());
        }
    }
    Ok(libraries)
}

fn library_for(name: &str) -> Option<&'static Library> {
    LIBRARIES
        .iter()
        .find_map(|(key, library)| (*key == name).then_some(library))
}

fn library_by_canonical(canonical: &str) -> &'static Library {
    LIBRARIES
        .iter()
        .find_map(|(_, library)| (library.canonical == canonical).then_some(library))
        .expect("canonical library is known")
}

fn run_library(library: &str) -> Result<(HashMap<String, u64>, Vec<String>), String> {
    let library_config = library_by_canonical(library);
    let output = Command::new("cargo")
        .args([
            "test",
            "-p",
            library_config.package,
            "--test",
            library_config.test_target,
            "--",
            "--nocapture",
        ])
        .output()
        .map_err(|error| format!("spawn cargo test for {library}: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    if !output.status.success() {
        eprint!("{combined}");
        return Err(format!("cargo test failed for {library}"));
    }

    let rows = parse_cu(&combined, library);
    if rows.is_empty() {
        eprint!("{combined}");
        return Err(format!(
            "no CU rows parsed for {library}; inspect the output above and update parsing if svm-unit-test changed its format"
        ));
    }

    Ok((rows, cu_lines(&combined)))
}

fn cu_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(parse_primary_cu_line)
        .map(|(test, cu)| format!("svm_test `{test}` => {cu} CUs"))
        .collect()
}

fn parse_cu(output: &str, library: &str) -> HashMap<String, u64> {
    let mut rows = HashMap::new();
    for line in output.lines() {
        if let Some((test_name, cu)) = parse_primary_cu_line(line) {
            let test_name = test_name.rsplit("::").next().unwrap_or(test_name);
            rows.insert(
                operation_from_test(test_name, library).to_string(),
                cu_number(cu),
            );
        }
    }
    rows
}

fn parse_primary_cu_line(line: &str) -> Option<(&str, &str)> {
    let start = line.find("svm_test `")? + "svm_test `".len();
    let rest = &line[start..];
    let end = rest.find('`')?;
    let test_name = &rest[..end];
    let after_name = &rest[end + 1..];
    let marker = "=>";
    let cu_start = after_name.find(marker)? + marker.len();
    let after_marker = after_name[cu_start..].trim_start();
    let cu_end = after_marker.find(" CUs")?;
    Some((test_name, after_marker[..cu_end].trim()))
}

fn cu_number(cu: &str) -> u64 {
    cu.chars()
        .filter(|character| *character != '_' && *character != ',')
        .collect::<String>()
        .parse()
        .expect("CU value is numeric")
}

fn operation_from_test<'a>(test_name: &'a str, library: &str) -> &'a str {
    test_name
        .strip_prefix(&format!("{library}_"))
        .unwrap_or(test_name)
        .pipe(operation_alias)
}

fn operation_alias(operation: &str) -> &str {
    match operation {
        "exp" | "exp_approximation" => "exp_or_approximation",
        _ => operation,
    }
}

fn render_table(results: &[(String, HashMap<String, u64>)]) -> String {
    let operations = results
        .iter()
        .flat_map(|(_, rows)| rows.keys().cloned())
        .collect::<BTreeSet<_>>();

    let mut table = Vec::new();
    table.push(
        std::iter::once("operation".to_string())
            .chain(results.iter().map(|(library, _)| library.clone()))
            .collect::<Vec<_>>(),
    );

    for operation in operations {
        let mut row = vec![operation.clone()];
        for (_, rows) in results {
            row.push(rows.get(&operation).map(u64::to_string).unwrap_or_default());
        }
        table.push(row);
    }

    let widths = column_widths(&table);
    let mut lines = Vec::new();
    for (index, row) in table.iter().enumerate() {
        lines.push(render_row(row, &widths));
        if index == 0 {
            lines.push(
                widths
                    .iter()
                    .map(|width| "-".repeat(*width))
                    .collect::<Vec<_>>()
                    .join("  "),
            );
        }
    }
    lines.join("\n")
}

fn column_widths(table: &[Vec<String>]) -> Vec<usize> {
    let column_count = table.first().map(Vec::len).unwrap_or_default();
    (0..column_count)
        .map(|column| table.iter().map(|row| row[column].len()).max().unwrap_or(0))
        .collect()
}

fn render_row(row: &[String], widths: &[usize]) -> String {
    row.iter()
        .enumerate()
        .map(|(index, cell)| format!("{cell:<width$}", width = widths[index]))
        .collect::<Vec<_>>()
        .join("  ")
}

fn save_csv(path: &PathBuf, results: &[(String, HashMap<String, u64>)]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let operations = results
        .iter()
        .flat_map(|(_, rows)| rows.keys().cloned())
        .collect::<BTreeSet<_>>();

    let mut file = File::create(path)?;
    writeln!(
        file,
        "{}",
        std::iter::once("operation".to_string())
            .chain(results.iter().map(|(library, _)| csv_escape(library)))
            .collect::<Vec<_>>()
            .join(",")
    )?;

    for operation in operations {
        let row = std::iter::once(csv_escape(&operation))
            .chain(
                results
                    .iter()
                    .map(|(_, rows)| rows.get(&operation).map(u64::to_string).unwrap_or_default()),
            )
            .collect::<Vec<_>>();
        writeln!(file, "{}", row.join(","))?;
    }

    Ok(())
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::library_for;

    #[test]
    fn recognizes_bigdecimal() {
        let library = library_for("bigdecimal").expect("bigdecimal library is registered");

        assert_eq!(library.canonical, "bigdecimal");
        assert_eq!(library.package, "cu_bigdecimal");
        assert_eq!(library.test_target, "bigdecimal");
    }
}
