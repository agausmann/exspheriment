struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var CLIP_POSITION = array<vec4<f32>, 4>(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 1.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 0.0, 1.0),
        vec4<f32>(1.0, -1.0, 0.0, 1.0),
    );
    var UV = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    var INDICES = array<i32, 6>(0, 1, 2, 2, 3, 0);

    var out: VertexOutput;
    let index = INDICES[vertex_index];
    out.clip_position = CLIP_POSITION[index];
    out.uv = UV[index];
    return out;
}

[[group(0), binding(0)]]
var hud_texture: texture_2d<f32>;
[[group(0), binding(1)]]
var hud_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(hud_texture, hud_sampler, in.uv);
}