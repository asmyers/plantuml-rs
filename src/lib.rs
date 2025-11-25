//! PlantUML Rust wrapper with bundled JRE.
//!
//! This crate provides a zero-dependency way to render PlantUML diagrams.
//! A JRE and PlantUML JAR are bundled and extracted on first use.
//!
//! # Example
//!
//! ```no_run
//! let svg = plantuml::render("@startuml\nAlice -> Bob: Hello\n@enduml").unwrap();
//! println!("{}", svg);
//! ```

mod binary;
mod error;
mod executor;

pub use binary::{get_bundle_paths, BundlePaths};
pub use error::{PlantUmlError, Result};

use std::fs;
use std::path::Path;

/// Render PlantUML syntax to an SVG string.
///
/// # Arguments
///
/// * `plantuml` - PlantUML source code as a string
///
/// # Returns
///
/// The rendered SVG as a string.
///
/// # Example
///
/// ```no_run
/// let svg = plantuml::render(r#"
/// @startuml
/// Alice -> Bob: Hello
/// Bob -> Alice: Hi!
/// @enduml
/// "#).unwrap();
/// ```
pub fn render(plantuml: &str) -> Result<String> {
    executor::execute_pipe(plantuml)
}

/// Render a PlantUML file to an SVG file.
///
/// # Arguments
///
/// * `input` - Path to the input PlantUML file
/// * `output` - Path where the SVG output will be written
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// plantuml::render_file(
///     Path::new("diagram.puml"),
///     Path::new("diagram.svg")
/// ).unwrap();
/// ```
pub fn render_file(input: &Path, output: &Path) -> Result<()> {
    let plantuml = fs::read_to_string(input).map_err(|source| PlantUmlError::InputRead {
        path: input.to_path_buf(),
        source,
    })?;

    let svg = executor::execute_pipe(&plantuml)?;

    fs::write(output, &svg).map_err(|source| PlantUmlError::OutputWrite {
        path: output.to_path_buf(),
        source,
    })?;

    Ok(())
}

/// Render PlantUML syntax to an SVG file.
///
/// # Arguments
///
/// * `plantuml` - PlantUML source code as a string
/// * `output` - Path where the SVG output will be written
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// plantuml::render_to_file(
///     "@startuml\nAlice -> Bob: Hello\n@enduml",
///     Path::new("diagram.svg")
/// ).unwrap();
/// ```
pub fn render_to_file(plantuml: &str, output: &Path) -> Result<()> {
    let svg = executor::execute_pipe(plantuml)?;

    fs::write(output, &svg).map_err(|source| PlantUmlError::OutputWrite {
        path: output.to_path_buf(),
        source,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let input = "@startuml\nAlice -> Bob: Hello\n@enduml";
        let result = render(input);
        assert!(result.is_ok(), "Failed to render: {:?}", result);

        let svg = result.unwrap();
        assert!(svg.contains("<svg"), "Output should be SVG");
    }

    #[test]
    fn test_render_sequence_diagram() {
        let input = r#"@startuml
participant Alice
participant Bob
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response
@enduml"#;

        let result = render(input);
        assert!(
            result.is_ok(),
            "Failed to render sequence diagram: {:?}",
            result
        );
    }

    #[test]
    fn test_render_class_diagram() {
        let input = r#"@startuml
class Car {
  +String make
  +String model
  +start()
  +stop()
}
@enduml"#;

        let result = render(input);
        assert!(
            result.is_ok(),
            "Failed to render class diagram: {:?}",
            result
        );
    }
}
