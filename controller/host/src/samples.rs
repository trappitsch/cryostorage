//! Module to handle sample names and locations.
use anyhow::{Result, bail};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SMP_POSITIONS: [&str; 8] = ["A1", "A2", "B1", "B2", "C1", "C2", "D1", "D2"];

/// Contain all the information about a sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// The name of the sample.
    name: String,
    /// Timestamp when the sample was first added to the list. None if empty sample.
    timestamp_added: Option<DateTime<Local>>,
}

impl Sample {
    /// Create an empty sample and return it.
    pub fn empty() -> Sample {
        Sample {
            name: "".into(),
            timestamp_added: None,
        }
    }

    /// Create a new sample from a given name with current timestamp.
    pub fn new(name: &str) -> Sample {
        Sample {
            name: name.into(),
            timestamp_added: Some(Local::now()),
        }
    }

    /// Get the date as a fomratted string or an empty string if no date.
    pub fn get_date(&self) -> String {
        if let Some(dt) = self.timestamp_added {
            dt.format("%Y-%m-%d").to_string()
        } else {
            String::new()
        }
    }

    /// Get only the sample name
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

/// Transform &str into a Sample.
///
/// If the &str is empty, return an empty sample w/o a timestamp.
impl From<&str> for Sample {
    fn from(value: &str) -> Self {
        if value.trim().is_empty() {
            Self::empty()
        } else {
            Self::new(value)
        }
    }
}

/// Hold all samples and their loaded positions.
///
/// This stores a BTreeMap where the key is the position (unique) and the value the sample itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Samples {
    names: BTreeMap<String, Sample>,
}

impl Samples {
    /// Create a new Samples instance with empty names for each position.
    pub fn new() -> Self {
        let mut names = BTreeMap::new();
        for &pos in SMP_POSITIONS.iter() {
            names.insert(pos.into(), Sample::empty());
        }
        Self { names }
    }

    /// Update the sample name at the given position and return the index of the entry.
    pub fn update_sample(&mut self, pos: &str, value: &str) -> Result<(usize, Sample)> {
        if self.names.contains_key(pos) {
            let smp: Sample = value.into();

            self.names.insert(pos.into(), smp.clone());

            Ok((
                SMP_POSITIONS
                    .iter()
                    .position(|&p| p == pos)
                    .expect("Cannot fail"),
                smp,
            ))
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
    type Item = (String, Sample);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut items = Vec::new();
        for &pos in SMP_POSITIONS.iter() {
            if let Some(sample) = self.names.get(pos) {
                items.push((pos.into(), sample.clone()));
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
