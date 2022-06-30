struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

struct VertexInput {
    [[location(0)]] data: u32;
};

[[stage(vertex)]]
fn vs_main(
    data: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let x = data.data & 0xf800000u;
    let y = data.data & 0x7c00000u;
    let z = data.data & 0x3e0000u;
    out.clip_position = vec4<f32>(f32(x), f32(y), f32(z), 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}