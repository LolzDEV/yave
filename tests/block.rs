use yave::assets::Identifier;
use yave::client::voxel::VoxelVertex;
use yave::world::chunk::Block;

#[test]
pub fn voxel_vertex() {
    let vertex = VoxelVertex::new(5, 4, 1, 3, 5);

    assert_eq!(vertex.x(), 5);
    assert_eq!(vertex.y(), 4);
    assert_eq!(vertex.z(), 1);
}

#[test]
pub fn block_encoding() {
    let block = Block::new(4, 2, 7, Identifier::new("base", "grass"));

    assert_eq!(block.x(), 4);
    assert_eq!(block.y(), 2);
    assert_eq!(block.z(), 7);
}
