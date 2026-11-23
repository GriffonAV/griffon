pub mod list_file;
pub mod manifest;
pub mod restore_file;

use std::path::{Path, PathBuf};

pub mod quarantine_file;

const QUARANTINED_SUFFIX: &str = ".quarantined";

fn generated_quarantine_stem(name: &str) -> Option<&str> {
    let stem = name.strip_suffix(QUARANTINED_SUFFIX)?;
    let mut parts = stem.split('_');
    let (Some(date), Some(time), Some(hash), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };

    (date.len() == 8
        && date.bytes().all(|byte| byte.is_ascii_digit())
        && time.len() == 6
        && time.bytes().all(|byte| byte.is_ascii_digit())
        && hash.len() == 16
        && hash
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')))
    .then_some(stem)
}

pub struct Quarantine {
    pub dir: PathBuf,
}

impl Quarantine {
    pub fn new(dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        log::info!("Quarantine directory: {}", dir.display());
        Ok(Self {
            dir: dir.to_path_buf(),
        })
    }

    pub fn default_dir() -> PathBuf {
        if cfg!(debug_assertions) {
            PathBuf::from("quarantine")
        } else {
            // is equal to ~/.local/share/griffon_scanner/quarantine
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("griffon_scanner")
                .join("quarantine")
        }
    }

    fn paths_for_quarantined_name(
        &self,
        quarantined_name: &str,
    ) -> Result<(PathBuf, PathBuf), String> {
        let name_path = Path::new(quarantined_name);
        let Some(stem) = generated_quarantine_stem(quarantined_name) else {
            return Err("Invalid quarantined file name; expected a name returned by q_list".into());
        };

        if name_path.file_name().and_then(|name| name.to_str()) != Some(quarantined_name) {
            return Err("Invalid quarantined file name; expected a name returned by q_list".into());
        }
        Ok((
            self.dir.join(quarantined_name),
            self.dir.join(format!("{stem}.json")),
        ))
    }
}
