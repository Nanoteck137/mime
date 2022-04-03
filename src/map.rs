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
    ConvertToU64Failed,
    FileCreationFailed(std::io::Error),
    FileWriteFailed(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

type Index = u32;

#[derive(Copy, Clone, Debug)]
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
}

pub struct Sector {
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<Index>,
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
}

pub struct Map {
    sectors: Vec<Sector>,
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
            sector.serialize(buffer)?;
        }

        Ok(())
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
        file.write_all(&buffer[..]).map_err(|e| Error::FileWriteFailed(e))?;

        Ok(())
    }
}
