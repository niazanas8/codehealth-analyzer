use clap::{Arg, Command};
use serde::Serialize;
use walkdir::WalkDir;
use syn::{visit::Visit, Stmt};
use std::fs;
use std::path::Path;

// --- Code metrics struct (overall totals) ---
#[derive(Default, Serialize, Clone)]
struct CodeMetrics {
    loc: usize,
    kloc: f64,
    cyclomatic_complexity: usize,
    functions: usize,
    comments: usize,
    longest_function_loc: usize,
    max_nesting_depth: usize,
    file_with_max_complexity: String,
    max_file_complexity: usize,
    halstead_operators: usize,
    halstead_operands: usize,
    halstead_unique_operators: usize,
    halstead_unique_operands: usize,
    cyclomatic_distribution: [usize; 3], // [<=5, 6-10, >10]
}

// --- New: per-function and per-file details ---
#[derive(Serialize, Clone)]
struct FunctionMetric {
    file: String,
    function: String,
    complexity: usize,
    loc: usize,
}

#[derive(Serialize, Clone, Default)]
struct FileMetrics {
    file: String,
    total_complexity: usize,
    functions: Vec<FunctionMetric>,
}

// --- Report structure for JSON export ---
#[derive(Serialize)]
struct Report {
    metrics: CodeMetrics,
    maintainability_index: f64,
    files: Vec<FileMetrics>,
    top_functions: Vec<FunctionMetric>,
}

// --- Cyclomatic complexity visitor ---
struct CyclomaticComplexityVisitor {
    complexity: usize,
    max_nesting: usize,
    current_nesting: usize,
}

impl CyclomaticComplexityVisitor {
    fn new() -> Self {
        Self {
            complexity: 1,
            max_nesting: 0,
            current_nesting: 0,
        }
    }
}

impl<'ast> Visit<'ast> for CyclomaticComplexityVisitor {
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if matches!(stmt, Stmt::Expr(expr, _) if matches!(
            expr,
            syn::Expr::If(_) | syn::Expr::Match(_) | syn::Expr::While(_) | syn::Expr::ForLoop(_)
        )) {
            self.complexity += 1;
        }

        if let Stmt::Expr(syn::Expr::Block(_), _) = stmt {
            self.current_nesting += 1;
            self.max_nesting = self.max_nesting.max(self.current_nesting);
        }

        syn::visit::visit_stmt(self, stmt);

        if let Stmt::Expr(syn::Expr::Block(_), _) = stmt {
            self.current_nesting -= 1;
        }
    }
}

// --- Analyze a single file ---
fn analyze_file(file_path: &Path) -> (CodeMetrics, FileMetrics) {
    let mut metrics = CodeMetrics::default();
    let mut file_detail = FileMetrics {
        file: file_path.to_string_lossy().to_string(),
        total_complexity: 0,
        functions: Vec::new(),
    };

    if let Ok(content) = fs::read_to_string(file_path) {
        metrics.loc = content.lines().count();
        metrics.comments = content
            .lines()
            .filter(|line| line.trim_start().starts_with("//"))
            .count();

        if let Ok(syntax) = syn::parse_file(&content) {
            for item in syntax.items {
                if let syn::Item::Fn(func) = item {
                    metrics.functions += 1;

                    let function_loc = func.block.stmts.len();
                    metrics.longest_function_loc =
                        metrics.longest_function_loc.max(function_loc);

                    let mut visitor = CyclomaticComplexityVisitor::new();
                    visitor.visit_item_fn(&func);

                    metrics.cyclomatic_complexity += visitor.complexity;
                    metrics.max_nesting_depth =
                        metrics.max_nesting_depth.max(visitor.max_nesting);

                    match visitor.complexity {
                        0..=5 => metrics.cyclomatic_distribution[0] += 1,
                        6..=10 => metrics.cyclomatic_distribution[1] += 1,
                        _ => metrics.cyclomatic_distribution[2] += 1,
                    }

                    // Add per-function record
                    file_detail.total_complexity += visitor.complexity;
                    let fname = func.sig.ident.to_string();
                    file_detail.functions.push(FunctionMetric {
                        file: file_detail.file.clone(),
                        function: fname,
                        complexity: visitor.complexity,
                        loc: function_loc,
                    });
                }
            }
        }
    }

    (metrics, file_detail)
}

// --- Analyze a directory ---
fn calculate_metrics(dir: &str) -> (CodeMetrics, Vec<FileMetrics>, Vec<FunctionMetric>) {
    let mut total = CodeMetrics::default();
    let mut files: Vec<FileMetrics> = Vec::new();
    let mut all_functions: Vec<FunctionMetric> = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let (fm, detail) = analyze_file(path);

            total.loc += fm.loc;
            total.cyclomatic_complexity += fm.cyclomatic_complexity;
            total.functions += fm.functions;
            total.comments += fm.comments;
            total.longest_function_loc = total.longest_function_loc.max(fm.longest_function_loc);
            total.max_nesting_depth = total.max_nesting_depth.max(fm.max_nesting_depth);

            for i in 0..3 {
                total.cyclomatic_distribution[i] += fm.cyclomatic_distribution[i];
            }

            if fm.cyclomatic_complexity > total.max_file_complexity {
                total.max_file_complexity = fm.cyclomatic_complexity;
                total.file_with_max_complexity = detail.file.clone();
            }

            files.push(detail.clone());
            all_functions.extend(detail.functions);
        }
    }

    total.kloc = total.loc as f64 / 1000.0;

    all_functions.sort_by(|a, b| b.complexity.cmp(&a.complexity));
    let top_functions = all_functions.into_iter().take(20).collect();

    (total, files, top_functions)
}

// --- Maintainability index ---
fn calculate_maintainability_index(metrics: &CodeMetrics) -> f64 {
    if metrics.functions == 0 {
        return 0.0;
    }

    let halstead_volume = (metrics.halstead_unique_operators
        + metrics.halstead_unique_operands) as f64
        * ((metrics.halstead_unique_operators + metrics.halstead_unique_operands) as f64).log2();
    let avg_cyclomatic = metrics.cyclomatic_complexity as f64 / metrics.functions as f64;

    let index = 171.0
        - 5.2 * halstead_volume.log2()
        - 0.23 * avg_cyclomatic
        - 16.2 * (metrics.loc as f64).log2();
    index.max(0.0).min(100.0)
}

// --- MAIN ---
fn main() {
    let matches = Command::new("CodeHealth Analyzer")
        .version("2.0")
        .author("Your Name <your.email@example.com>")
        .about("Scans codebases and reports metrics such as cyclomatic complexity, maintainability, and risk factors")
        .arg(
            Arg::new("path")
                .long("path")
                .default_value(".")
                .help("Path to the directory or file to analyze"),
        )
        .arg(
            Arg::new("report")
                .long("report")
                .value_parser(["text", "json"])
                .default_value("text")
                .help("Choose report format"),
        )
        .arg(
            Arg::new("max-complexity")
                .long("max-complexity")
                .value_parser(clap::value_parser!(u32))
                .help("Fail if max cyclomatic complexity exceeds this threshold"),
        )
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    let report = matches.get_one::<String>("report").unwrap();
    let max_complexity = matches.get_one::<u32>("max-complexity").copied();

    let (metrics, files, top_functions) = calculate_metrics(path);
    let maintainability_index = calculate_maintainability_index(&metrics);

    if report == "json" {
        let output = Report {
            metrics: metrics.clone(),
            maintainability_index,
            files,
            top_functions,
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("Code Metrics:");
        println!("Lines of Code (LOC): {}", metrics.loc);
        println!("KLOC: {:.2}", metrics.kloc);
        println!("Cyclomatic Complexity: {}", metrics.cyclomatic_complexity);
        println!(
            "Average Cyclomatic Complexity per Function: {:.2}",
            metrics.cyclomatic_complexity as f64 / metrics.functions.max(1) as f64
        );
        println!(
            "Cyclomatic Complexity Distribution: [Easy (<=5): {}, Moderate (6-10): {}, High (>10): {}]",
            metrics.cyclomatic_distribution[0],
            metrics.cyclomatic_distribution[1],
            metrics.cyclomatic_distribution[2]
        );
        println!("Number of Functions: {}", metrics.functions);
        println!("Longest Function (LOC): {}", metrics.longest_function_loc);
        println!("Maximum Nesting Depth: {}", metrics.max_nesting_depth);
        println!(
            "Comment Density: {:.2}%",
            metrics.comments as f64 / metrics.loc.max(1) as f64 * 100.0
        );
        println!(
            "Maintainability Index: {:.2} (0-100)",
            maintainability_index
        );
        println!(
            "File with Maximum Complexity: {}",
            metrics.file_with_max_complexity
        );
        println!(
            "Maximum Cyclomatic Complexity in a File: {}",
            metrics.max_file_complexity
        );

        // --- Top offenders list ---
        println!("\n⚠️ Top 5 Most Complex Functions:");
        for (i, f) in top_functions.iter().take(5).enumerate() {
            println!(
                "{}. {}::{} → complexity={} LOC={}",
                i + 1,
                f.file,
                f.function,
                f.complexity,
                f.loc
            );
        }
    }

    // Threshold warning for CI/CD
    if let Some(th) = max_complexity {
        if metrics.max_file_complexity as u32 > th {
            eprintln!(
                "⚠️  Maximum cyclomatic complexity ({}) exceeds threshold ({}).",
                metrics.max_file_complexity, th
            );
            std::process::exit(2);
        }
    }
}
