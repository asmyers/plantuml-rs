use once_cell::sync::OnceCell;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use crate::error::{PlantUmlError, Result};

/// Embedded PlantUML bundle (JRE + JAR)
const PLANTUML_BUNDLE: &[u8] = include_bytes!("../binaries/plantuml-bundle.zip");

/// Version string for cache directory
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cached path to the extracted bundle directory
static EXTRACTED_DIR: OnceCell<PathBuf> = OnceCell::new();

/// Paths to the Java executable and PlantUML JAR
pub struct BundlePaths {
    pub java_exe: PathBuf,
    pub plantuml_jar: PathBuf,
}

/// Get the paths to the Java executable and PlantUML JAR.
///
/// The bundle is extracted to the user's cache directory on first call.
/// Subsequent calls return the cached paths.
pub fn get_bundle_paths() -> Result<BundlePaths> {
    let dir = EXTRACTED_DIR.get_or_try_init(|| extract_bundle())?;

    Ok(BundlePaths {
        java_exe: dir.join("jre").join("bin").join("java.exe"),
        plantuml_jar: dir.join("plantuml.jar"),
    })
}

/// Extract the embedded bundle to the cache directory.
fn extract_bundle() -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    let java_exe = cache_dir.join("jre").join("bin").join("java.exe");
    let jar_path = cache_dir.join("plantuml.jar");

    // If both main files exist, assume extraction is complete
    if java_exe.exists() && jar_path.exists() {
        return Ok(cache_dir);
    }

    // Create cache directory if needed
    fs::create_dir_all(&cache_dir).map_err(PlantUmlError::BinaryExtraction)?;

    // Extract the bundle ZIP
    let cursor = Cursor::new(PLANTUML_BUNDLE);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| PlantUmlError::BinaryExtraction(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| PlantUmlError::BinaryExtraction(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let name = file.name().to_string();

        // Skip directories (we'll create them when needed)
        if name.ends_with('/') {
            let dir_path = cache_dir.join(&name);
            fs::create_dir_all(&dir_path).map_err(PlantUmlError::BinaryExtraction)?;
            continue;
        }

        let output_path = cache_dir.join(&name);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(PlantUmlError::BinaryExtraction)?;
        }

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(PlantUmlError::BinaryExtraction)?;

        let mut output_file =
            fs::File::create(&output_path).map_err(PlantUmlError::BinaryExtraction)?;
        output_file
            .write_all(&contents)
            .map_err(PlantUmlError::BinaryExtraction)?;
    }

    Ok(cache_dir)
}

/// Get the cache directory for this version of the library.
fn get_cache_dir() -> Result<PathBuf> {
    let base = dirs::cache_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(std::env::temp_dir);

    Ok(base.join("plantuml-rs").join(VERSION))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_embedded() {
        // Verify the bundle is embedded and has reasonable size
        assert!(!PLANTUML_BUNDLE.is_empty(), "Bundle should not be empty");
        assert!(
            PLANTUML_BUNDLE.len() > 10_000_000,
            "Bundle should be at least 10MB (JRE + JAR)"
        );
    }
}
