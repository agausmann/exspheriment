struct Viewport {
    view_proj: mat4x4<f32>;
    eye_zn: vec4<f32>;
    forward_xfov: vec4<f32>;
    up_yfov: vec4<f32>;
};

struct Point {
    position_size: vec4<f32>;
    color: vec4<f32>;
};

struct Points {
    points: array<Point>;
};

[[group(0), binding(0)]]
var<uniform> viewport: Viewport;

[[group(1), binding(0)]]
var output: texture_storage_2d<rgba8unorm, write>;

[[group(1), binding(1)]]
var<storage, read> points: Points;

[[stage(compute), workgroup_size(1)]]
fn test_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let dims = textureDimensions(output);
    let texel = vec2<i32>(id.xy);
    let color = vec4<f32>(vec2<f32>(id.xy) / vec2<f32>(dims), 0.0, 1.0);
    textureStore(output, texel, color);
}

[[stage(compute), workgroup_size(1)]]
fn point_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let eye = viewport.eye_zn.xyz;
    let zn = viewport.eye_zn.w;

    let forward = normalize(viewport.forward_xfov.xyz);
    let xfov = viewport.forward_xfov.w;

    let up = normalize(viewport.up_yfov.xyz);
    let yfov = viewport.up_yfov.w;

    let point = points.points[id.z].position_size.xyz;
    let size = points.points[id.z].position_size.w;
    let color = points.points[id.z].color;

    let right = normalize(cross(forward, up));
    let dims = textureDimensions(output);

    let dx = tan(xfov * 0.5) * 2.0 / f32(dims.x - 1);
    let dy = tan(yfov * 0.5) * 2.0 / f32(dims.y - 1);

    let eye_ray = normalize(
        forward
        + dx * (f32(id.x) - f32(dims.x) * 0.5) * right
        - dy * (f32(id.y) - f32(dims.y) * 0.5) * up
    );

    // textureStore(output, vec2<i32>(id.xy), vec4<f32>(eye_ray, 1.0));

    let t0 = (dot(eye_ray, point) - dot(eye_ray, eye)) / dot(eye_ray, eye_ray);

    let p_a = viewport.view_proj * vec4<f32>(eye + eye_ray * t0, 1.0);
    let p_b = viewport.view_proj * vec4<f32>(point, 1.0);
    let p_a = p_a.xy / p_a.w;
    let p_b = p_b.xy / p_b.w;
    let distance = length((p_b - p_a) * vec2<f32>(dims) * 0.5);

    if (t0 >= 0.0 && distance <= size) {
        textureStore(output, vec2<i32>(id.xy), color);
    }
}