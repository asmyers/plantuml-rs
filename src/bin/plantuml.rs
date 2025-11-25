//! PlantUML CLI passthrough
//!
//! This binary passes all arguments directly to the bundled PlantUML JAR.
//! Run `plantuml-rs --help` to see PlantUML's help.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    // Get bundle paths (extracts on first run)
    let paths = match plantuml::get_bundle_paths() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error extracting PlantUML bundle: {}", e);
            return ExitCode::from(1);
        }
    };

    // Collect all arguments (skip the program name)
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Run: java -jar plantuml.jar <args...>
    let status = Command::new(&paths.java_exe)
        .arg("-jar")
        .arg(&paths.plantuml_jar)
        .args(&args)
        .status();

    match status {
        Ok(s) => {
            if let Some(code) = s.code() {
                ExitCode::from(code as u8)
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("Error running PlantUML: {}", e);
            ExitCode::from(1)
        }
    }
}
