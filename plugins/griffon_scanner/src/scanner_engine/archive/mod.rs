pub mod extract;

#[derive(PartialEq, Debug)]
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

#[cfg(test)]
mod tests {
    use super::extract::extract_entries;
    use super::{ArchiveKind, detect_archive};
    use std::io::Write;
    use zip::CompressionMethod;
    use zip::write::{SimpleFileOptions, ZipWriter};

    fn build_zip(files: &[(&str, &[u8], CompressionMethod)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, content, method) in files {
            writer
                .start_file(
                    *name,
                    SimpleFileOptions::default().compression_method(*method),
                )
                .unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn detects_zip() {
        let zip = build_zip(&[("a.txt", b"hello", CompressionMethod::Stored)]);
        assert_eq!(detect_archive(&zip), ArchiveKind::Zip);
    }

    #[test]
    fn plain_bytes_are_not_archives() {
        assert_eq!(detect_archive(b"just some text"), ArchiveKind::Unknown);
        assert_eq!(detect_archive(&[]), ArchiveKind::Unknown);
    }

    #[test]
    fn extracts_every_file_with_its_name() {
        let zip = build_zip(&[
            ("a.txt", b"first", CompressionMethod::Stored),
            ("dir/b.txt", b"second", CompressionMethod::Deflated),
        ]);
        let entries = extract_entries(&zip, &ArchiveKind::Zip);

        let got: Vec<_> = entries
            .iter()
            .map(|e| (e.name.as_str(), e.bytes.as_slice()))
            .collect();
        assert_eq!(
            got,
            [("a.txt", &b"first"[..]), ("dir/b.txt", &b"second"[..])]
        );
    }

    #[test]
    fn skips_directory_entries() {
        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .add_directory("empty_dir/", SimpleFileOptions::default())
            .unwrap();
        let zip = writer.finish().unwrap().into_inner();

        assert!(extract_entries(&zip, &ArchiveKind::Zip).is_empty());
    }

    #[test]
    fn skips_entries_with_suspicious_compression_ratio() {
        // 1 MiB of zeros deflates to ~1 KiB: ratio far above the 100x limit.
        let bomb = vec![0u8; 1024 * 1024];
        let zip = build_zip(&[
            ("bomb.bin", &bomb, CompressionMethod::Deflated),
            ("ok.txt", b"normal", CompressionMethod::Stored),
        ]);
        let entries = extract_entries(&zip, &ArchiveKind::Zip);

        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["ok.txt"]);
    }

    #[test]
    fn corrupt_or_unsupported_input_yields_nothing() {
        assert!(extract_entries(b"PK\x03\x04 truncated", &ArchiveKind::Zip).is_empty());
        assert!(extract_entries(b"whatever", &ArchiveKind::Unknown).is_empty());
        assert!(extract_entries(b"whatever", &ArchiveKind::Tar).is_empty());
    }
}
