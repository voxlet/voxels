#include "compute_globals.wgsli"
#include "ray.wgsli"
#include "trace.wgsli"

[[stage(compute), workgroup_size(8, 8)]]
fn main(
    [[builtin(global_invocation_id)]]
    gid: vec3<u32>
) {
    init_globals();

    let ray: Ray = ray_for(gid);
    let pixel_index: u32 = gid.y * u32(state.resolution.x) + gid.x;
    let color: vec4<f32> = trace_ray(ray);

    pixel_buffer.pixels[pixel_index] = color;

    // pixel_buffer.pixels[pixel_index] = textureSampleLevel(voxels, voxel_sampler, vec3<f32>(vec2<f32>(gid.xy) / state.resolution, 0.5), 0.0);
}
