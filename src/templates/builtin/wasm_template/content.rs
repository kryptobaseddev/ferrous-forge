//! WASM template file content generation

use super::templates::{index_html_content, index_js_content};
use std::collections::HashMap;

/// Create the files for WASM template
pub fn create_wasm_files() -> HashMap<String, String> {
    let mut files = HashMap::new();

    files.insert("Cargo.toml".to_string(), cargo_toml_content());
    files.insert("src/lib.rs".to_string(), lib_rs_content());
    files.insert("www/index.html".to_string(), index_html_content());
    files.insert("www/index.js".to_string(), index_js_content());
    files.insert("www/package.json".to_string(), package_json_content());
    files.insert(
        "www/webpack.config.js".to_string(),
        webpack_config_content(),
    );
    files.insert(
        ".ferrous-forge/config.toml".to_string(),
        config_toml_content(),
    );

    files
}

/// Cargo.toml content for WASM project
fn cargo_toml_content() -> String {
    r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"
authors = ["{{author}}"]
description = "{{description}}"
license = "MIT OR Apache-2.0"
repository = "https://github.com/{{author}}/{{project_name}}"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
console_error_panic_hook = { version = "0.1", optional = true }
wee_alloc = { version = "0.4", optional = true }

[dependencies.getrandom]
version = "0.2"
features = ["js"]

[features]
default = ["console_error_panic_hook"]

[profile.release]
opt-level = "s"
debug = false
lto = true
"#
    .to_string()
}

/// src/lib.rs content for WASM library - part 1
fn lib_rs_content() -> String {
    format!(
        "{}\n{}\n{}",
        lib_header(),
        lib_core_functions(),
        lib_calculator()
    )
}

/// Library header with imports and macros
fn lib_header() -> String {
    r#"//! {{project_name}} - WebAssembly library
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use wasm_bindgen::prelude::*;

// Import the `console.log` function from the browser's console
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}"#
    .to_string()
}

/// Core WASM functions
fn lib_core_functions() -> String {
    r#"/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    console_log!("{{project_name}} WASM module initialized!");
}

/// Greet function exposed to JavaScript
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Greetings from Rust and WebAssembly 🦀", name)
}

/// Add two numbers
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}"#
    .to_string()
}

/// Calculator implementation
fn lib_calculator() -> String {
    r#"/// Calculator struct demonstrating complex types
#[wasm_bindgen]
pub struct Calculator {
    value: f64,
}

#[wasm_bindgen]
impl Calculator {
    /// Create a new calculator
    #[wasm_bindgen(constructor)]
    pub fn new() -> Calculator {
        Calculator { value: 0.0 }
    }

    /// Add to the current value
    pub fn add(&mut self, value: f64) -> f64 {
        self.value += value;
        self.value
    }

    /// Subtract from the current value
    pub fn subtract(&mut self, value: f64) -> f64 {
        self.value -= value;
        self.value
    }

    /// Get the current value
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Reset the calculator
    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_calculator() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(10.0), 10.0);
        assert_eq!(calc.subtract(3.0), 7.0);
        calc.reset();
        assert_eq!(calc.value(), 0.0);
    }
}"#
    .to_string()
}

/// www/package.json content
fn package_json_content() -> String {
    r#"{
  "name": "{{project_name}}-www",
  "version": "0.1.0",
  "description": "Web frontend for {{project_name}} WASM module",
  "main": "index.js",
  "scripts": {
    "build": "webpack --mode=production",
    "start": "webpack serve --mode=development"
  },
  "devDependencies": {
    "webpack": "^5.88.0",
    "webpack-cli": "^5.1.0",
    "webpack-dev-server": "^4.15.0"
  }
}
"#
    .to_string()
}

/// www/webpack.config.js content
fn webpack_config_content() -> String {
    r#"const path = require('path');

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  devServer: {
    static: {
      directory: path.join(__dirname),
    },
    compress: true,
    port: 8080,
  },
  experiments: {
    asyncWebAssembly: true,
  }
};
"#
    .to_string()
}

/// .ferrous-forge/config.toml content
fn config_toml_content() -> String {
    r#"# Ferrous Forge configuration for WASM projects

[validation]
max_line_length = 100
max_file_length = 300
max_function_length = 50
allow_unwrap = false
allow_expect = false

[wasm]
# WASM-specific configuration
allow_js_interop = true
optimize_size = true
"#
    .to_string()
}
