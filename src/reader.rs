use anyhow::Result;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

pub enum LogReader {
    PlainFile(BufReader<File>),
    GzipFile(BufReader<GzDecoder<File>>),
    Stdin(BufReader<std::io::Stdin>),
}

impl LogReader {
    pub fn new(path: Option<&str>) -> Result<Self> {
        match path {
            None => Ok(LogReader::Stdin(BufReader::new(stdin()))),
            Some(p) => {
                let file = File::open(p)?;
                if p.ends_with(".gz") {
                    let decoder = GzDecoder::new(file);
                    Ok(LogReader::GzipFile(BufReader::new(decoder)))
                } else {
                    Ok(LogReader::PlainFile(BufReader::new(file)))
                }
            }
        }
    }

    pub fn lines(&mut self) -> Box<dyn Iterator<Item = Result<String>> + '_> {
        match self {
            LogReader::PlainFile(reader) => Box::new(reader.lines().map(|r| r.map_err(Into::into))),
            LogReader::GzipFile(reader) => Box::new(reader.lines().map(|r| r.map_err(Into::into))),
            LogReader::Stdin(reader) => Box::new(reader.lines().map(|r| r.map_err(Into::into))),
        }
    }
}

/// Create readers for multiple files or stdin
pub fn create_readers(files: &[String]) -> Result<Vec<(Option<String>, LogReader)>> {
    if files.is_empty() {
        Ok(vec![(None, LogReader::new(None)?)])
    } else {
        files
            .iter()
            .map(|f| Ok((Some(f.clone()), LogReader::new(Some(f))?)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_reader_plain_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();
        temp_file.flush().unwrap();

        let mut reader = LogReader::new(Some(temp_file.path().to_str().unwrap())).unwrap();
        let lines: Vec<_> = reader.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].as_ref().unwrap(), "Line 1");
        assert_eq!(lines[1].as_ref().unwrap(), "Line 2");
        assert_eq!(lines[2].as_ref().unwrap(), "Line 3");
    }

    #[test]
    fn test_reader_gzip_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().with_extension("gz");

        let file = File::create(&temp_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        writeln!(encoder, "Compressed Line 1").unwrap();
        writeln!(encoder, "Compressed Line 2").unwrap();
        encoder.finish().unwrap();

        let mut reader = LogReader::new(Some(temp_path.to_str().unwrap())).unwrap();
        let lines: Vec<_> = reader.lines().collect();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].as_ref().unwrap(), "Compressed Line 1");
        assert_eq!(lines[1].as_ref().unwrap(), "Compressed Line 2");

        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_reader_nonexistent_file() {
        let result = LogReader::new(Some("/nonexistent/file.log"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_readers_empty() {
        let readers = create_readers(&[]);
        assert!(readers.is_ok());
        let readers = readers.unwrap();
        assert_eq!(readers.len(), 1);
        assert!(readers[0].0.is_none()); // Should be stdin
    }

    #[test]
    fn test_create_readers_multiple_files() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();
        writeln!(temp_file1, "File 1").unwrap();
        writeln!(temp_file2, "File 2").unwrap();
        temp_file1.flush().unwrap();
        temp_file2.flush().unwrap();

        let files = vec![
            temp_file1.path().to_str().unwrap().to_string(),
            temp_file2.path().to_str().unwrap().to_string(),
        ];

        let readers = create_readers(&files).unwrap();
        assert_eq!(readers.len(), 2);
        assert_eq!(readers[0].0.as_ref().unwrap(), files[0].as_str());
        assert_eq!(readers[1].0.as_ref().unwrap(), files[1].as_str());
    }

    #[test]
    fn test_create_readers_with_invalid_file() {
        let files = vec!["/nonexistent/file.log".to_string()];
        let result = create_readers(&files);
        assert!(result.is_err());
    }
}
