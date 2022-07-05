use self::chunk::Chunk;

pub mod chunk;

pub struct Chunks {
    pub chunks: Vec<Chunk>,
}

impl Chunks {
    pub fn get_chunk(&self, x: i64, y: i64) -> Option<&Chunk> {
        for chunk in self.chunks.iter() {
            if chunk.x == x && chunk.y == y {
                return Some(chunk);
            }
        }

        None
    }
}

impl Default for Chunks {
    fn default() -> Self {
        Self { chunks: Vec::new() }
    }
}
