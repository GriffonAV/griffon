mod accounts;
mod policy;
mod runtime;
mod system;

use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::AuditState;

const MAX_TEXT_FILE_BYTES: u64 = 64 * 1024;

pub(super) fn run_all(state: &mut AuditState) {
    system::run(state);
    accounts::run(state);
    policy::run(state);
    runtime::run(state);
}

pub(super) fn read_limited(path: &Path, max_bytes: u64) -> io::Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file is not regular or exceeds {max_bytes} bytes"),
        ));
    }
    let mut bytes = Vec::with_capacity(initial_read_capacity(metadata.len(), max_bytes));
    fs::File::open(path)?
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file exceeds {max_bytes} bytes"),
        ));
    }
    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn initial_read_capacity(metadata_len: u64, max_bytes: u64) -> usize {
    const PROCFS_READ_CAPACITY: u64 = 8 * 1024;

    if metadata_len == 0 {
        max_bytes.min(PROCFS_READ_CAPACITY) as usize
    } else {
        metadata_len.min(max_bytes) as usize
    }
}

fn read_small(path: &Path) -> io::Result<String> {
    read_limited(path, MAX_TEXT_FILE_BYTES)
}

fn mode_string(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    format!("{:04o}", metadata.mode() & 0o7777)
}

#[cfg(test)]
mod tests {
    use super::initial_read_capacity;

    #[test]
    fn procfs_files_receive_a_bounded_initial_capacity() {
        assert_eq!(initial_read_capacity(0, 32 * 1024), 8 * 1024);
        assert_eq!(initial_read_capacity(0, 1024), 1024);
        assert_eq!(initial_read_capacity(512, 32 * 1024), 512);
    }
}
