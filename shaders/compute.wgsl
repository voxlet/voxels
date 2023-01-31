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

var<private> voxel_radius: array<vec3<f32>, 10>;
var<private> voxel_inv_radius: array<vec3<f32>, 10>;
var<private> max_mip_level: u32;

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

fn box_at(point: vec3<f32>, mip_level: u32) -> Box {
    var box: Box;
    box.radius = voxel_radius[mip_level];
    box.inv_radius = voxel_inv_radius[mip_level];

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

    return Intersection(dist, sgn);
}

struct Hit {
    hit: bool;
    intersection: Intersection;
    steps: u32;
    voxel: vec4<f32>;
};

let max_march_steps: u32 = 128u;
fn march_ray(ray: Ray) -> Hit {
    var res: Hit;
    res.hit = false;
    res.steps = 0u;

    var voxel: vec4<f32> = vec4<f32>(0.0);

    var box: Box = box_at(ray.origin, 0u);
    var exit: Intersection = intersect(ray, box);
    var exit_point: vec3<f32>;
    var mip_level: u32 = 0u;
    var prev_voxel: vec4<f32>;
    var min_mip_level: u32;
    loop {
        // min_mip_level = max(0u, u32(log2(exit.distance)));
        exit_point = ray.origin + ray.direction * exit.distance;
        loop {
            box = box_at(
                exit_point - exit.normal * state.voxel_size * 0.5,
                mip_level
            );
            prev_voxel = voxel;
            voxel = select(
                vec4<f32>(0.0),
                textureSampleLevel(
                    voxels,
                    voxel_sampler,
                    box.center,
                    f32(mip_level)
                ),
                any(box.center > vec3<f32>(1.0))
            );
            if (voxel.a > 0.0) {
                // if (mip_level <= min_mip_level) {
                if (mip_level == 0u) {
                    break;
                }
                mip_level = mip_level - 1u;
                continue;
            }
            if (prev_voxel.a > 0.0 || mip_level == max_mip_level) {
                break;
            }
            mip_level = mip_level + 1u;
        }

        if (voxel.a > 0.0 || res.steps > max_march_steps) {
            res.hit = voxel.a > 0.0;
            res.intersection = exit;
            res.voxel = voxel;
            return res;
        }

        exit = intersect(ray, box);
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

let light: vec3<f32> = vec3<f32>(0.0, 3.0, 0.0);
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

[[stage(compute), workgroup_size(8, 8)]]
fn main(
    [[builtin(global_invocation_id)]]
    gid: vec3<u32>
) {
    max_mip_level = u32(ceil(log2(1.0 / state.voxel_size))) - 1u;
    var i: u32 = 0u;
    loop {
        voxel_radius[i] = vec3<f32>(state.voxel_size * 0.5 * exp2(f32(i)));
        voxel_inv_radius[i] = 1.0 / voxel_radius[i];
        i = i + 1u;
        if (i > max_mip_level) {
            break;
        }
    }

    let ray: Ray = ray_for(gid);
    let pixel_index: u32 = gid.y * u32(state.resolution.x) + gid.x;
    let color: vec4<f32> = trace_ray(ray);

    pixel_buffer.pixels[pixel_index] = color;

    // pixel_buffer.pixels[pixel_index] = textureSampleLevel(voxels, voxel_sampler, vec3<f32>(vec2<f32>(gid.xy) / state.resolution, 0.5), 0.0);
}
