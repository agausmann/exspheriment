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

struct Conic {
    focus_size: vec4<f32>;
    e_vec: vec4<f32>;
    p_vec: vec4<f32>;
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

struct Conics {
    conics: array<Conic>;
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

[[group(1), binding(4)]]
var<storage, read> conics: Conics;

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

[[stage(compute), workgroup_size(64)]]
fn conic_main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= u32(dims.x) || id.y >= u32(dims.y)) {
        return;
    }

    let eye = viewport.eye;
    let eye_ray = eye_ray(id);

    let con_focus = conics.conics[id.z].focus_size.xyz;
    let stroke_width = conics.conics[id.z].focus_size.w;
    let con_e_vec = conics.conics[id.z].e_vec;
    let con_p_vec = conics.conics[id.z].p_vec;
    let con_color = conics.conics[id.z].color;

    let e = con_e_vec.w;
    var u = normalize(con_e_vec.xyz);

    let p = con_p_vec.w;
    let v = normalize(con_p_vec.xyz);

    let a = eye_ray;
    let w = con_focus - eye;

    let n = cross(u, v);

    // Find intersection of ray with orbital plane:
    let intersect = dot(w, n) / dot(a, n) * a - w;

    // Initial guess, using the angle between u and the intersection point:
    let su = dot(normalize(intersect), u);
    let sv = dot(normalize(intersect), v);
    var theta = atan2(sv, su);


    for (var i = 0; i < 10; i = i + 1) {
        let den = 1.0 + e * cos(theta);

        let r = p / den * (u * cos(theta) + v * sin(theta) + w);
        let dr = p / (den * den) * (u * -sin(theta) + v * (e + cos(theta)));
        let ddr = p / (den * den * den) * (
            u * -(e + e * sin(theta) * sin(theta) + cos(theta))
            + v * sin(theta) * (2.0 * e * e + e * cos(theta) - 1.0)
        );

        let g = dot(a, a) * dot(r, dr) - dot(a, r) * dot(a, dr);
        let dg = (
            dot(a, a) * (dot(dr, dr) + dot(r, ddr))
            - dot(a, dr) * dot(a, dr)
            - dot(a, r) * dot(a, ddr)
        );

        let delta = g / dg;
        theta = theta - delta;
    }
    

    let con_approach = p / (1.0 + e * cos(theta)) * (u * cos(theta) + v * sin(theta)) + con_focus;
    let t = dot(con_approach - eye, a) / dot(a, a);
    let eye_approach = a * t + eye;

    let p_eye = viewport.view_proj * vec4<f32>(eye_approach, 1.0);
    let p_con = viewport.view_proj * vec4<f32>(con_approach, 1.0);
    let n_eye = p_eye.xy / p_eye.w;
    let n_con = p_con.xy / p_con.w;

    textureStore(output, vec2<i32>(id.xy), vec4<f32>(n_con, 0.0, 1.0));

    let distance = length((n_eye - n_con) * vec2<f32>(dims) * 0.5);

    if (t >= 0.0 && distance <= stroke_width) {
        textureStore(output, vec2<i32>(id.xy), con_color);
    }
}
