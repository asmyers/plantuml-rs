use std::env;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;

const PLANTUML_VERSION: &str = "1.2025.10";
const PLANTUML_JAR_URL: &str = "https://github.com/plantuml/plantuml/releases/download/v1.2025.10/plantuml-1.2025.10.jar";

// Eclipse Temurin JRE 21 for Windows x64 (LTS version)
const JRE_URL: &str = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jre_x64_windows_hotspot_21.0.5_11.zip";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let binaries_dir = Path::new(&out_dir).join("binaries");

    // Create binaries directory if it doesn't exist
    fs::create_dir_all(&binaries_dir).expect("Failed to create binaries directory");

    let bundle_zip = binaries_dir.join("plantuml-bundle.zip");

    // Check if bundle already exists
    if bundle_zip.exists() {
        println!("cargo:warning=PlantUML bundle already exists at {:?}", bundle_zip);
        return;
    }

    // Download PlantUML JAR
    println!("cargo:warning=Downloading PlantUML JAR v{}...", PLANTUML_VERSION);
    let jar_bytes = download_file(PLANTUML_JAR_URL);
    println!("cargo:warning=Downloaded PlantUML JAR: {} bytes", jar_bytes.len());

    // Download JRE
    println!("cargo:warning=Downloading Eclipse Temurin JRE 21...");
    let jre_bytes = download_file(JRE_URL);
    println!("cargo:warning=Downloaded JRE: {} bytes", jre_bytes.len());

    // Create the bundle ZIP containing JRE + JAR
    println!("cargo:warning=Creating bundle ZIP...");
    create_bundle_zip(&bundle_zip, &jar_bytes, &jre_bytes);

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

    response.bytes().expect("Failed to read response body").to_vec()
}

fn create_bundle_zip(output_path: &Path, jar_bytes: &[u8], jre_zip_bytes: &[u8]) {
    use zip::write::SimpleFileOptions;

    let file = File::create(output_path).expect("Failed to create bundle ZIP");
    let mut zip_writer = zip::ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

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
        let mut file = jre_archive.by_index(i).expect("Failed to read JRE ZIP entry");
        let name = file.name().to_string();

        // Skip the top-level directory (jdk-21.0.5+11-jre/)
        let relative_path = name
            .split_once('/')
            .map(|(_, rest)| rest)
            .unwrap_or(&name);

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
