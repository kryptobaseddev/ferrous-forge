//! Template System 2.0 - Project templates with Ferrous Forge standards built-in
//!
//! This module provides a complete template system for creating new Rust projects
//! that are pre-configured to comply with Ferrous Forge standards.

pub mod builtin;
pub mod engine;
pub mod manifest;
pub mod registry;

pub use engine::{TemplateEngine, TemplateVariable};
pub use manifest::{TemplateFile, TemplateKind, TemplateManifest};
pub use registry::BuiltinTemplate;
pub use registry::TemplateRegistry;
