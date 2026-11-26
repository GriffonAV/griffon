use crate::scanner_engine::archive::ArchiveKind;
use std::io::Cursor;
use zip::ZipArchive;

const MAX_TOTAL_SIZE: usize = 500 * 1024 * 1024;
const MAX_COMPRESSION_RATIO: f64 = 100.0;

pub struct ArchiveEntry {
    pub name: String, // path inside the archive
    pub bytes: Vec<u8>,
}

pub fn extract_entries(bytes: &[u8], kind: &ArchiveKind) -> Vec<ArchiveEntry> {
    match kind {
        ArchiveKind::Zip => extract_zip(bytes),
        // ArchiveKind::TarGz => extract_tar(flate2_decode(bytes)),
        // ArchiveKind::Tar => extract_tar(bytes.to_vec()),
        // ArchiveKind::GzipSingle => extract_gz_single(bytes),
        ArchiveKind::Unknown => vec![],
        _ => {
            log::warn!("Unsupported archive type for extraction");
            vec![]
        }
    }
}

fn extract_zip(bytes: &[u8]) -> Vec<ArchiveEntry> {
    let cursor = Cursor::new(bytes);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(_) => return vec![],
    };
    let mut total_size = 0usize;
    let mut entries = Vec::new();

    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i)
            && file.is_file()
        {
            // Used only as a label, never as a path. An undecodable name must
            // not cause the entry to be skipped, so fall back to a lossy form.
            let name = match file.name() {
                Ok(n) => n.into_owned(),
                Err(_) => String::from_utf8_lossy(file.name_raw()).into_owned(),
            };
            let ratio = file.size() as f64 / file.compressed_size() as f64;
            if ratio > MAX_COMPRESSION_RATIO {
                log::warn!(
                    "Skipping {} in ZIP archive due to high compression ratio ({:.2})",
                    name,
                    ratio
                );
                continue;
            }

            total_size += file.size() as usize;
            if total_size > MAX_TOTAL_SIZE {
                log::warn!(
                    "Skipping {} in ZIP archive because total extracted size exceeds limit",
                    name
                );
                break;
            }

            let mut buf = Vec::with_capacity(file.size() as usize);
            let mut f = file;

            std::io::copy(&mut f, &mut buf).ok();
            log::info!("Extracted {} from ZIP archive", name);
            entries.push(ArchiveEntry { name, bytes: buf });
        }
    }
    entries
}
