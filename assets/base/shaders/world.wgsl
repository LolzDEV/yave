struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct VertexInput {
    @location(0) data: u32,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    data: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let x = data.data >> 27u;
    let y = (data.data & 0x7c00000u) >> 22u;
    let z = (data.data & 0x3e0000u) >> 17u;
    out.clip_position = camera.view_proj * vec4<f32>(f32(x), f32(y), f32(z), 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}