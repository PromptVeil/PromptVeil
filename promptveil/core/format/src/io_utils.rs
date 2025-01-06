use std::io::{Read, Write, Seek, SeekFrom};

pub struct FileWriter<W: Write + Seek> {
    writer: W,
    current_pos: u64,
}

impl<W: Write + Seek> FileWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            current_pos: 0,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.writer.write(data)?;
        self.current_pos += bytes_written as u64;
        Ok(bytes_written)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.current_pos = self.writer.seek(pos)?;
        Ok(self.current_pos)
    }

    pub fn position(&self) -> u64 {
        self.current_pos
    }
}

pub struct FileReader<R: Read + Seek> {
    reader: R,
    current_pos: u64,
}

impl<R: Read + Seek> FileReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            current_pos: 0,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        self.current_pos += bytes_read as u64;
        Ok(bytes_read)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.current_pos = self.reader.seek(pos)?;
        Ok(self.current_pos)
    }

    pub fn position(&self) -> u64 {
        self.current_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_file_writer() {
        let mut buffer = Cursor::new(Vec::new());
        let mut writer = FileWriter::new(&mut buffer);

        assert_eq!(writer.write(b"Hello").unwrap(), 5);
        assert_eq!(writer.position(), 5);

        writer.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(writer.position(), 0);

        assert_eq!(writer.write(b"World").unwrap(), 5);
        assert_eq!(writer.position(), 5);

        let data = buffer.into_inner();
        assert_eq!(&data, b"World");
    }

    #[test]
    fn test_file_reader() {
        let data = b"Hello, World!";
        let mut reader = FileReader::new(Cursor::new(data));

        let mut buf = [0u8; 5];
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"Hello");
        assert_eq!(reader.position(), 5);

        reader.seek(SeekFrom::Start(7)).unwrap();
        assert_eq!(reader.position(), 7);

        let mut buf = [0u8; 5];
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"World");
    }
} 