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

    /// Get a vector of tuples for the btreemap.
    pub fn get_for_slint(&self) -> [(slint::SharedString, slint::SharedString); 8] {
        let mut model: [(slint::SharedString, slint::SharedString); 8] = Default::default();
        println!("BTreeMap contents: {:?}", self.names);
        for (it, (key, value)) in self.names.iter().enumerate() {
            println!("Key: {}, Value: {}", key, value);
            model[it].1 = key.into();
            model[it].0 = value.into();
        }
        model
    }

    /// Update the sample name at the given position and return the index of the entry.
    pub fn update_sample(&mut self, pos: &str, value: &str) -> Result<usize> {
        if self.names.contains_key(pos) {
            self.names.insert(pos.into(), value.into());
            Ok(SMP_POSITIONS
                .iter()
                .position(|&p| p == pos)
                .expect("Cannot fail"))
        } else {
            bail!("Position {} does not exist", pos);
        }
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
}
