#include "state.include.wgsl"

[[group(0), binding(0)]]
var<uniform> state: State;

[[block]]
struct PixelBuffer {
    pixels: array<vec4<f32>>;
};

[[group(0), binding(1)]]
var<storage> pixelBuffer: [[access(read)]] PixelBuffer;

struct Vertex {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vert_main(
    [[builtin(vertex_index)]] in_vertex_index: u32
) -> Vertex {
    let x = f32((in_vertex_index << 1u) & 2u);
    let y = f32(in_vertex_index & 2u);
    let uv = vec2<f32>(x, y);
    var out: Vertex;
    out.pos = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

[[stage(fragment)]]
fn frag_main(in: Vertex) -> [[location(0)]] vec4<f32> {
    let buffer_coord = floor(in.uv * state.resolution);
    let pixel_index = u32(buffer_coord.y * state.resolution.x + buffer_coord.x);
    return pixelBuffer.pixels[pixel_index];
}
