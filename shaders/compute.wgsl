#include "state.wgsli"

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

var<private> voxel_radius: vec3<f32>;
var<private> voxel_inv_radius: vec3<f32>;

struct Ray {
    origin: vec3<f32>;
    direction: vec3<f32>;
    inv_direction: vec3<f32>;
};

struct Box {
    center: vec3<f32>;
    radius: vec3<f32>;
    inv_radius: vec3<f32>;
};

struct Hit {
    distance: f32;
    normal: vec3<f32>;
};

fn box_at(point: vec3<f32>) -> Box {
    var box: Box;
    box.radius = voxel_radius;
    box.inv_radius = voxel_inv_radius;

    let box_size = box.radius * 2.0;
    box.center = floor(point / box_size) * box_size + box.radius;

    return box;
}

fn intersect(ray: Ray, box: Box) -> Hit {
    let origin = ray.origin - box.center;

    var sgn: vec3<f32> = -sign(ray.direction);
    let distance_to_plane =
        (-box.radius * sgn - origin) * ray.inv_direction;

    let test: vec3<bool> = vec3<bool>(
        (distance_to_plane.x >= 0.0) &&
        all(abs(origin.yz + ray.direction.yz * distance_to_plane.x) < box.radius.yz),
        (distance_to_plane.y >= 0.0) &&
        all(abs(origin.zx + ray.direction.zx * distance_to_plane.y) < box.radius.zx),
        (distance_to_plane.z >= 0.0) &&
        all(abs(origin.xy + ray.direction.xy * distance_to_plane.z) < box.radius.xy),
    );

    sgn = select(sgn, vec3<f32>(0.0, 0.0, 0.0), test);

    let dist = select(
        distance_to_plane.x,
        select(
            distance_to_plane.y,
            distance_to_plane.z,
            sgn.y != 0.0
        ),
        sgn.x != 0.0
    );

    var hit: Hit;
    hit.distance = dist;
    hit.normal = -sgn;
    return hit;
}

var max_ray_distance: f32 = 1.4143;
fn trace_ray(ray: Ray) -> vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var box: Box = box_at(ray.origin);
    var exit: Hit;
    var count: i32 = 0;
    loop {
        count = count + 1;
        exit = intersect(ray, box);
        box = box_at(box.center + vec3<f32>(box.radius * 2.0) * exit.normal);
        color = textureSampleLevel(
            voxels,
            voxel_sampler,
            box.center,
            0.0
        );
        if (color.a > 0.5 || count > 2048 || exit.distance > max_ray_distance) {
            color = color * (1.0 + dot(exit.normal, vec3<f32>(1.0, 2.0, 1.0)) * -0.3);
            color = select(vec4<f32>(1.0,0.0,0.0,1.0), color, count > 1024);
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
    ray.inv_direction = 1.0 / ray.direction;
    return ray;
}

[[stage(compute), workgroup_size(30, 30)]]
fn main(
    [[builtin(global_invocation_id)]]
    gid: vec3<u32>
) {
    voxel_radius = vec3<f32>(state.voxel_size * 0.5);
    voxel_inv_radius = 1.0 / voxel_radius;

    let ray: Ray = ray_for(gid);
    let pixel_index: u32 = gid.y * u32(state.resolution.x) + gid.x;
    let color: vec4<f32> = trace_ray(ray);

    pixel_buffer.pixels[pixel_index] = color;

    // pixel_buffer.pixels[pixel_index] = textureSampleLevel(voxels, voxel_sampler, vec3<f32>(vec2<f32>(gid.xy) / state.resolution, 0.5), 0.0);
}
