[[block]]
struct State {
  resolution: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> state: State;

[[block]]
struct PixelBuffer {
  pixels: array<vec4<f32>>;
};

[[group(0), binding(1)]]
var<storage> pixelBuffer: [[access(write)]] PixelBuffer;


[[stage(compute), workgroup_size(32, 32)]]
fn main(
  [[builtin(global_invocation_id)]]
  gid: vec3<u32>
) {
  let pixelIndex: u32 = gid.y * u32(state.resolution.x) + gid.x;
  let color: vec4<f32> = vec4<f32>(vec2<f32>(gid.xy) / state.resolution, 1.0, 1.0);
  pixelBuffer.pixels[pixelIndex] = color;
}
