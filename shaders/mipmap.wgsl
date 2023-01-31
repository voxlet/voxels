[[block]]
struct Args {
    size: u32;
    mip_level: u32;
};

[[group(0), binding(0)]]
var<uniform> args: Args;

[[group(0), binding(1)]]
var input: [[access(read)]] texture_storage_3d<rgba8unorm>;

[[group(0), binding(2)]]
var output: [[access(write)]] texture_storage_3d<rgba8unorm>;

[[stage(compute), workgroup_size(2, 2, 2)]]
fn main(
    [[builtin(global_invocation_id)]]
    gid: vec3<u32>
) {
    // SPIR-V module not valid: Expected float vector type as Result Type:
    // VectorTimesScalar %30 = OpVectorTimesScalar %v3uint %24 %uint_2
    let p = vec3<i32>(vec3<f32>(gid) * 2.0);
    let v1 = textureLoad(input, p);
    let v2 = textureLoad(input, p + vec3<i32>(0,0,1));
    let v3 = textureLoad(input, p + vec3<i32>(0,1,0));
    let v4 = textureLoad(input, p + vec3<i32>(0,1,1));
    let v5 = textureLoad(input, p + vec3<i32>(1,0,0));
    let v6 = textureLoad(input, p + vec3<i32>(1,0,1));
    let v7 = textureLoad(input, p + vec3<i32>(1,1,0));
    let v8 = textureLoad(input, p + vec3<i32>(1,1,1));

    let v = v1+v2+v3+v4+v5+v6+v7+v8;
    textureStore(
        output,
        vec3<i32>(gid),
        vec4<f32>(vec3<f32>(v.rgb / v.a), select(1.0, 0.0, v.a > 0.0))
    );
}
