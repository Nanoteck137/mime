//! Module for all the unit tests

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
        vertex.serialize(&mut buffer).unwrap();

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
        sector.serialize(&mut buffer).unwrap();

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
        map.serialize(&mut buffer).unwrap();

        let mut index = 0;

        // Test the header
        assert_eq!(&buffer[0..4], b"MIME");
        skip!(index, 4);

        assert_eq!(parse_u32!(buffer, index), CURRENT_VERSION);

        assert_eq!(parse_u64!(buffer, index), 1);
    }

    #[test]
    fn vertex_deserialize() {
        let vertex = Vertex::new(0.0, 1.0, 2.0, [3.0, 4.0, 5.0, 6.0]);

        let mut buffer = Vec::new();
        vertex.serialize(&mut buffer).unwrap();

        let result = Vertex::deserialize(&buffer).unwrap();

        assert_eq!(result, vertex);
    }

    fn compare_sector(a: &Sector, b: &Sector) {
        assert_eq!(a.vertex_buffer.len(), b.vertex_buffer.len());
        for index in 0..a.vertex_buffer.len() {
            assert_eq!(a.vertex_buffer[index], b.vertex_buffer[index]);
        }

        assert_eq!(a.index_buffer.len(), b.index_buffer.len());
        for index in 0..a.index_buffer.len() {
            assert_eq!(a.index_buffer[index], a.index_buffer[index]);
        }
    }

    #[test]
    fn sector_deserialize() {
        let mut vertex_buffer = Vec::new();
        vertex_buffer.push(Vertex::new(0.0, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(0.0, 1.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(1.0, 1.0, 0.0, [1.0, 1.0, 1.0, 1.0]));
        vertex_buffer.push(Vertex::new(1.0, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]));

        let index_buffer = vec![0, 1, 2, 2, 3, 0];

        let sector = Sector::new(vertex_buffer, index_buffer);
        let mut buffer = Vec::new();
        sector.serialize(&mut buffer).unwrap();

        let result = Sector::deserialize(&buffer).unwrap();

        compare_sector(&result, &sector);
    }

    #[test]
    fn map_deserialize() {
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
        map.serialize(&mut buffer).unwrap();

        let result = Map::deserialize(&buffer).unwrap();

        assert_eq!(result.sectors.len(), map.sectors.len());

        for i in 0..result.sectors.len() {
            compare_sector(&result.sectors[i], &map.sectors[i]);
        }
    }
}
