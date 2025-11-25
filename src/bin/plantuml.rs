//! PlantUML CLI tool
//!
//! Usage:
//!   plantuml <input.puml> [output.svg]
//!   plantuml --stdin

use std::env;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

fn print_usage() {
    eprintln!("PlantUML Rust wrapper");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  plantuml <input.puml> [output.svg]  Render a PlantUML file to SVG");
    eprintln!("  plantuml --stdin                    Read from stdin, write SVG to stdout");
    eprintln!("  plantuml --help                     Show this help message");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "--stdin" => {
            // Read from stdin, write to stdout
            let mut input = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut input) {
                eprintln!("Error reading stdin: {}", e);
                return ExitCode::from(1);
            }

            match plantuml::render(&input) {
                Ok(svg) => {
                    if let Err(e) = io::stdout().write_all(svg.as_bytes()) {
                        eprintln!("Error writing to stdout: {}", e);
                        return ExitCode::from(1);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error rendering diagram: {}", e);
                    ExitCode::from(1)
                }
            }
        }
        input_path => {
            let input = Path::new(input_path);

            // Determine output path
            let output_path = if args.len() >= 3 {
                args[2].clone()
            } else {
                // Default: replace extension with .svg
                let stem = input.file_stem().unwrap_or_default().to_string_lossy();
                format!("{}.svg", stem)
            };
            let output = Path::new(&output_path);

            match plantuml::render_file(input, output) {
                Ok(()) => {
                    eprintln!("Rendered {} -> {}", input.display(), output.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::from(1)
                }
            }
        }
    }
}
