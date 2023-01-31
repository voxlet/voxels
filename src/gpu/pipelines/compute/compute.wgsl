[[block]]
struct PixelBuffer {
  pixels: array<vec4<f32>>;
};

[[group(0), binding(0)]]
var<storage> pixelBuffer: [[access(write)]] PixelBuffer;

[[block]]
struct Screen {
  res: vec2<f32>;
};

[[group(0), binding(1)]]
var<uniform> screen: Screen;

[[stage(compute), workgroup_size(1)]]
fn main(
  [[builtin(global_invocation_id)]]
  gid: vec3<u32>
) {
  let pixelIndex: u32 = gid.y * u32(screen.res.x) + gid.x;
  let color: vec4<f32> = vec4<f32>(vec2<f32>(gid.xy) / screen.res, 1.0, 1.0);
  pixelBuffer.pixels[pixelIndex] = color;
}
