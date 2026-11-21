// The light on every block, and the water. Sun shadows from the shadow map (soft, five taps), sky and block light,
// a little specular, wet ground in the rain; water with waves, depth colour, refraction, a mirror of the land and sky and
// foam where it meets the shore.
override water_clear: f32 = 0.6;
override shadows: bool = true;
override reflections: bool = true;

// How much of the light reaches `pos` (camera-relative) on a surface facing `n`: 1 lit, 0 in shadow.
fn sun_visibility(pos: vec3<f32>, n: vec3<f32>, sky_light: f32) -> f32 {
    let guard = smoothstep(0.25, 0.9, sky_light);
    if frame.has_shadow == 0u || !shadows {
        return guard;
    }
    let bias = frame.shadow_center_texel * 1.6 + 0.03;
    let sp = shadow_position(pos + n * bias);
    let z = sp.z - 0.0009;
    let r = 1.4 / max(frame.shadow_resolution, 1.0);
    var lit = textureSampleCompareLevel(shadow_map, shadow_sampler, sp.xy, z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, sp.xy + vec2<f32>(r, 0.3 * r), z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, sp.xy + vec2<f32>(-r, -0.3 * r), z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, sp.xy + vec2<f32>(0.3 * r, r), z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, sp.xy + vec2<f32>(-0.3 * r, -r), z);
    lit = lit / 5.0;
    // Past the map's edge there is no shadow to read: fall back to the sky light.
    let edge = smoothstep(0.82, 1.0, length(pos.xz) / max(frame.shadow_distance, 1.0));
    return mix(lit, guard, edge) * guard;
}

fn ambient_light(n: vec3<f32>, sky_light: f32) -> vec3<f32> {
    let day = daylight();
    let up = n.y * 0.5 + 0.5;
    let sky = mix(vec3<f32>(0.30, 0.38, 0.52), vec3<f32>(0.44, 0.56, 0.78), up) * mix(0.30, 1.0, day);
    let ground = vec3<f32>(0.30, 0.27, 0.24) * mix(0.18, 0.55, day);
    let night = vec3<f32>(0.035, 0.05, 0.10);
    var a = mix(ground, sky, up);
    a = mix(night, a, day * 0.85 + 0.15);
    return a * pow(sky_light, 1.6) * mix(1.0, 0.7, frame.rain);
}

fn block_light(b: f32) -> vec3<f32> {
    let k = pow(b, 3.2);
    return vec3<f32>(1.0, 0.60, 0.30) * k * 1.55 + vec3<f32>(0.10, 0.07, 0.05) * b * b;
}

// Wave height at a world xz, two scrolling octaves.
fn wave(p: vec2<f32>) -> f32 {
    let t = frame.time;
    let a = vnoise(p * 0.55 + vec2<f32>(t * 0.045, t * 0.03));
    let b = vnoise(p * 1.35 - vec2<f32>(t * 0.06, -t * 0.05));
    let c = vnoise(p * 3.4 + vec2<f32>(t * 0.10, t * 0.08));
    return a * 0.55 + b * 0.3 + c * 0.15;
}

fn wave_normal(p: vec2<f32>, strength: f32) -> vec3<f32> {
    let e = 0.12;
    let h = wave(p);
    let dx = wave(p + vec2<f32>(e, 0.0)) - h;
    let dz = wave(p + vec2<f32>(0.0, e)) - h;
    return normalize(vec3<f32>(-dx * strength / e, 1.0, -dz * strength / e));
}

fn view_depth_of(depth: f32, uv: vec2<f32>) -> f32 {
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    let v = frame.projection_inverse * ndc;
    return -(v.z / v.w);
}

fn water(input: MaterialInput) -> vec4<f32> {
    let pos = input.position;
    let view = normalize(-pos);
    let world_xz = pos.xz + frame.camera_position.xz;
    // Only the top of a body of water ripples.
    let top = select(0.0, 1.0, input.face == 1u);
    let n = normalize(mix(input.normal, wave_normal(world_xz, 0.55), top));

    let px = input.screen * frame.resolution;
    // What lies under the water: the picture before it, nudged by the ripple (refraction), unless the nudge lands on
    // something in front of the water.
    let surface_clip = frame.projection * frame.view * vec4<f32>(pos, 1.0);
    let surface_depth = surface_clip.z / surface_clip.w;
    let dim = vec2<i32>(frame.resolution) - vec2<i32>(1);
    let base = vec2<i32>(px);
    let off = vec2<i32>(n.xz * 14.0 * top);
    let t1 = clamp(base + off, vec2<i32>(0), dim);
    var tex = t1;
    if textureLoad(opaque_depth, t1, 0) > surface_depth + 0.0002 {
        tex = base;
    }
    let floor_depth = textureLoad(opaque_depth, tex, 0);
    let floor_color = textureLoad(opaque_color, tex, 0).rgb;
    // How far light travelled through the water, in blocks. Open sky below the surface counts as deep.
    var thick = 24.0;
    if floor_depth > 0.0 {
        let uv = (vec2<f32>(tex) + 0.5) / frame.resolution;
        thick = max(view_depth_of(floor_depth, uv) - view_depth_of(surface_depth, input.screen), 0.0);
    }
    // Clearer water absorbs more slowly. Red goes first, then green; blue travels furthest.
    let k = mix(1.9, 0.35, water_clear);
    let absorb = exp(-thick * k * vec3<f32>(0.55, 0.16, 0.07));
    let tone = input.albedo.rgb * input.tint.rgb;
    let day = daylight();
    let sky_l = input.light.y;
    let body = tone * vec3<f32>(0.16, 0.52, 0.62) * (0.35 + 0.65 * max(day, 0.12)) * pow(max(sky_l, 0.15), 1.5);
    var under = mix(body, floor_color * absorb + body * (1.0 - absorb) * 0.35, absorb);
    under = mix(body, under, smoothstep(0.0, 0.25, thick));

    // Fresnel: grazing angles mirror, looking straight down sees through.
    let ndv = clamp(dot(n, view), 0.0, 1.0);
    let fres = 0.02 + 0.98 * pow(1.0 - ndv, 5.0);
    let rdir = reflect(-view, n);
    var mirror = sky_gradient(vec3<f32>(rdir.x, abs(rdir.y), rdir.z)) * pow(max(sky_l, 0.2), 1.2);
    if frame.has_reflection == 1u && reflections {
        let ruv = clamp(input.screen + n.xz * vec2<f32>(0.03, 0.05) * top, vec2<f32>(0.002), vec2<f32>(0.998));
        let m = textureSampleLevel(reflection, reflection_sampler, ruv, 0.0).rgb;
        mirror = m;
    }
    // The sun's glitter.
    let l = frame.light_direction;
    let glint = pow(max(dot(rdir, l), 0.0), 380.0) * 22.0 + pow(max(dot(rdir, l), 0.0), 40.0) * 0.8;
    let sun_ok = sun_visibility(pos, vec3<f32>(0.0, 1.0, 0.0), sky_l);
    var color = mix(under, mirror, fres);
    color += light_color() * glint * sun_ok * top * (1.0 - frame.rain * 0.7);
    // Foam at the shore.
    let foam_n = vnoise(world_xz * 2.6 + vec2<f32>(frame.time * 0.3, -frame.time * 0.2));
    let foam = smoothstep(0.55, 0.0, thick) * smoothstep(0.35, 0.75, foam_n) * top * 0.75;
    color = mix(color, vec3<f32>(0.92, 0.96, 1.0) * mix(0.35, 1.0, day), foam);
    // Lightning shows in the water.
    color += vec3<f32>(0.5, 0.56, 0.8) * lightning_state().x * frame.thunder * 0.3 * (0.4 + fres);
    return vec4<f32>(color, input.albedo.a * 0.0 + 1.0);
}

fn material(input: MaterialInput) -> Surface {
    let vanilla = input.albedo * input.tint * vec4<f32>(input.lightmap, 1.0);
    // The Nether, the End and other worlds keep vanilla's light, a touch richer.
    if frame.dimension != 0 {
        return Surface(vec4<f32>(pow(vanilla.rgb, vec3<f32>(0.93)) * 1.04, vanilla.a));
    }
    if input.fluid == 1u && frame.eye_in != 1u {
        return Surface(water(input));
    }
    if input.fluid == 2u {
        // Lava glows.
        return Surface(vec4<f32>(input.albedo.rgb * input.tint.rgb * 1.9, input.albedo.a));
    }

    let n = input.normal;
    let sky_l = input.light.y;
    let day = daylight();
    let l = frame.light_direction;
    let vis = sun_visibility(input.position, n, sky_l);
    let ndl = max(dot(n, l), 0.0);
    let storm = 1.0 - frame.rain * 0.75;
    let sun_i = mix(0.30, 1.0, day);
    let flash = lightning_state().x * frame.thunder;
    // Vanilla's face shade and occlusion come in with the tint; keep half of the face shade so the blocks stay readable.
    let shade = mix(vec3<f32>(1.0), input.tint.rgb, 0.7);

    // Wet ground in and after the rain: darker, a touch glossy.
    let wet = frame.wetness * smoothstep(0.5, 1.0, n.y) * pow(sky_l, 2.0);
    var albedo = input.albedo.rgb * mix(vec3<f32>(1.0), vec3<f32>(0.72, 0.74, 0.78), wet * 0.6);

    var light = ambient_light(n, sky_l) + light_color() * ndl * vis * storm * sun_i * 1.15;
    light += vec3<f32>(0.40, 0.46, 0.72) * flash * (0.5 + 0.5 * n.y) * pow(sky_l, 1.5);
    light += block_light(input.light.x) + input.colored_light * 1.2;
    var color = albedo * shade * light + albedo * input.emission * 2.2;

    // A little shine: LabPBR smoothness where a resource pack has it, wet ground otherwise.
    var smooth_ = wet * 0.7;
    var f0 = 0.04;
    if block_pbr_maps() {
        let s = block_specular_map(input.uv);
        smooth_ = max(smooth_, s.r);
        f0 = max(f0, s.g * s.g);
    }
    if smooth_ > 0.05 {
        let view = normalize(-input.position);
        let h = normalize(view + l);
        let power = mix(8.0, 220.0, smooth_ * smooth_);
        let spec = pow(max(dot(n, h), 0.0), power) * smooth_ * (0.5 + 0.5 * power / 220.0) * 3.0;
        let fres = f0 + (1.0 - f0) * pow(1.0 - max(dot(n, view), 0.0), 5.0) * smooth_;
        color += light_color() * spec * vis * ndl * storm + sky_gradient(reflect(-view, n)) * fres * wet * 0.5;
    }

    if !input.translucent {
        set_surface(n, smooth_, 0.0);
    }
    return Surface(vec4<f32>(color, input.albedo.a * input.tint.a));
}
