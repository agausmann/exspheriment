struct Viewport {
    view_proj: mat4x4<f32>;
    eye: vec3<f32>;
    forward_xfov: vec4<f32>;
    up_yfov: vec4<f32>;
};

struct Point {
    position_size: vec4<f32>;
    color: vec4<f32>;
};

struct Line {
    start_size: vec4<f32>;
    end: vec3<f32>;
    color: vec4<f32>;
};

struct Ellipse {
    center_size: vec4<f32>;
    axis_1: vec3<f32>;
    axis_2: vec3<f32>;
    color: vec4<f32>;
};

struct Points {
    points: array<Point>;
};

struct Lines {
    lines: array<Line>;
};

struct Ellipses {
    ellipses: array<Ellipse>;
};

[[group(0), binding(0)]]
var<uniform> viewport: Viewport;

[[group(1), binding(0)]]
var output: texture_storage_2d<rgba8unorm, write>;

[[group(1), binding(1)]]
var<storage, read> points: Points;

[[group(1), binding(2)]]
var<storage, read> lines: Lines;

[[group(1), binding(3)]]
var<storage, read> ellipses: Ellipses;

fn eye_ray(id: vec3<u32>) -> vec3<f32> {
    let forward = normalize(viewport.forward_xfov.xyz);
    let xfov = viewport.forward_xfov.w;

    let up = normalize(viewport.up_yfov.xyz);
    let yfov = viewport.up_yfov.w;

    let right = normalize(cross(forward, up));
    let dims = textureDimensions(output);
    let dx = tan(xfov * 0.5) * 2.0 / f32(dims.x - 1);
    let dy = tan(yfov * 0.5) * 2.0 / f32(dims.y - 1);

    let eye_ray = normalize(
        forward
        + dx * (f32(id.x) - f32(dims.x) * 0.5) * right
        - dy * (f32(id.y) - f32(dims.y) * 0.5) * up
    );
    return eye_ray;
}

[[stage(compute), workgroup_size(1)]]
fn test_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let dims = textureDimensions(output);
    let texel = vec2<i32>(id.xy);
    let color = vec4<f32>(vec2<f32>(id.xy) / vec2<f32>(dims), 0.0, 1.0);
    textureStore(output, texel, color);
}

[[stage(compute), workgroup_size(64)]]
fn point_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= u32(dims.x) || id.y >= u32(dims.y)) {
        return;
    }

    let eye = viewport.eye;
    let eye_ray = eye_ray(id);

    let point = points.points[id.z].position_size.xyz;
    let point_size = points.points[id.z].position_size.w;
    let point_color = points.points[id.z].color;

    let t0 = (dot(eye_ray, point) - dot(eye_ray, eye)) / dot(eye_ray, eye_ray);

    let eye_approach = eye + eye_ray * t0;
    let p_eye = viewport.view_proj * vec4<f32>(eye_approach, 1.0);
    let p_point = viewport.view_proj * vec4<f32>(point, 1.0);
    let n_eye = p_eye.xy / p_eye.w;
    let n_point = p_point.xy / p_point.w;

    let distance = length((n_eye - n_point) * vec2<f32>(dims) * 0.5);

    if (t0 >= 0.0 && distance <= point_size) {
        textureStore(output, vec2<i32>(id.xy), point_color);
    }
}

[[stage(compute), workgroup_size(64)]]
fn line_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= u32(dims.x) || id.y >= u32(dims.y)) {
        return;
    }

    let eye = viewport.eye;
    let eye_ray = eye_ray(id);

    let line_start = lines.lines[id.z].start_size.xyz;
    let line_size = lines.lines[id.z].start_size.w;
    let line_end = lines.lines[id.z].end;
    let line_color = lines.lines[id.z].color;
    let line_ray = line_end - line_start;

    let a = dot(eye_ray, eye_ray);
    let b = dot(eye_ray, line_ray);
    let c = dot(eye_ray, line_start - eye);
    let d = dot(eye_ray, line_ray);
    let e = dot(line_ray, line_ray);
    let f = dot(line_ray, line_start - eye);

    let t0 = (c * e - f * b) / (a * e - d * b);
    let u0 = (c * d - f * a) / (a * e - d * b);
    let u0 = clamp(u0, 0.0, 1.0);

    let eye_approach = eye + eye_ray * t0;
    let line_approach = line_start + line_ray * u0;
    let p_eye = viewport.view_proj * vec4<f32>(eye_approach, 1.0);
    let p_line = viewport.view_proj * vec4<f32>(line_approach, 1.0);
    let n_eye = p_eye.xy / p_eye.w;
    let n_line = p_line.xy / p_line.w;

    let distance = length((n_eye - n_line) * vec2<f32>(dims) * 0.5);

    if (t0 >= 0.0 && distance <= line_size) {
        textureStore(output, vec2<i32>(id.xy), line_color);
    }
}

[[stage(compute), workgroup_size(64)]]
fn ellipse_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= u32(dims.x) || id.y >= u32(dims.y)) {
        return;
    }

    let eye = viewport.eye;
    let eye_ray = eye_ray(id);

    let el_center = ellipses.ellipses[id.z].center_size.xyz;
    let stroke_width = ellipses.ellipses[id.z].center_size.w;
    let el_axis_1 = ellipses.ellipses[id.z].axis_1;
    let el_axis_2 = ellipses.ellipses[id.z].axis_2;
    let el_color = ellipses.ellipses[id.z].color;

    let a = eye_ray;
    let u = el_axis_1;
    let v = el_axis_2;
    let w = el_center - eye;
    let n = cross(u, v);

    // Cull if ellipse is behind eye
    if (dot(w, a) < 0.0) {
        return;
    }

    // Find intersection of ray with elliptical plane;
    // cull if it is too far away from the ellipse curve (>1.25r or <0.75r).
    // This assumes that the stroke width is << 0.25r.
    let intersect = dot(w, n) / dot(a, n) * a - w;

    // These projections are divided by the lengths of u and v an extra time.
    // The resulting pair of projections creates a coordinate pair that is
    // related to the unit circle - if r is greater than 1, it is outside the
    // original ellipse.
    let u_proj = dot(intersect, u) / dot(u, u);
    let v_proj = dot(intersect, v) / dot(v, v);
    let r = sqrt(u_proj * u_proj + v_proj * v_proj);
    if (r > 1.25 || r < 0.75) {
        return;
    }

    // Otherwise continue with numerical approximation.

    // Initial guess, using the angle between u and the intersection point:
    let su = dot(normalize(intersect), normalize(u));
    let sv = dot(normalize(intersect), normalize(v));
    var theta = atan2(sv, su);

    for (var i = 0; i < 2; i = i + 1) {
        let e = u * cos(theta) + v * sin(theta) + w;
        let de = v * cos(theta) - u * sin(theta);
        let dde = -(u * cos(theta) + v * sin(theta));

        let g = dot(a, a) * dot(e, de) - dot(a, e) * dot(a, de);
        let dg = dot(a, a) * (dot(e, dde) + dot(de, de))
            - dot(a, de) * dot(a, de) - dot(a, e) * dot(a, dde);
        
        let delta = g / dg;
        theta = theta - delta;
    }

    let el_approach = el_axis_1 * cos(theta) + el_axis_2 * sin(theta) + el_center;
    let t = dot(el_approach - eye, a) / dot(a, a);
    let eye_approach = a * t + eye;

    let p_eye = viewport.view_proj * vec4<f32>(eye_approach, 1.0);
    let p_el = viewport.view_proj * vec4<f32>(el_approach, 1.0);
    let n_eye = p_eye.xy / p_eye.w;
    let n_el = p_el.xy / p_el.w;

    let distance = length((n_eye - n_el) * vec2<f32>(dims) * 0.5);

    if (t >= 0.0 && distance <= stroke_width) {
        textureStore(output, vec2<i32>(id.xy), el_color);
    }
}
