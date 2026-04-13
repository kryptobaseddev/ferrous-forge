//! Embedded template for Rust embedded projects

use crate::templates::{
    BuiltinTemplate, TemplateFile, TemplateKind, TemplateManifest, TemplateVariable,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Create the embedded template
pub fn create_embedded_template() -> BuiltinTemplate {
    let manifest = create_embedded_manifest();
    let files = create_embedded_files();

    BuiltinTemplate { manifest, files }
}

/// Create the manifest for embedded template
fn create_embedded_manifest() -> TemplateManifest {
    let mut manifest = TemplateManifest::new("embedded".to_string(), TemplateKind::Embedded);

    manifest.description = "Rust embedded project with no_std support".to_string();
    manifest.author = "Ferrous Forge Team".to_string();

    add_embedded_variables(&mut manifest);
    add_embedded_files(&mut manifest);
    add_embedded_commands(&mut manifest);

    manifest
}

/// Add variables to the embedded manifest
fn add_embedded_variables(manifest: &mut TemplateManifest) {
    manifest.add_variable(TemplateVariable::required(
        "project_name".to_string(),
        "Name of the embedded project".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "author".to_string(),
        "Author name".to_string(),
        "Unknown".to_string(),
    ));

    manifest.add_variable(TemplateVariable::optional(
        "target_chip".to_string(),
        "Target microcontroller".to_string(),
        "cortex-m3".to_string(),
    ));
}

/// Add files to the embedded manifest
fn add_embedded_files(manifest: &mut TemplateManifest) {
    manifest.add_file(TemplateFile::new(
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/main.rs"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from("memory.x"),
        PathBuf::from("memory.x"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from(".cargo/config.toml"),
        PathBuf::from(".cargo/config.toml"),
    ));

    manifest.add_file(TemplateFile::new(
        PathBuf::from(".ferrous-forge/config.toml"),
        PathBuf::from(".ferrous-forge/config.toml"),
    ));
}

/// Add post-generate commands to the embedded manifest
fn add_embedded_commands(manifest: &mut TemplateManifest) {
    manifest.add_post_generate("cargo fmt".to_string());
    manifest.add_post_generate("cargo check".to_string());
}

/// Create the files for embedded template
fn create_embedded_files() -> HashMap<String, String> {
    let mut files = HashMap::new();

    files.insert("Cargo.toml".to_string(), cargo_toml_content());
    files.insert("src/main.rs".to_string(), main_rs_content());
    files.insert("memory.x".to_string(), memory_x_content());
    files.insert(".cargo/config.toml".to_string(), cargo_config_content());
    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        config_toml_content(),
    );

    files
}

/// Cargo.toml content for embedded project
fn cargo_toml_content() -> String {
    r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"
authors = ["{{author}}"]
description = "A Ferrous Forge compliant embedded Rust project"
license = "MIT OR Apache-2.0"
repository = "https://github.com/{{author}}/{{project_name}}"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
nb = "1.1"

[dependencies.cortex-m-semihosting]
version = "0.5"
optional = true

[[bin]]
name = "{{project_name}}"
test = false
bench = false

[features]
default = []
semihosting = ["cortex-m-semihosting"]

[profile.dev]
debug = true
lto = false
codegen-units = 16

[profile.release]
debug = true
lto = true
codegen-units = 1
opt-level = "s"
"#
    .to_string()
}

/// src/main.rs content for embedded project
fn main_rs_content() -> String {
    r#"//! {{project_name}} - A Ferrous Forge compliant embedded project
#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use cortex_m_rt::entry;
use panic_halt as _;

#[cfg(feature = "semihosting")]
use cortex_m_semihosting::{debug, hprintln};

/// Main entry point
#[entry]
fn main() -> ! {
    // Initialize hardware here
    init_hardware();

    #[cfg(feature = "semihosting")]
    let _ = hprintln!("{{project_name}} starting...");

    // Main application loop
    loop {
        // Your application logic here
        run_application();

        #[cfg(feature = "semihosting")]
        let _ = hprintln!("Application loop iteration");
    }
}

/// Initialize hardware peripherals
fn init_hardware() {
    // Initialize clocks, GPIO, and other peripherals
    // Add your hardware initialization code here
}

/// Run main application logic
fn run_application() {
    // Add your main application logic here
    cortex_m::asm::wfi(); // Wait for interrupt
}

/// Exit function for semihosting
#[cfg(feature = "semihosting")]
#[allow(unreachable_code)]
fn exit() -> ! {
    debug::exit(debug::EXIT_SUCCESS);
    panic!("Exit failed");
}
"#
    .to_string()
}

/// memory.x content for linker script
fn memory_x_content() -> String {
    r#"/* Linker script for the target microcontroller */
/* Adjust these values based on your specific target */
MEMORY
{
  /* NOTE: Adjust these memory regions based on your target */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}
"#
    .to_string()
}

/// .cargo/config.toml content
fn cargo_config_content() -> String {
    r#"[target.thumbv7m-none-eabi]
# Cortex-M3 and compatible
runner = "probe-rs run --chip {{target_chip}}"

[target.thumbv7em-none-eabi]
# Cortex-M4 without FPU
runner = "probe-rs run --chip {{target_chip}}"

[target.thumbv7em-none-eabihf]
# Cortex-M4F with FPU
runner = "probe-rs run --chip {{target_chip}}"

[build]
# Set default target based on your microcontroller
target = "thumbv7m-none-eabi"

[env]
DEFMT_LOG = "debug"
"#
    .to_string()
}

/// .ferrous-forge/config.toml content
fn config_toml_content() -> String {
    r#"# Ferrous Forge configuration for embedded projects

[validation]
max_line_length = 100
max_file_length = 300
max_function_length = 50
allow_unwrap = false
allow_expect = false

[embedded]
# Embedded-specific configuration
allow_panic_handler = true
allow_no_std = true
allow_no_main = true
"#
    .to_string()
}
