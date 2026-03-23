use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use log::debug;

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    Executable(ExecutableType),
    Script(ScriptType),
    Archive(ArchiveType),
    Document(DocumentType),
    GenericBinary, // error with magic byte / or non classified file -> run generic signature rule only.
    Unknown,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanStage {
    Pre,
    Post,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutableType {
    Elf,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptType {
    Shell,
    Python,
    Perl,
    Php,
    Other,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveType {
    Zip,
    Tar,
    Gzip,
    SevenZip,
    Unknown,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentType {
    Pdf,
    Other,
}

pub struct FileContext {
    path: PathBuf,
    detected_type: FileType,
    extension: Option<String>,
    size: u64,
}

struct MagicSig {
    offset: usize,
    bytes: &'static [u8],
    file_type: FileType,
}

static MAGIC_SIG: &[MagicSig] = &[
    MagicSig {
        offset: 0,
        bytes: b"\x7FELF",
        file_type: FileType::Executable(ExecutableType::Elf),
    },
    MagicSig {
        offset: 0,
        bytes: b"PK\x03\x04",
        file_type: FileType::Archive(ArchiveType::Zip),
    },
];

pub fn get(path: &Path) -> io::Result<FileContext> {
    debug!("Finding file class for: {}", path.display());
    if !path.exists() || !path.is_file() {
        return Ok(FileContext {
            path: path.to_path_buf(),
            detected_type: FileType::Unknown,
            extension: None,
            size: 0,
        });
    }

    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str().map(|s| s.to_lowercase()));
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 16];
    let n = file.read(&mut buffer)?;
    if let Some(detected_type) = get_type(&buffer[..n]) {
        return Ok(FileContext {
            path: path.to_path_buf(),
            detected_type,
            extension,
            size,
        });
    }

    Ok(FileContext {
        path: path.to_path_buf(),
        detected_type: FileType::Unknown,
        extension: None,
        size: size,
    })
}

fn get_type(buf: &[u8]) -> Option<FileType> {
    for sig in MAGIC_SIG {
        if buf.len() >= sig.offset + sig.bytes.len()
            && &buf[sig.offset..sig.offset + sig.bytes.len()] == sig.bytes
        {
            return Some(sig.file_type);
        }
    }
    None
}
