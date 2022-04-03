pub use map::{ Vertex, Map, Sector };

pub mod map;

#[cfg(test)]
mod tests {
    use crate::map::*;

    macro_rules! parse_u32 {
        ($buf:expr, $i:expr) => {{
            let res = u32::from_le_bytes($buf[$i..$i + 4].try_into().unwrap());
            $i += 4;
            res
        }}
    }

    macro_rules! parse_u64 {
        ($buf:expr, $i:expr) => {{
            let res = u64::from_le_bytes($buf[$i..$i + 8].try_into().unwrap());
            $i += 8;
            res
        }}
    }

    macro_rules! parse_f32 {
        ($buf:expr, $i:expr) => {{
            let res = f32::from_le_bytes($buf[$i..$i + 4].try_into().unwrap());
            $i += 4;
            res
        }}
    }

    macro_rules! skip {
        ($x:expr, $n:expr) => {{
            $x += $n;
        }}
    }

    #[test]
    fn vertex_serialize() {
        let vertex = Vertex::new(0.0, 1.0, 2.0, [3.0, 4.0, 5.0, 6.0]);
        let mut buffer = Vec::new();
        vertex.serialize(&mut buffer);

        let mut index = 0;

        assert_eq!(parse_f32!(buffer, index), 0.0);
        assert_eq!(parse_f32!(buffer, index), 1.0);
        assert_eq!(parse_f32!(buffer, index), 2.0);

        assert_eq!(parse_f32!(buffer, index), 3.0);
        assert_eq!(parse_f32!(buffer, index), 4.0);
        assert_eq!(parse_f32!(buffer, index), 5.0);
        assert_eq!(parse_f32!(buffer, index), 6.0);
    }

    #[test]
    fn sector_serialize() {
        let mut vertex_buffer = Vec::new();
        vertex_buffer.push(Vertex::new(0.0, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(0.0, 1.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(1.0, 1.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(1.0, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]));

        let index_buffer = vec![0, 1, 2, 2, 3, 0];

        let sector = Sector::new(vertex_buffer, index_buffer);
        let mut buffer = Vec::new();
        sector.serialize(&mut buffer);

        let mut index = 0;

        assert_eq!(parse_u64!(buffer, index), 4);
        assert_eq!(parse_u64!(buffer, index), 6);

        // TODO(patrik): Test vertices?
        skip!(index, 4 * std::mem::size_of::<Vertex>());

        assert_eq!(parse_u32!(buffer, index), 0);
        assert_eq!(parse_u32!(buffer, index), 1);
        assert_eq!(parse_u32!(buffer, index), 2);
        assert_eq!(parse_u32!(buffer, index), 2);
        assert_eq!(parse_u32!(buffer, index), 3);
        assert_eq!(parse_u32!(buffer, index), 0);
    }

    #[test]
    fn map_serialize() {
        let mut vertex_buffer = Vec::new();
        vertex_buffer.push(Vertex::new(0.0, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(0.0, 1.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(1.0, 1.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(1.0, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]));

        let index_buffer = vec![0, 1, 2, 2, 3, 0];

        let mut sectors = Vec::new();
        sectors.push(Sector::new(vertex_buffer, index_buffer));

        let map = Map::new(sectors);

        let mut buffer = Vec::new();
        map.serialize(&mut buffer);

        let mut index = 0;

        // Test the header
        assert_eq!(&buffer[0..4], b"MIME");
        skip!(index, 4);

        assert_eq!(parse_u32!(buffer, index), CURRENT_VERSION);

        assert_eq!(parse_u64!(buffer, index), 1);
    }
}
