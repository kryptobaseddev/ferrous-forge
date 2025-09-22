//! Template utility functions

use crate::{Error, Result};
use std::collections::HashMap;

/// Parse variable from key=value format
pub fn parse_var(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(Error::template(format!("Invalid variable format: '{}'. Use key=value", s)));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Collect template variables from command line arguments
pub fn collect_template_variables(variables: &[String]) -> Result<HashMap<String, String>> {
    let mut template_vars = HashMap::new();
    
    for var in variables {
        let (key, value) = parse_var(var)?;
        template_vars.insert(key, value);
    }
    
    Ok(template_vars)
}