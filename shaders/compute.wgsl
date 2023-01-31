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

fn box_at(point: vec3<f32>) -> Box {
    var box: Box;
    box.radius = voxel_radius;
    box.inv_radius = voxel_inv_radius;

    let box_size = box.radius * 2.0;
    box.center = floor(point / box_size) * box_size + box.radius;

    return box;
}

struct Intersection {
    distance: f32;
    normal: vec3<f32>;
};

fn intersect(ray: Ray, box: Box) -> Intersection {
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

    var res: Intersection;
    res.distance = dist;
    res.normal = sgn;
    return res;
}

struct Hit {
    hit: bool;
    intersection: Intersection;
    steps: u32;
    voxel: vec4<f32>;
};

let max_march_steps: u32 = 2048u;
let max_ray_distance: f32 = 1.4143;
fn march_ray(ray: Ray) -> Hit {
    var res: Hit;
    res.hit = false;
    res.steps = 0u;

    var exit: Intersection;
    var voxel: vec4<f32> = vec4<f32>(0.0);
    var box: Box = box_at(ray.origin);
    loop {
        exit = intersect(ray, box);
        box = box_at(box.center - vec3<f32>(box.radius * 2.0) * exit.normal);
        voxel = select(
            vec4<f32>(0.0),
            textureSampleLevel(
                voxels,
                voxel_sampler,
                box.center,
                0.0
            ),
            any(box.center > vec3<f32>(1.0))
        );
        if (voxel.a > 0.5 ||
            res.steps > max_march_steps ||
            exit.distance > max_ray_distance) {
            res.hit = voxel.a > 0.5;
            res.intersection = exit;
            res.voxel = voxel;
            return res;
        }
        res.steps = res.steps + 1u;
    };
    return res;
}

fn ray_from(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    var ray: Ray;
    ray.origin = origin;
    ray.direction = normalize(direction);
    ray.inv_direction = 1.0 / ray.direction;
    return ray;
}

let light: vec3<f32> = vec3<f32>(1.0, 3.0, 1.0);
let shadow_intensity: f32 = 0.9;
fn trace_ray(ray: Ray) -> vec4<f32> {
    let hit = march_ray(ray);
    if (!hit.hit) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // shadow
    let hit_point = ray.direction * hit.intersection.distance + ray.origin;
    let light_dir = light - hit_point;
    let irradiance = dot(hit.intersection.normal, light_dir);
    if (irradiance <= 0.0) {
        return vec4<f32>(hit.voxel.rgb * (1.0 - shadow_intensity), hit.voxel.a);
    }
    let direct_light_ray = ray_from(hit_point, light_dir);
    let shadow = march_ray(direct_light_ray);

    return select(
        vec4<f32>(hit.voxel.rgb * (1.0 - shadow_intensity), hit.voxel.a),
        vec4<f32>(
            hit.voxel.rgb * (1.0 - shadow_intensity * (1.0 - irradiance)),
            hit.voxel.a
        ),
        shadow.hit
    );
}

fn ray_for(gid: vec3<u32>) -> Ray {
    var xy: vec2<f32> = vec2<f32>(gid.xy) / state.resolution * 2.0 - vec2<f32>(1.0, 1.0);
    xy.x = xy.x * state.resolution.x / state.resolution.y;
    return ray_from(
        state.camera_position,
        state.camera_rotation * vec3<f32>(xy, 1.0)
    );
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
