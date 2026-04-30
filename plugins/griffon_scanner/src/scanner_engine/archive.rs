#[derive(PartialEq)]
pub enum ArchiveKind {
    Zip,
    // TarGz,
    Tar,
    // GzipSingle,
    Unknown,
}

pub fn detect_archive(bytes: &[u8]) -> ArchiveKind {
    match infer::get(bytes).map(|t| t.mime_type()) {
        Some("application/zip") => ArchiveKind::Zip,
        Some("application/x-tar") => ArchiveKind::Tar,
        // Some("application/gzip") => {
        //     if is_tar_gz(bytes) {
        //         ArchiveKind::TarGz
        //     } else {
        //         ArchiveKind::GzipSingle
        //     }
        // }
        _ => ArchiveKind::Unknown,
    }
}

// fn is_tar_gz(bytes: &[u8]) -> bool {
//     use flate2::read::GzDecoder;
//     use std::io::Read;
//     let mut gz = GzDecoder::new(bytes);
//     let mut buf = [0u8; 265];
//     gz.read_exact(&mut buf)
//         .map(|_| infer::get(&buf).map(|t| t.mime_type()) == Some("application/x-tar"))
//         .unwrap_or(false)
// }
