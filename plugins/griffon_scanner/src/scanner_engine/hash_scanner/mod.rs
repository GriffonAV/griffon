use std::{
    collections::HashSet,
    fs,
    io::{self, BufRead},
    path::Path,
};

pub mod hash;

pub struct SignatureDb {
    hashes: HashSet<String>,
}

impl SignatureDb {
    pub fn load(path: &Path) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        let hashes = reader
            .lines()
            .filter_map(|line| {
                let line = line.ok()?;
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    None
                } else {
                    Some(trimmed.to_lowercase())
                }
            })
            .collect::<HashSet<String>>();

        Ok(Self { hashes })
    }

    pub fn contains(&self, hash: &str) -> bool {
        self.hashes.contains(&hash.to_lowercase())
    }

    pub fn count(&self) -> usize {
        self.hashes.len()
    }
}
