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
    /// FIXME: delete?
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

    /// Get the sample name for a given position.
    pub fn get_sample_name(&self, pos: &str) -> Option<String> {
        self.names.get(pos).cloned()
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

// Implement Iterator trait for owned value of Samples.
//
// It returns a tuple of (position, sample_name) for each iteration, until depleted.
// Position come from SMP_POSITIONS, and sample_name from the BTreeMap.
impl IntoIterator for Samples {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut items = Vec::new();
        for &pos in SMP_POSITIONS.iter() {
            if let Some(name) = self.names.get(pos) {
                items.push((pos.into(), name.clone()));
            }
        }
        items.into_iter()
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
