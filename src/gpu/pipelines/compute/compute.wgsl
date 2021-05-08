[[block]]
struct State {
  camera_rotation: mat3x3<f32>;
  camera_position: vec3<f32>;
  resolution: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> state: State;

[[block]]
struct PixelBuffer {
  pixels: array<vec4<f32>>;
};

[[group(0), binding(1)]]
var<storage> pixel_buffer: [[access(write)]] PixelBuffer;

[[group(0), binding(2)]]
var voxels: texture_3d<f32>;

[[group(0), binding(3)]]
var voxel_sampler: sampler;

struct Ray {
  origin: vec3<f32>;
  direction: vec3<f32>;
};

fn trace_ray(ray: Ray) -> vec4<f32> {
  let voxel_size = 1.0 / 256.0;
  let max_ray_distance = 1.0;

  var ray_distance: f32 = 0.00001;
  var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
  var ray_end: vec3<f32>;
  loop {
    ray_end = ray.origin + ray.direction * ray_distance;
    color = textureSampleLevel(voxels, voxel_sampler, ray_end, 0.0);
    if (color.a > 0.5) {
      break;
    }
    ray_distance = ray_distance + voxel_size * 0.25;
    if (ray_distance > max_ray_distance) {
      break;
    }
  };
  return color;
}

fn ray_for(gid: vec3<u32>) -> Ray {
  var ray: Ray;
  ray.origin = state.camera_position;
  var xy: vec2<f32> = vec2<f32>(gid.xy) / state.resolution * 2.0 - vec2<f32>(1.0, 1.0);
  xy.x = xy.x * state.resolution.x / state.resolution.y;
  ray.direction = normalize(state.camera_rotation * vec3<f32>(xy, 1.0));
  return ray;
}

[[stage(compute), workgroup_size(32, 32)]]
fn main(
  [[builtin(global_invocation_id)]]
  gid: vec3<u32>
) {
  let ray: Ray = ray_for(gid);
  let pixel_index: u32 = gid.y * u32(state.resolution.x) + gid.x;
  let color: vec4<f32> = trace_ray(ray);
  pixel_buffer.pixels[pixel_index] = color;
  //pixel_buffer.pixels[pixel_index] = textureSampleLevel(voxels, voxel_sampler, vec3<f32>(vec2<f32>(gid.xy) / state.resolution, 0.5), 0.0);
}
