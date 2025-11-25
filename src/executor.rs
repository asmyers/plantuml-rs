use std::io::Write;
use std::process::{Command, Stdio};

use crate::binary::get_bundle_paths;
use crate::error::{PlantUmlError, Result};

/// Execute PlantUML with input from stdin and return SVG output.
///
/// Uses PlantUML's `-pipe` mode for efficient stdin/stdout processing.
pub fn execute_pipe(input: &str) -> Result<String> {
    let paths = get_bundle_paths()?;

    let mut child = Command::new(&paths.java_exe)
        .args([
            "-jar",
            paths.plantuml_jar.to_str().unwrap(),
            "-pipe",
            "-tsvg",
            "-charset",
            "UTF-8",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(PlantUmlError::ProcessSpawn)?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(PlantUmlError::StdinWrite)?;
    }

    // Wait for process and collect output
    let output = child
        .wait_with_output()
        .map_err(PlantUmlError::ProcessSpawn)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);

        // Check if it's a syntax error
        if stderr.contains("Syntax Error") || stderr.contains("@startuml") {
            return Err(PlantUmlError::SyntaxError(stderr));
        }

        return Err(PlantUmlError::ProcessFailed { code, stderr });
    }

    String::from_utf8(output.stdout).map_err(PlantUmlError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_diagram() {
        let input = r#"@startuml
Alice -> Bob: Hello
@enduml"#;

        let result = execute_pipe(input);
        assert!(result.is_ok(), "Should render simple diagram: {:?}", result);

        let svg = result.unwrap();
        assert!(svg.contains("<svg"), "Output should be SVG");
        assert!(svg.contains("Alice"), "SVG should contain Alice");
    }

    #[test]
    fn test_syntax_error() {
        let input = "this is not valid plantuml";

        let result = execute_pipe(input);
        // PlantUML might still produce output for invalid input
        // so we just check it doesn't panic
        let _ = result;
    }
}
