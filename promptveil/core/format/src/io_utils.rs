use std::io::{self, Read, Write, Seek, SeekFrom};
use bincode::{serialize_into, deserialize_from};
use crate::{PVeilFile, PVeilHeader, Result, Error, BlockIndex};

pub struct FileWriter<W: Write + Seek> {
    writer: W,
    current_offset: u64,
}

impl<W: Write + Seek> FileWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            current_offset: 0,
        }
    }

    pub fn write_file(&mut self, file: &PVeilFile) -> Result<()> {
        // Write header
        let header_size = self.write_header(&file.header)?;
        self.current_offset += header_size;

        // Write schema
        let schema_offset = self.current_offset;
        serialize_into(&mut self.writer, &file.schema)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        self.current_offset += bincode::serialized_size(&file.schema)
            .map_err(|e| Error::Serialization(e.to_string()))? as u64;

        // Update header with schema offset
        self.writer.seek(SeekFrom::Start(0))?;
        let mut header = file.header.clone();
        header.schema_offset = schema_offset;
        serialize_into(&mut self.writer, &header)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Write partitions and update block index
        self.writer.seek(SeekFrom::Start(self.current_offset))?;
        let mut block_index = BlockIndex::new();
        
        for partition in &file.partitions {
            let partition_start = self.writer.seek(SeekFrom::Current(0))?;
            serialize_into(&mut self.writer, partition)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            
            // Update block index with correct offsets
            self.current_offset = partition.update_block_index(&mut block_index, partition_start);
            self.writer.seek(SeekFrom::Start(self.current_offset))?;
        }

        // Write block index at the end
        serialize_into(&mut self.writer, &block_index)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(())
    }

    fn write_header(&mut self, header: &PVeilHeader) -> Result<u64> {
        let start = self.writer.seek(SeekFrom::Current(0))?;
        serialize_into(&mut self.writer, header)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let end = self.writer.seek(SeekFrom::Current(0))?;
        Ok(end - start)
    }
}

pub struct FileReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> FileReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_file(&mut self) -> Result<PVeilFile> {
        // Read and validate header
        let header: PVeilHeader = deserialize_from(&mut self.reader)
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        if &header.magic != b"PVEIL" {
            return Err(Error::InvalidMagic);
        }

        // Read schema
        self.reader.seek(SeekFrom::Start(header.schema_offset))?;
        let schema = deserialize_from(&mut self.reader)
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        // Read partitions
        let mut partitions = Vec::with_capacity(header.partition_count as usize);
        for _ in 0..header.partition_count {
            let partition = deserialize_from(&mut self.reader)
                .map_err(|e| Error::Deserialization(e.to_string()))?;
            partitions.push(partition);
        }

        // Read block index
        let block_index = deserialize_from(&mut self.reader)
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        Ok(PVeilFile {
            header,
            schema,
            partitions,
            block_index,
        })
    }
}

#[derive(Debug)]
pub struct Header {
    magic: [u8; 5],
    version: u32,
}

impl Header {
    pub fn new() -> Self {
        Self {
            magic: *b"PVEIL",
            version: 1,
        }
    }
} 