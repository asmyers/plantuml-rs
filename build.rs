use std::env;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;

const PLANTUML_VERSION: &str = "1.2025.10";
const PLANTUML_JAR_URL: &str =
    "https://github.com/plantuml/plantuml/releases/download/v1.2025.10/plantuml-1.2025.10.jar";

// Eclipse Temurin JRE 21 URLs for each platform
const JRE_URL_WINDOWS_X64: &str = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jre_x64_windows_hotspot_21.0.5_11.zip";
const JRE_URL_LINUX_X64: &str = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jre_x64_linux_hotspot_21.0.5_11.tar.gz";
const JRE_URL_LINUX_AARCH64: &str = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jre_aarch64_linux_hotspot_21.0.5_11.tar.gz";
const JRE_URL_MACOS_X64: &str = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jre_x64_mac_hotspot_21.0.5_11.tar.gz";
const JRE_URL_MACOS_AARCH64: &str = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jre_aarch64_mac_hotspot_21.0.5_11.tar.gz";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let (jre_url, is_tarball, is_macos) = match (target_os.as_str(), target_arch.as_str()) {
        ("windows", "x86_64") => (JRE_URL_WINDOWS_X64, false, false),
        ("linux", "x86_64") => (JRE_URL_LINUX_X64, true, false),
        ("linux", "aarch64") => (JRE_URL_LINUX_AARCH64, true, false),
        ("macos", "x86_64") => (JRE_URL_MACOS_X64, true, true),
        ("macos", "aarch64") => (JRE_URL_MACOS_AARCH64, true, true),
        _ => {
            panic!(
                "Unsupported platform: {}-{}. Supported: windows-x86_64, linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64",
                target_os, target_arch
            );
        }
    };

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let binaries_dir = Path::new(&out_dir).join("binaries");

    // Create binaries directory if it doesn't exist
    fs::create_dir_all(&binaries_dir).expect("Failed to create binaries directory");

    let bundle_zip = binaries_dir.join("plantuml-bundle.zip");

    // Check if bundle already exists
    if bundle_zip.exists() {
        println!(
            "cargo:warning=PlantUML bundle already exists at {:?}",
            bundle_zip
        );
        return;
    }

    // Download PlantUML JAR
    println!(
        "cargo:warning=Downloading PlantUML JAR v{}...",
        PLANTUML_VERSION
    );
    let jar_bytes = download_file(PLANTUML_JAR_URL);
    println!(
        "cargo:warning=Downloaded PlantUML JAR: {} bytes",
        jar_bytes.len()
    );

    // Download JRE
    println!(
        "cargo:warning=Downloading Eclipse Temurin JRE 21 for {}-{}...",
        target_os, target_arch
    );
    let jre_bytes = download_file(jre_url);
    println!("cargo:warning=Downloaded JRE: {} bytes", jre_bytes.len());

    // Create the bundle ZIP containing JRE + JAR
    println!("cargo:warning=Creating bundle ZIP...");
    if is_tarball {
        create_bundle_zip_from_tarball(&bundle_zip, &jar_bytes, &jre_bytes, is_macos);
    } else {
        create_bundle_zip_from_zip(&bundle_zip, &jar_bytes, &jre_bytes);
    }

    println!("cargo:warning=Bundle created at {:?}", bundle_zip);
}

fn download_file(url: &str) -> Vec<u8> {
    let response = reqwest::blocking::Client::builder()
        .user_agent("plantuml-rs-build")
        .build()
        .expect("Failed to create HTTP client")
        .get(url)
        .send()
        .expect("Failed to download file");

    if !response.status().is_success() {
        panic!("Failed to download {}: HTTP {}", url, response.status());
    }

    response
        .bytes()
        .expect("Failed to read response body")
        .to_vec()
}

fn create_bundle_zip_from_zip(output_path: &Path, jar_bytes: &[u8], jre_zip_bytes: &[u8]) {
    use zip::write::SimpleFileOptions;

    let file = File::create(output_path).expect("Failed to create bundle ZIP");
    let mut zip_writer = zip::ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Add the PlantUML JAR
    zip_writer
        .start_file("plantuml.jar", options)
        .expect("Failed to add JAR to bundle");
    zip_writer
        .write_all(jar_bytes)
        .expect("Failed to write JAR to bundle");

    // Extract JRE ZIP and add contents to bundle
    let cursor = Cursor::new(jre_zip_bytes);
    let mut jre_archive = zip::ZipArchive::new(cursor).expect("Failed to open JRE ZIP");

    for i in 0..jre_archive.len() {
        let mut file = jre_archive
            .by_index(i)
            .expect("Failed to read JRE ZIP entry");
        let name = file.name().to_string();

        // Skip the top-level directory (jdk-21.0.5+11-jre/)
        let relative_path = name.split_once('/').map(|(_, rest)| rest).unwrap_or(&name);

        if relative_path.is_empty() {
            continue;
        }

        let bundle_path = format!("jre/{}", relative_path);

        if name.ends_with('/') {
            // Directory
            zip_writer
                .add_directory(&bundle_path, options)
                .expect("Failed to add directory to bundle");
        } else {
            // File
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .expect("Failed to read JRE file");

            zip_writer
                .start_file(&bundle_path, options)
                .expect("Failed to add file to bundle");
            zip_writer
                .write_all(&contents)
                .expect("Failed to write file to bundle");
        }
    }

    zip_writer.finish().expect("Failed to finalize bundle ZIP");
}

fn create_bundle_zip_from_tarball(
    output_path: &Path,
    jar_bytes: &[u8],
    jre_tarball_bytes: &[u8],
    is_macos: bool,
) {
    use flate2::read::GzDecoder;
    use tar::Archive;
    use zip::write::SimpleFileOptions;

    let file = File::create(output_path).expect("Failed to create bundle ZIP");
    let mut zip_writer = zip::ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Add the PlantUML JAR
    zip_writer
        .start_file("plantuml.jar", options)
        .expect("Failed to add JAR to bundle");
    zip_writer
        .write_all(jar_bytes)
        .expect("Failed to write JAR to bundle");

    // Extract JRE tarball and add contents to bundle
    let cursor = Cursor::new(jre_tarball_bytes);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    for entry in archive.entries().expect("Failed to read tarball entries") {
        let mut entry = entry.expect("Failed to read tarball entry");
        let path = entry
            .path()
            .expect("Failed to get entry path")
            .to_path_buf();
        let path_str = path.to_string_lossy().to_string();

        // Skip the top-level directory (jdk-21.0.5+11-jre/)
        let relative_path = path_str
            .split_once('/')
            .map(|(_, rest)| rest)
            .unwrap_or(&path_str);

        if relative_path.is_empty() {
            continue;
        }

        // macOS JRE has an extra Contents/Home directory structure
        let relative_path = if is_macos {
            relative_path
                .strip_prefix("Contents/Home/")
                .or_else(|| relative_path.strip_prefix("Contents/Home"))
                .unwrap_or(relative_path)
        } else {
            relative_path
        };

        // Skip if empty after stripping macOS prefix
        if relative_path.is_empty() || relative_path == "Contents" || relative_path == "Contents/" {
            continue;
        }

        let bundle_path = format!("jre/{}", relative_path);

        if entry.header().entry_type().is_dir() {
            zip_writer
                .add_directory(&bundle_path, options)
                .expect("Failed to add directory to bundle");
        } else if entry.header().entry_type().is_file() {
            let mut contents = Vec::new();
            entry
                .read_to_end(&mut contents)
                .expect("Failed to read tarball file");

            // Preserve executable bit via Unix mode
            #[cfg(unix)]
            let options = {
                let mode = entry.header().mode().unwrap_or(0o644);
                options.unix_permissions(mode)
            };

            zip_writer
                .start_file(&bundle_path, options)
                .expect("Failed to add file to bundle");
            zip_writer
                .write_all(&contents)
                .expect("Failed to write file to bundle");
        }
        // Skip symlinks and other types for now
    }

    zip_writer.finish().expect("Failed to finalize bundle ZIP");
}
