//! Custom map format

// NOTE(patrik):
// Map Format:
//      Header: 8 bytes (Magic: MIME Version: 4bytes)
//      Sectors:
//          - Vertex Buffer
//          - Index Buffer
//          - Collision Boxes

use std::path::Path;
use std::fs::File;
use std::io::Write;

pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug)]
pub enum Error {
    ArrayConvertionFailed,
    ConvertToU64Failed,
    ConvertToUsizeFailed,
    FileCreationFailed(std::io::Error),
    FileWriteFailed(std::io::Error),
    IncorrectMagic,
    IncorrectVersion,

    BufferToSmallVertex,
    BufferToSmallSector,
    BufferToSmallMap,
}

pub type Result<T> = std::result::Result<T, Error>;

type Index = u32;

// The size of the mime header
// 4 byte magic + version
const HEADER_SIZE: usize = 4 + std::mem::size_of::<u32>();

// The header magic
const HEADER_MAGIC: &'static [u8] = b"MIME";

// The size of a single vertex (x, y, z, r, g, b, a)
const VERTEX_SIZE: usize = 7 * std::mem::size_of::<f32>();

// The size of a single index
const INDEX_SIZE: usize = std::mem::size_of::<u32>();

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, color: [f32; 4]) -> Self {
        Self {
            x, y, z, color
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<()> {
        // Vertex Position (x, y)
        buffer.extend_from_slice(&self.x.to_le_bytes());
        buffer.extend_from_slice(&self.y.to_le_bytes());
        buffer.extend_from_slice(&self.z.to_le_bytes());

        // Vertex Color (r, g, b, a)
        buffer.extend_from_slice(&self.color[0].to_le_bytes());
        buffer.extend_from_slice(&self.color[1].to_le_bytes());
        buffer.extend_from_slice(&self.color[2].to_le_bytes());
        buffer.extend_from_slice(&self.color[3].to_le_bytes());

        Ok(())
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < VERTEX_SIZE {
            return Err(Error::BufferToSmallVertex);
        }

        let x = f32::from_le_bytes(
            buffer[0..4].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let y = f32::from_le_bytes(
            buffer[4..8].try_into()
            .map_err(|_| Error::ArrayConvertionFailed)?);
        let z = f32::from_le_bytes(
            buffer[8..12].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);

        let r = f32::from_le_bytes(
            buffer[12..16].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let g = f32::from_le_bytes(
            buffer[16..20].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let b = f32::from_le_bytes(
            buffer[20..24].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let a = f32::from_le_bytes(
            buffer[24..28].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);

        Ok(Vertex::new(x, y, z, [r, g, b, a]))
    }
}

pub struct Sector {
    pub vertex_buffer: Vec<Vertex>,
    pub index_buffer: Vec<Index>,
}

impl Sector {
    pub fn new(vertex_buffer: Vec<Vertex>, index_buffer: Vec<Index>)
        -> Sector
    {
        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<()> {
        // Vertex buffer count
        let count: u64 =
            self.vertex_buffer.len().try_into()
                .map_err(|_| Error::ConvertToU64Failed)?;
        buffer.extend_from_slice(&count.to_le_bytes());

        // Index buffer count
        let count: u64 =
            self.index_buffer.len().try_into()
                .map_err(|_| Error::ConvertToU64Failed)?;
        buffer.extend_from_slice(&count.to_le_bytes());

        // Serialize the vertex buffer
        for vertex in &self.vertex_buffer {
            vertex.serialize(buffer)?;
        }

        // Serialize the index buffer
        for index in &self.index_buffer {
            buffer.extend_from_slice(&index.to_le_bytes());
        }

        Ok(())
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < std::mem::size_of::<u64>() * 2 {
            return Err(Error::BufferToSmallSector);
        }

        let vertex_count = u64::from_le_bytes(
            buffer[0..8].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let vertex_count: usize =
            vertex_count.try_into().map_err(|_| Error::ConvertToUsizeFailed)?;

        let index_count = u64::from_le_bytes(
            buffer[8..16].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let index_count: usize =
            index_count.try_into().map_err(|_| Error::ConvertToUsizeFailed)?;

        let buffer = &buffer[16..];

        if buffer.len() < VERTEX_SIZE * vertex_count {
            return Err(Error::BufferToSmallSector);
        }

        let mut vertex_buffer = Vec::with_capacity(vertex_count);

        for i in 0..vertex_count {
            let start = i * VERTEX_SIZE;
            let buffer = &buffer[start..start + VERTEX_SIZE];
            let vertex = Vertex::deserialize(&buffer)?;
            vertex_buffer.push(vertex);
        }

        let buffer = &buffer[(vertex_count * VERTEX_SIZE)..];

        let mut index_buffer = Vec::with_capacity(index_count);

        if buffer.len() < INDEX_SIZE * index_count {
            return Err(Error::BufferToSmallSector);
        }

        for i in 0..index_count {
            let start = i * INDEX_SIZE;
            let buffer = &buffer[start..start + INDEX_SIZE];
            let index = u32::from_le_bytes(
                buffer.try_into()
                    .map_err(|_| Error::ArrayConvertionFailed)?);
            index_buffer.push(index);
        }

        Ok(Self::new(vertex_buffer, index_buffer))
    }
}

pub struct Map {
    pub sectors: Vec<Sector>,
}

impl Map {
    pub fn new(sectors: Vec<Sector>) -> Self {
        Self {
            sectors
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<()> {
        // Magic
        buffer.extend_from_slice(b"MIME");
        // Version
        buffer.extend_from_slice(&CURRENT_VERSION.to_le_bytes());

        // Serialize the sector count
        let count: u64 =
            self.sectors.len().try_into()
                .map_err(|_| Error::ConvertToU64Failed)?;
        buffer.extend_from_slice(&count.to_le_bytes());

        // Serialize all the sectors
        for sector in &self.sectors {
            let mut sector_buffer = Vec::new();
            sector.serialize(&mut sector_buffer)?;

            let sector_size: u64 =
                sector_buffer.len().try_into()
                    .map_err(|_| Error::ConvertToU64Failed)?;
            buffer.extend_from_slice(&sector_size.to_le_bytes());
            buffer.extend_from_slice(&sector_buffer);
        }

        Ok(())
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < HEADER_SIZE {
            return Err(Error::BufferToSmallMap);
        }

        let magic = &buffer[0..4];
        if magic != HEADER_MAGIC {
            return Err(Error::IncorrectMagic);
        }

        let version = u32::from_le_bytes(
            buffer[4..8].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        if version != CURRENT_VERSION {
            return Err(Error::IncorrectVersion);
        }

        let buffer = &buffer[8..];

        if buffer.len() < std::mem::size_of::<u64>() {
            return Err(Error::BufferToSmallMap);
        }

        let sector_count = u64::from_le_bytes(
            buffer[0..8].try_into()
                .map_err(|_| Error::ArrayConvertionFailed)?);
        let sector_count: usize =
            sector_count.try_into().map_err(|_| Error::ConvertToUsizeFailed)?;

        let buffer = &buffer[8..];

        let mut sectors = Vec::with_capacity(sector_count);

        let mut offset = 0;

        for _i in 0..sector_count {
            let start = offset;
            let sector_size = u64::from_le_bytes(
                buffer[start..start + 8].try_into()
                    .map_err(|_| Error::ArrayConvertionFailed)?);
            let sector_size: usize =
                sector_size.try_into()
                    .map_err(|_| Error::ConvertToUsizeFailed)?;
            let start = start + 8;

            let sector =
                Sector::deserialize(&buffer[start..start + sector_size])?;
            sectors.push(sector);

            offset += sector_size + 8;
        }

        Ok(Self::new(sectors))
    }

    pub fn save_to_file<P>(&self, filename: P) -> Result<()>
        where P: AsRef<Path>
    {
        // Create the buffer holding the serialized data
        let mut buffer = Vec::new();

        // Serialize the map
        self.serialize(&mut buffer)?;

        // Write the buffer to a file
        let mut file = File::create(filename)
            .map_err(|e| Error::FileCreationFailed(e))?;
        file.write_all(&buffer[..])
            .map_err(|e| Error::FileWriteFailed(e))?;

        Ok(())
    }
}
