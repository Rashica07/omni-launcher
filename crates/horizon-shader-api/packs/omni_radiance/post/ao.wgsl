// Rays marched through the depth buffer, at half resolution. Twelve rays fan out around each pixel's surface and count how
// many hit something nearby (ambient occlusion); eight more run toward the light and say whether a close block shades the
// point (contact shadow, for the fine shadow the shadow map is too coarse for). r: openness, g: light reaching it.
override ao_strength: f32 = 0.8;

fn project(p: vec3<f32>) -> vec3<f32> {
    let c = frame.projection * (frame.view * vec4<f32>(p, 1.0));
    if c.w <= 0.0001 {
        return vec3<f32>(-1.0);
    }
    let ndc = c.xyz / c.w;
    return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z);
}

// Whether the world has something in front of camera-relative point `s` (within `reach` blocks): 1 if so.
fn blocked(s: vec3<f32>, bias: f32, reach: f32) -> f32 {
    let q = project(s);
    if q.x < 0.0 || q.x > 1.0 || q.y < 0.0 || q.y > 1.0 {
        return 0.0;
    }
    let d = depth_at(q.xy);
    if d <= 0.0 {
        return 0.0;
    }
    let scene = length(view_position(q.xy, d));
    let gap = length(s) - scene;
    return select(0.0, 1.0 - smoothstep(reach * 0.6, reach, gap), gap > bias);
}

fn post(input: PostInput) -> vec4<f32> {
    let d = depth_at(input.uv);
    if d <= 0.0 {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    }
    let p = view_position(input.uv, d);
    let dist = length(p);
    if dist > 96.0 {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    }
    let texel = vec2<i32>(input.uv * frame.resolution);
    let surf = surface_at(texel);
    var n = surf.normal;
    if !surf.present {
        // Entities and water: the normal from the depth around the pixel.
        let px = 1.0 / frame.resolution;
        let a = view_position(input.uv + vec2<f32>(px.x, 0.0), depth_at(input.uv + vec2<f32>(px.x, 0.0))) - p;
        let b = view_position(input.uv + vec2<f32>(0.0, px.y), depth_at(input.uv + vec2<f32>(0.0, px.y))) - p;
        n = normalize(cross(b, a));
        if dot(n, p) > 0.0 {
            n = -n;
        }
    }
    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.y) < 0.99);
    let t = normalize(cross(up, n));
    let b = cross(n, t);
    let ign = fract(52.9829189 * fract(dot(input.pixel, vec2<f32>(0.06711056, 0.00583715))));
    let radius = 0.9;
    var occ = 0.0;
    for (var i = 0; i < 12; i++) {
        let u = (f32(i) + 0.5) / 12.0;
        let phi = f32(i) * 2.39996323 + ign * 6.2831853;
        let st = sqrt(u);
        let local = vec3<f32>(cos(phi) * st, sin(phi) * st, sqrt(1.0 - u));
        let dir = t * local.x + b * local.y + n * local.z;
        let s = p + n * 0.05 + dir * radius * (0.2 + 0.8 * u);
        occ += blocked(s, 0.04, radius * 1.6);
    }
    var ao = 1.0 - occ / 12.0 * 1.4;
    ao = mix(1.0, clamp(ao, 0.0, 1.0), ao_strength * (1.0 - smoothstep(50.0, 96.0, dist)));

    // Contact shadow toward the light.
    var contact = 1.0;
    let l = frame.light_direction;
    if frame.dimension == 0 && l.y > 0.02 && dist < 60.0 {
        var hit = 0.0;
        for (var i = 1; i <= 8; i++) {
            let f = f32(i) / 8.0;
            let s = p + n * 0.06 + l * (0.05 + f * f * 1.7 + ign * 0.1);
            hit = max(hit, blocked(s, 0.03, 0.5));
        }
        contact = 1.0 - hit * (1.0 - smoothstep(35.0, 60.0, dist));
    }
    return vec4<f32>(ao, contact, 0.0, 1.0);
}
