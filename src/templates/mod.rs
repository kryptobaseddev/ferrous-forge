//! Template System 2.0 - Project templates with Ferrous Forge standards built-in
//!
//! This module provides a complete template system for creating new Rust projects
//! that are pre-configured to comply with Ferrous Forge standards.
//!
//! @task T021
//! @epic T014

/// Built-in project template definitions.
pub mod builtin;
/// Template rendering engine with variable substitution.
pub mod engine;
/// Template manifest and file specification types.
pub mod manifest;
/// Template discovery and registration.
pub mod registry;
/// Template repository management for community templates.
pub mod repository;
/// Template validation.
pub mod validation;

pub use engine::{TemplateEngine, TemplateVariable};
pub use manifest::{TemplateFile, TemplateKind, TemplateManifest};
pub use registry::BuiltinTemplate;
pub use registry::TemplateRegistry;
pub use repository::{CachedTemplate, TemplateIndex, TemplateRepository};
