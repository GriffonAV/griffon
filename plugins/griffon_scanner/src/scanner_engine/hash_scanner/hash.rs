use std::{fs, io, path::Path};

use sha2::Digest;

use crate::scanner_engine::{
    data_type::{Severity, Threat},
    hash_scanner::SignatureDb,
};

pub fn hash_file(path: &Path) -> io::Result<String> {
    //! is there a way to prevent multiple allocation of let bytes
    let bytes = fs::read(path)?;
    let digest = sha2::Sha256::digest(&bytes);
    Ok(hex::encode(digest))
}

#[allow(dead_code)]
pub fn scan_bytes(path: &Path, input: &[u8], db: &SignatureDb) -> Result<Vec<Threat>, String> {
    let digest = sha2::Sha256::digest(input);
    let hash = hex::encode(digest);

    if db.contains(&hash) {
        Ok(vec![Threat {
            path: path.to_path_buf(),
            name: format!("known-malware:{}", &hash[..16]),
            severity: Severity::High,
            matched_rule: "hash-db".to_string(),
        }])
    } else {
        Ok(Vec::new())
    }
}

#[allow(dead_code)]
pub fn scan_file(path: &Path, db: &SignatureDb) -> Result<Vec<Threat>, String> {
    let file_hash = hash_file(path);

    match file_hash {
        Ok(hash) => {
            if db.contains(&hash) {
                Ok(vec![Threat {
                    path: path.to_path_buf(),
                    name: format!("known-malware:{}", &hash[..16]),
                    severity: Severity::High,
                    matched_rule: "hash-db".to_string(),
                }])
            } else {
                Ok(Vec::new())
            }
        }
        Err(e) => Err(format!("Failed to hash file: {}", e)),
    }
}
