//! Module to handle sample names and locations.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SMP_POSITIONS: [&str; 8] = ["A1", "A2", "B1", "B2", "C1", "C2", "D1", "D2"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Samples {
    names: BTreeMap<String, String>,
}

impl Samples {
    /// Create a new Samples instance with empty names for each position.
    pub fn new() -> Self {
        let mut names = BTreeMap::new();
        for &pos in SMP_POSITIONS.iter() {
            names.insert(pos.into(), String::new());
        }
        Self { names }
    }

    /// Get the name of the sample at the given position.
    pub fn get_name(&self, position: &str) -> Result<String> {
        match self.names.get(position) {
            Some(entry) => Ok(entry.clone()),
            None => bail!("Could not find sample name at position {}.", position),
        }
    }

    /// Set the name of the sample at the given position.
    pub fn set_name(&mut self, position: &str, name: &str) -> Result<()> {
        match self.names.get_mut(position) {
            Some(entry) => *entry = name.into(),
            None => bail!("Could not find sample name at position {}.", position),
        }
        Ok(())
    }

    /// Get a clone of the entire names HashMap.
    pub fn get_names(&self) -> BTreeMap<String, String> {
        self.names.clone()
    }
}

impl Iterator for Samples {
    type Item = (&'static str, String);

    fn next(&mut self) -> Option<Self::Item> {
        // This is a simple iterator that goes through the sample positions
        // and returns their names. It will return None when all positions
        // have been iterated over.
        for &pos in SMP_POSITIONS.iter() {
            if let Some(name) = self.names.get(pos) {
                return Some((pos, name.clone()));
            }
        }
        None
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let smp = Samples::new();
        assert_eq!(smp.names.len(), 8);
    }

    #[test]
    fn test_name() {
        let mut smp = Samples::new();
        smp.set_name("A1", "Sample A1").unwrap();
        assert_eq!(smp.get_name("A1").unwrap(), "Sample A1");

        assert!(smp.set_name("Invalid name", "still not valid").is_err());
        assert!(smp.get_name("Invalid name").is_err());
    }
}
