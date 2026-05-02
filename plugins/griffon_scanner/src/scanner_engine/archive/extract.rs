use crate::scanner_engine::archive::ArchiveKind;

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
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(bytes);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(_) => return vec![],
    };

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i)
            && file.is_file()
        {
            let mut buf = Vec::with_capacity(file.size() as usize);
            let mut f = file;
            std::io::copy(&mut f, &mut buf).ok();
            log::info!("Extracted {} from ZIP archive", f.name());
            entries.push(ArchiveEntry {
                name: f.name().to_string(),
                bytes: buf,
            });
        }
    }
    entries
}
