use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::Path;

pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_file(path: &Path, data: &[u8]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_io() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        let data = b"Hello, World!";
        write_file(&file_path, data).unwrap();
        
        let read_data = read_file(&file_path).unwrap();
        assert_eq!(data.to_vec(), read_data);
    }
} 