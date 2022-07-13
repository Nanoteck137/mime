//! Mime is a library for a simple Map format used primarily for my 3D engines
#![warn(missing_docs)]

pub use map::{ Mime, Map, Sector, Mesh, Vertex };

pub mod map;

#[cfg(test)]
mod tests;

/// Error enum for all the errors for this library
#[derive(Debug)]
pub enum Error {
    /// Failed to convert a slice to an array
    SliceConvertionError(std::array::TryFromSliceError),

    /// Failed to convert a integer
    IntegerConvertionError(std::num::TryFromIntError),

    /// Failed to create file
    FileCreationFailed(std::io::Error),

    /// Failed write to file
    FileWriteFailed(std::io::Error),

    /// Deserialization failed with incorrect magic
    IncorrectMagic,

    /// Deserialization failed with incorrect version
    IncorrectVersion,

    /// Deserialization of vertex failed, the buffer is too small to
    /// parse data from
    BufferToSmallVertex,

    /// Deserialization of sector failed, the buffer is too small to
    /// parse data from
    BufferToSmallSector,

    /// Deserialization of map failed, the buffer is too small to
    /// parse data from
    BufferToSmallMap,
}

/// A Result type for the library
pub type Result<T> = std::result::Result<T, Error>;
