//! Custom map format

// TODO(patrik): Should we do this?
use crate::*;

use std::path::Path;
use std::fs::File;
use std::io::Write;

// TODO(patrik): Make a better verison
/// The current version of the file format
pub const CURRENT_VERSION: u32 = 1;

type Index = u32;

/// The size of the mime header
const HEADER_SIZE: usize = 4 + std::mem::size_of::<u32>();

/// The header magic
const HEADER_MAGIC: &[u8] = b"MIME";

/// The size of a single vertex (x, y, z, r, g, b, a)
const VERTEX_SIZE: usize = 7 * std::mem::size_of::<f32>();

/// The size of a single index
const INDEX_SIZE: usize = std::mem::size_of::<u32>();

/// A single vertex in 3D space
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vertex {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Z position
    pub z: f32,

    /// The color of the vertex (r, g, b, a)
    pub color: [f32; 4],
}

impl Vertex {
    /// Creates a new vertex
    ///
    /// # Arguments
    ///
    /// * `x` - The x position
    /// * `y` - The y position
    /// * `z` - The z position
    /// * `color` - The color (r, g, b, a)
    ///
    /// # Returns
    ///
    /// * [Self] - The new vertex
    pub fn new(x: f32, y: f32, z: f32, color: [f32; 4]) -> Self {
        Self {
            x, y, z, color
        }
    }

    /// Serialize a vertex the to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we use to append the data to
    ///
    /// # Returns
    ///
    /// * `Ok()` - Successfully serialized the vertex
    /// * `Err(`[Error]`)` - Failed to serialize the vertex
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

    /// Deserialize a vertex to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we should deserialize
    ///
    /// # Returns
    ///
    /// * `Ok(`[Self]`)` - Successfully deserialized the vertex
    /// * `Err(`[Error]`)` - Failed to deserialize the vertex
    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < VERTEX_SIZE {
            return Err(Error::BufferToSmallVertex);
        }

        let x = f32::from_le_bytes(
            buffer[0..4].try_into()
                .map_err(Error::SliceConvertionError)?);
        let y = f32::from_le_bytes(
            buffer[4..8].try_into()
            .map_err(Error::SliceConvertionError)?);
        let z = f32::from_le_bytes(
            buffer[8..12].try_into()
                .map_err(Error::SliceConvertionError)?);

        let r = f32::from_le_bytes(
            buffer[12..16].try_into()
                .map_err(Error::SliceConvertionError)?);
        let g = f32::from_le_bytes(
            buffer[16..20].try_into()
                .map_err(Error::SliceConvertionError)?);
        let b = f32::from_le_bytes(
            buffer[20..24].try_into()
                .map_err(Error::SliceConvertionError)?);
        let a = f32::from_le_bytes(
            buffer[24..28].try_into()
                .map_err(Error::SliceConvertionError)?);

        Ok(Vertex::new(x, y, z, [r, g, b, a]))
    }
}

pub struct Mesh {
    /// The vertex buffer of the mesh
    pub vertex_buffer: Vec<Vertex>,

    /// The index buffer of the mesh
    pub index_buffer: Vec<Index>,
}

impl Mesh {
    pub fn new(vertex_buffer: Vec<Vertex>, index_buffer: Vec<u32>) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    /// Serialize the mesh to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we use to append the data to
    ///
    /// # Returns
    ///
    /// * `Ok()` - Successfully serialized the mesh
    /// * `Err(`[Error]`)` - Failed to serialize the mesh
    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<()> {
        // Vertex buffer count
        let count: u64 =
            self.vertex_buffer.len().try_into()
                .map_err(Error::IntegerConvertionError)?;
        buffer.extend_from_slice(&count.to_le_bytes());

        // Index buffer count
        let count: u64 = self.index_buffer.len().try_into()
            .map_err(Error::IntegerConvertionError)?;
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

    /// Deserialize the mesh to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we should deserialize
    ///
    /// # Returns
    ///
    /// * `Ok(`[Self]`)` - Successfully deserialized the mesh
    /// * `Err(`[Error]`)` - Failed to deserialize the mesh
    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        if buffer.len() < std::mem::size_of::<u64>() * 2 {
            // TODO(patrik): Change this error
            return Err(Error::BufferToSmallSector);
        }

        let vertex_count = u64::from_le_bytes(
            buffer[0..8].try_into()
                .map_err(Error::SliceConvertionError)?);
        let vertex_count: usize = vertex_count.try_into()
            .map_err(Error::IntegerConvertionError)?;

        let index_count = u64::from_le_bytes(
            buffer[8..16].try_into()
                .map_err(Error::SliceConvertionError)?);
        let index_count: usize = index_count.try_into()
            .map_err(Error::IntegerConvertionError)?;

        let buffer = &buffer[16..];

        if buffer.len() < VERTEX_SIZE * vertex_count {
            return Err(Error::BufferToSmallSector);
        }

        let mut vertex_buffer = Vec::with_capacity(vertex_count);

        for i in 0..vertex_count {
            let start = i * VERTEX_SIZE;
            let buffer = &buffer[start..start + VERTEX_SIZE];
            let vertex = Vertex::deserialize(buffer)?;
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
                    .map_err(Error::SliceConvertionError)?);
            index_buffer.push(index);
        }

        Ok(Self::new(vertex_buffer, index_buffer))
    }
}

/// A sector of the map contains the mesh
pub struct Sector {
    pub floor_mesh: Mesh,
    pub ceiling_mesh: Mesh,
    pub wall_mesh: Mesh,
}

impl Sector {
    /// Creates a new sector
    ///
    /// # Arguments
    ///
    /// * `floor_mesh`   - The mesh of the floor for this sector
    /// * `ceiling_mesh` - The mesh of the ceiling for this sector
    /// * `wall_mesh`    - The mesh of the walls for this sector
    ///
    /// # Returns
    ///
    /// * [`Self`] - The new sector
    pub fn new(floor_mesh: Mesh, ceiling_mesh: Mesh, wall_mesh: Mesh)
        -> Sector
    {
        Self {
            floor_mesh,
            ceiling_mesh,
            wall_mesh,
        }
    }

    /// Serialize the sector to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we use to append the data to
    ///
    /// # Returns
    ///
    /// * `Ok()` - Successfully serialized the sector
    /// * `Err(`[Error]`)` - Failed to serialize the sector
    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<()> {
        let mut temp_buffer = Vec::new();
        self.floor_mesh.serialize(&mut temp_buffer)?;

        let size: u64 = temp_buffer.len().try_into()
            .map_err(Error::IntegerConvertionError)?;

        buffer.extend_from_slice(&size.to_le_bytes());
        buffer.extend_from_slice(&temp_buffer);

        let mut temp_buffer = Vec::new();
        self.ceiling_mesh.serialize(&mut temp_buffer)?;

        let size: u64 = temp_buffer.len().try_into()
            .map_err(Error::IntegerConvertionError)?;

        buffer.extend_from_slice(&size.to_le_bytes());
        buffer.extend_from_slice(&temp_buffer);

        let mut temp_buffer = Vec::new();
        self.wall_mesh.serialize(&mut temp_buffer)?;

        let size: u64 = temp_buffer.len().try_into()
            .map_err(Error::IntegerConvertionError)?;

        buffer.extend_from_slice(&size.to_le_bytes());
        buffer.extend_from_slice(&temp_buffer);

        Ok(())
    }

    // TODO(patrik): Change this comment
    /// Deserialize the sector to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we should deserialize
    ///
    /// # Returns
    ///
    /// * `Ok(`[Self]`)` - Successfully deserialized the mesh
    /// * `Err(`[Error]`)` - Failed to deserialize the mesh
    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        let floor_mesh_size = u64::from_le_bytes(
            buffer[0..8].try_into()
                .map_err(Error::SliceConvertionError)?);
        let floor_mesh_size: usize = floor_mesh_size.try_into()
            .map_err(Error::IntegerConvertionError)?;
        let buffer = &buffer[8..];

        let floor_mesh = Mesh::deserialize(&buffer[0..floor_mesh_size])?;
        let buffer = &buffer[floor_mesh_size..];

        let ceiling_mesh_size = u64::from_le_bytes(buffer[0..8].try_into().map_err(Error::SliceConvertionError)?);
        let ceiling_mesh_size: usize = ceiling_mesh_size.try_into()
            .map_err(Error::IntegerConvertionError)?;
        let buffer = &buffer[8..];

        let ceiling_mesh = Mesh::deserialize(&buffer[0..ceiling_mesh_size])?;
        let buffer = &buffer[ceiling_mesh_size..];

        let wall_mesh_size = u64::from_le_bytes(buffer[0..8].try_into().map_err(Error::SliceConvertionError)?);
        let wall_mesh_size: usize = wall_mesh_size.try_into()
            .map_err(Error::IntegerConvertionError)?;
        let buffer = &buffer[8..];

        let wall_mesh = Mesh::deserialize(&buffer[0..wall_mesh_size])?;

        Ok(Sector::new(floor_mesh, ceiling_mesh, wall_mesh))
    }
}

/// The map structure containing infomation about the map
pub struct Map {
    /// The sectors of the map
    pub sectors: Vec<Sector>,
}

impl Map {
    /// Create a new map structure
    ///
    /// # Arguments
    ///
    /// * `sectors` - Map sectors
    ///
    /// # Returns
    ///
    /// * [Self] - Returns the created map structure
    pub fn new(sectors: Vec<Sector>) -> Self {
        Self {
            sectors
        }
    }

    /// Serialize the map to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we use to append the data to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully serialized the map
    /// * `Err(`[Error]`)` - Failed to serialize the map
    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<()> {
        // Magic
        buffer.extend_from_slice(b"MIME");
        // Version
        buffer.extend_from_slice(&CURRENT_VERSION.to_le_bytes());

        // Serialize the sector count
        let count: u64 =
            self.sectors.len().try_into()
                .map_err(Error::IntegerConvertionError)?;
        buffer.extend_from_slice(&count.to_le_bytes());

        // Serialize all the sectors
        for sector in &self.sectors {
            let mut sector_buffer = Vec::new();
            sector.serialize(&mut sector_buffer)?;

            let sector_size: u64 =
                sector_buffer.len().try_into()
                    .map_err(Error::IntegerConvertionError)?;
            buffer.extend_from_slice(&sector_size.to_le_bytes());
            buffer.extend_from_slice(&sector_buffer);
        }

        Ok(())
    }

    /// Deserialize the buffer and create a map structure
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer we should deserialize
    ///
    /// # Returns
    ///
    /// * `Ok(`[Map]`)` - Successfully derserialized the data and created a
    ///                   map structure
    /// * `Err(`[Error]`)` - Failed to deserialize the data
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
                .map_err(Error::SliceConvertionError)?);
        if version != CURRENT_VERSION {
            return Err(Error::IncorrectVersion);
        }

        let buffer = &buffer[8..];

        if buffer.len() < std::mem::size_of::<u64>() {
            return Err(Error::BufferToSmallMap);
        }

        let sector_count = u64::from_le_bytes(
            buffer[0..8].try_into()
                .map_err(Error::SliceConvertionError)?);
        let sector_count: usize = sector_count.try_into()
            .map_err(Error::IntegerConvertionError)?;

        let buffer = &buffer[8..];

        let mut sectors = Vec::with_capacity(sector_count);

        let mut offset = 0;

        for _i in 0..sector_count {
            let start = offset;
            let sector_size = u64::from_le_bytes(
                buffer[start..start + 8].try_into()
                    .map_err(Error::SliceConvertionError)?);
            let sector_size: usize =
                sector_size.try_into()
                    .map_err(Error::IntegerConvertionError)?;
            let start = start + 8;

            let sector =
                Sector::deserialize(&buffer[start..start + sector_size])?;
            sectors.push(sector);

            offset += sector_size + 8;
        }

        Ok(Self::new(sectors))
    }

    /// Serialize the map and write the serialized data to a file
    ///
    /// # Arguments
    ///
    /// * `filename` - Filename of the file we should create to write
    ///                the serialized data to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully serialized the map and wrote the date
    ///              to the file
    /// * `Err(`[Error]`)` - Failed to serialize the map or write the data
    ///                      to the file
    pub fn save_to_file<P>(&self, filename: P) -> Result<()>
        where P: AsRef<Path>
    {
        // Create the buffer holding the serialized data
        let mut buffer = Vec::new();

        // Serialize the map
        self.serialize(&mut buffer)?;

        // Write the buffer to a file
        let mut file = File::create(filename)
            .map_err(Error::FileCreationFailed)?;
        file.write_all(&buffer[..])
            .map_err(Error::FileWriteFailed)?;

        Ok(())
    }
}
