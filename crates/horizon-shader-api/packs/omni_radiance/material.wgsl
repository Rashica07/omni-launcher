// Vanilla's own lighting, kept, with three additions: soft sun shadows from the shadow map, the light of held and mod
// lights (`colored_light`), and a mirror on water. Nothing replaces the light the game worked out, so blocks keep their
// familiar look and only gain shadow.
override shadows: bool = true;
override reflections: bool = true;
override shadow_strength: f32 = 0.85;
override water_shine: f32 = 0.8;

fn sun_visibility(pos: vec3<f32>, n: vec3<f32>) -> f32 {
    if frame.has_shadow == 0u || !shadows {
        return 1.0;
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
    let edge = smoothstep(0.82, 1.0, length(pos.xz) / max(frame.shadow_distance, 1.0));
    return mix(lit, 1.0, edge);
}


// ---- Water (from Vivid Horizon, MIT, Evan-lhm; adapted to OMNI Radiance's sky) ----

// The water's surface height: three layers of noise drifting in different directions.
fn water_height(p: vec2<f32>, detail: f32) -> f32 {
    let t = frame.time;
    let ra = mat2x2<f32>(0.93, 0.37, -0.37, 0.93);
    let rb = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
    let rc = mat2x2<f32>(0.28, 0.96, -0.96, 0.28);
    return vnoise(ra * p * 0.9 + vec2<f32>(t * 0.35, t * 0.2)) * 0.55
        + vnoise(rb * p * 2.3 + vec2<f32>(-t * 0.5, t * 0.4)) * 0.3 * mix(0.4, 1.0, detail)
        + vnoise(rc * p * 5.1 + vec2<f32>(t * 0.7, -t * 0.6)) * 0.15 * detail;
}

// Ripples: the slope of water_height; small ripples fade with distance and at low angles, where they would only shimmer.
fn water_normal(world: vec2<f32>, dist: f32, facing: f32) -> vec3<f32> {
    let e = 0.08;
    let foot = dist / max(facing, 0.08);
    let detail = 1.0 - smoothstep(8.0, 40.0, foot);
    let h = water_height(world, detail);
    let dx = water_height(world + vec2<f32>(e, 0.0), detail) - h;
    let dz = water_height(world + vec2<f32>(0.0, e), detail) - h;
    let strength = 0.12 * (1.0 - smoothstep(12.0, 48.0, dist)) * (1.0 - 0.7 * smoothstep(30.0, 120.0, foot));
    return normalize(vec3<f32>(-dx / e * strength, 1.0, -dz / e * strength));
}

// Camera-relative distance to what lies behind the water at `screen`.
fn behind_distance(screen: vec2<f32>) -> f32 {
    let texel = vec2<i32>(clamp(screen, vec2<f32>(0.0), vec2<f32>(0.9999)) * frame.resolution);
    let depth = textureLoad(opaque_depth, texel, 0);
    if depth <= 0.0 {
        return 1e4;
    }
    let ndc = vec4<f32>(screen.x * 2.0 - 1.0, 1.0 - screen.y * 2.0, depth, 1.0);
    let v = frame.projection_inverse * ndc;
    return length(v.xyz / v.w);
}

// Raindrop rings, as a tilt to add to the normal.
fn rain_rings(p: vec2<f32>) -> vec2<f32> {
    let cell = floor(p);
    let seed = hash2(cell);
    let phase = frame.time * (2.2 + frame.thunder * 0.8) + seed * 7.0;
    let drop = floor(phase);
    let age = fract(phase);
    let falls = step(hash2(cell + vec2<f32>(drop * 3.0, 17.0)), 0.3 + 0.5 * frame.rain + 0.2 * frame.thunder);
    let spot = vec2<f32>(hash2(cell + vec2<f32>(drop * 13.0, 7.0)), hash2(cell + vec2<f32>(5.0, drop * 11.0)));
    let d = fract(p) - (0.3 + 0.4 * spot);
    let r = length(d);
    let front = r - age * 0.3;
    let wave = sin(front * 70.0) * (1.0 - smoothstep(0.0, 0.05, abs(front))) * (1.0 - age);
    return d / max(r, 1e-3) * wave * falls;
}

// Screen-space reflection for water the mirror does not cover: rgb the colour hit, a how sure it is.
fn screen_reflection(p: vec3<f32>, r: vec3<f32>) -> vec4<f32> {
    var t = 0.4;
    for (var i = 0; i < 24; i++) {
        let q = p + r * t;
        let s = project(vec4<f32>(q, 1.0));
        if s.z <= 0.0 {
            break;
        }
        let uv = s.xy;
        if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
            break;
        }
        let scene = behind_distance(uv);
        let dq = length(q);
        if dq > scene && dq < scene + 0.6 + t * 0.15 {
            let edge = smoothstep(0.0, 0.08, min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y)));
            return vec4<f32>(to_gamma(textureLoad(opaque_color, vec2<i32>(uv * frame.resolution), 0).rgb), edge);
        }
        t = t * 1.18 + 0.15;
    }
    return vec4<f32>(0.0);
}

// The sky the water mirrors, in gamma: the game's colours deepened overhead like the sky stage, a warm band by the sun, a hint of cloud.
fn water_sky(dir_in: vec3<f32>) -> vec3<f32> {
    let d = normalize(dir_in);
    let up = clamp(d.y, 0.0, 1.0);
    let day = daylight();
    var c = mix(fog_g(), sky_g() * vec3<f32>(0.9, 1.0, 1.15), pow(up, 0.45));
    let cs = max(dot(d, frame.sun_direction), 0.0);
    c += to_gamma(light_color()) * (pow(cs, 24.0) * 0.5 + pow(cs, 220.0) * 1.6) * day * (1.0 - frame.rain * 0.9);
    let cl = smoothstep(0.5, 0.8, vnoise(d.xz / (d.y + 0.08) * 1.3 + vec2<f32>(frame.time * 0.01, 0.0)));
    c = mix(c, vec3<f32>(luminance(c) * 1.2 + 0.1), cl * 0.35 * smoothstep(0.02, 0.2, d.y));
    return c;
}

fn rainy_biome() -> f32 {
    return smoothstep(0.0, 0.1, frame.downfall) * smoothstep(0.1, 0.2, frame.temperature);
}

fn water(input: MaterialInput, sun: f32) -> vec4<f32> {
    let dist = length(input.position);
    let view = -input.position / dist;
    let world = (input.position.xz + frame.camera_position.xz) % vec2<f32>(4096.0);
    var n = select(input.normal, water_normal(world, dist, abs(view.y)), input.face == 1u);
    var ring = 0.0;
    let drops = frame.rain * rainy_biome() * smoothstep(0.9, 0.97, input.light.y) * (1.0 - smoothstep(32.0, 80.0, dist));
    if input.face == 1u && drops > 0.0 {
        let tilt = rain_rings(world * 2.2) + rain_rings(world * 2.2 + vec2<f32>(0.5, 0.37));
        n = normalize(n + vec3<f32>(tilt.x, 0.0, tilt.y) * 0.35 * drops);
        ring = min(length(tilt), 1.0) * drops;
    }
    let light = luminance(to_gamma(input.lightmap));

    // Under the surface: the bottom seen through the waves, fading with depth into turquoise.
    var refract_uv = clamp(input.screen + n.xz * 0.015, vec2<f32>(0.0), vec2<f32>(0.9999));
    var deep = behind_distance(refract_uv) - dist;
    if deep < 0.0 {
        refract_uv = input.screen;
        deep = max(behind_distance(refract_uv) - dist, 0.0);
    }
    deep = min(deep, select(1.2, 64.0, input.face == 1u));
    let bottom = to_gamma(textureLoad(opaque_color, vec2<i32>(refract_uv * frame.resolution), 0).rgb);
    let absorb = exp(-vec3<f32>(0.6, 0.24, 0.17) * deep);
    let scatter = vec3<f32>(0.05, 0.21, 0.25) * light * mix(0.6, 1.0, sun);
    let murk = mix(0.3, 1.0, 1.0 - exp(-deep * 0.6));
    let under = bottom * absorb * (1.0 - murk * 0.5) + scatter * murk;

    // Above: our sky, with the world's mirror where it shows terrain.
    let r = reflect(-view, n);
    var above = water_sky(vec3<f32>(r.x, abs(r.y), r.z)) * mix(0.25, 1.0, input.light.y);
    let on_plane = abs(input.position.y + frame.camera_position.y - frame.water_plane) < 0.6;
    if reflections && frame.has_reflection == 1u && input.face == 1u && on_plane {
        let bend = 0.04 * mix(0.3, 1.0, smoothstep(0.1, 0.5, abs(view.y)));
        let uv = clamp(input.screen + n.xz * bend, vec2<f32>(0.001), vec2<f32>(0.999));
        let mirror = to_gamma(textureSampleLevel(reflection, reflection_sampler, uv, 0.0).rgb);
        // The mirror draws vanilla's sky; where it matches that, keep ours.
        let vanilla_sky = mix(fog_g(), sky_g(), smoothstep(0.0, 0.4, abs(r.y)));
        let terrain = smoothstep(0.04, 0.15, length(mirror - vanilla_sky));
        above = mix(above, mirror, terrain);
    } else if reflections && input.face == 1u && dist < 64.0 {
        let hit = screen_reflection(input.position, r);
        above = mix(above, hit.rgb, hit.a * (1.0 - smoothstep(40.0, 64.0, dist)));
    }

    // Schlick's Fresnel: clear turquoise looking down, a sky sheen near grazing angles.
    let fresnel = 0.02 + 0.98 * pow(1.0 - max(dot(n, view), 0.0), 5.0);
    var c = mix(under, above, clamp(fresnel * (0.2 + water_shine), 0.0, 1.0));
    // The sun's glint on the waves.
    let h = normalize(view + frame.sun_direction);
    let glint = pow(max(dot(n, h), 0.0), 300.0) * daylight() * sun * input.light.y * water_shine * (1.0 - frame.rain);
    c += to_gamma(light_color()) * glint * 4.0;
    if input.face == 1u {
        let shallow = 1.0 - smoothstep(0.0, 3.0, deep);
        let lit = light * mix(0.15, 1.0, daylight());
        c = mix(c, vec3<f32>(0.2, 0.7, 0.66) * lit, 0.3 * shallow);
        let lace = smoothstep(0.55, 0.8, vnoise(world * 2.6 + vec2<f32>(frame.time * 0.25, 0.0)));
        c = mix(c, vec3<f32>(0.9, 0.97, 1.0) * lit, 0.5 * lace * (1.0 - smoothstep(0.0, 0.9, deep)));
        c += fog_g() * (0.3 + 0.7 * lit) * ring * 0.35;
    } else {
        // Falling water: the flow texture in the biome's water colour with streaks sliding down.
        let w = input.position + frame.camera_position;
        let across = (w.x + w.z) % 4096.0;
        let fall = vec2<f32>(across * 2.5, w.y * 0.35 + frame.time * 1.6);
        let streak = vnoise(fall) * 0.6 + vnoise(fall * vec2<f32>(2.7, 2.0) + 3.1) * 0.4;
        let tex = luminance(to_gamma(input.albedo.rgb));
        let hue = mix(to_gamma(input.tint.rgb), vec3<f32>(0.35, 0.75, 0.85), 0.45);
        let body = hue * (0.5 + 0.8 * tex) * (0.15 + 0.85 * light);
        c = mix(c, body, 0.35 + 0.2 * streak);
        c += vec3<f32>(0.8, 0.95, 1.0) * (0.15 + 0.85 * light) * smoothstep(0.6, 0.85, streak) * 0.25;
    }
    return vec4<f32>(to_linear(c), 1.0);
}

// The surface seen from below: the world above through a bright window, total internal reflection beyond it.
fn water_from_below(input: MaterialInput) -> vec4<f32> {
    let dist = length(input.position);
    let view = -input.position / dist;
    let world = (input.position.xz + frame.camera_position.xz) % vec2<f32>(4096.0);
    let n = water_normal(world, dist, abs(view.y));
    let cos_i = clamp(dot(n, -view), 0.0, 1.0);
    let window = smoothstep(0.5, 0.72, cos_i);
    let uv = clamp(input.screen + n.xz * 0.03, vec2<f32>(0.0), vec2<f32>(0.9999));
    let above = to_gamma(textureLoad(opaque_color, vec2<i32>(uv * frame.resolution), 0).rgb);
    let light = mix(0.15, 1.0, input.light.y) * mix(0.25, 1.0, daylight());
    let depths = vec3<f32>(0.04, 0.22, 0.32) * light;
    let see = max(window, 0.4);
    var c = mix(depths, above * vec3<f32>(0.8, 1.0, 1.05) + depths * 0.3, see);
    c += vec3<f32>(0.5, 0.8, 0.9) * light * (window * (1.0 - window)) * 0.8;
    return vec4<f32>(to_linear(c), 1.0);
}

fn material(input: MaterialInput) -> Surface {
    var c = input.albedo * input.tint * vec4<f32>(input.lightmap, 1.0);
    if frame.dimension == 0 && input.fluid != 2u {
        // Sun and sky as two lights: the sun's warm, only where the shadow map and the face's angle let it in; the
        // shade is the sky's cool blue. Blocks keep vanilla's own light underneath (cave and torch light untouched).
        let sun_share = smoothstep(0.45, 0.95, input.light.y) * daylight() * (1.0 - frame.rain * 0.7);
        let ndl = dot(input.normal, frame.light_direction);
        let vis = sun_visibility(input.position, input.normal) * smoothstep(-0.05, 0.3, ndl);
        let warm = mix(vec3<f32>(1.16, 1.02, 0.84), vec3<f32>(1.10, 1.05, 0.95), 1.0 - sun_low());
        let cool = mix(vec3<f32>(1.0), vec3<f32>(0.46, 0.56, 0.78), shadow_strength);
        let lit = mix(cool, warm, vis);
        c = vec4<f32>(c.rgb * mix(vec3<f32>(1.0), lit, sun_share), c.a);
    }
    // Block light takes the colour of its source: warm near torches, cool by sea lanterns, red by redstone ([[lights]] in
    // the manifest). Tints vanilla's light instead of adding to it, so nothing burns white (after Vivid Horizon, MIT).
    if input.fluid == 0u {
        let near = block_light_color(input.position);
        let lamp = mix(vec3<f32>(1.0, 0.55, 0.22), near.rgb * 4.0, min(near.a, 1.0));
        let hue = lamp / max(max(lamp.r, lamp.g), max(lamp.b, 0.001));
        let day_l = daylight();
        let fade = 1.0 - day_l * input.light.y * 0.7;
        let tint = max(pow(input.light.x, 2.0) * fade, min(near.a, 1.0) * input.light.x * 0.6 * fade);
        let surface = input.albedo.rgb * input.tint.rgb;
        c = vec4<f32>(c.rgb * mix(vec3<f32>(1.0), hue * 1.15, tint * 0.75) + surface * hue * tint * 0.08, c.a);
    }
    // Held items, dropped items and mods' lights.
    c = vec4<f32>(c.rgb + input.albedo.rgb * input.tint.rgb * input.colored_light, c.a);

    if input.fluid == 1u {
        let sun_w = smoothstep(0.45, 0.95, input.light.y);
        if frame.eye_in == 0u {
            return Surface(water(input, sun_w));
        }
        if frame.eye_in == 1u && input.face == 1u {
            return Surface(water_from_below(input));
        }
        if frame.eye_in == 1u {
            return Surface(water(input, sun_w));
        }
    }
    return Surface(c);
}
