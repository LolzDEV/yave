use yave::client::voxel::VoxelVertex;

#[test]
pub fn voxel_vertex() {
    let vertex = VoxelVertex::new(5, 4, 1, 3, 5);

    assert_eq!(vertex.x(), 5);
    assert_eq!(vertex.y(), 4);
    assert_eq!(vertex.z(), 1);
}