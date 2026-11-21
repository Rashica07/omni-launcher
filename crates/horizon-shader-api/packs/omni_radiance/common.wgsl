fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// 1 while the sun is up in the Overworld, fading out around the horizon.
fn daylight() -> f32 {
    return select(0.0, smoothstep(-0.1, 0.1, frame.sun_direction.y), frame.dimension == 0);
}

// 1 when the sun is on the horizon, 0 from a third of the way up.
fn sun_low() -> f32 {
    return 1.0 - smoothstep(0.0, 0.35, abs(frame.sun_direction.y));
}

fn hash11(p: f32) -> f32 {
    var x = fract(p * 0.1031);
    x *= x + 33.33;
    x *= x + x;
    return fract(x);
}

// Lightning, the same in every stage: a strike every few seconds in a thunderstorm.
// x: the flash's strength, y: the bolt's compass angle, z: seconds since the strike, w: 1 while a bolt shows.
override lightning: bool = true;

fn lightning_state() -> vec4<f32> {
    if !lightning || frame.thunder < 0.05 || frame.dimension != 0 {
        return vec4<f32>(0.0, 0.0, -1.0, 0.0);
    }
    let period = 6.5;
    let slot = floor(frame.time / period);
    let dt = frame.time - slot * period - (0.4 + hash11(slot * 1.7) * 4.0);
    let on = select(0.0, 1.0, hash11(slot * 3.1 + 5.0) < 0.35 + frame.thunder * 0.65);
    var f = 0.0;
    if dt > 0.0 {
        f = exp(-dt * 14.0) + 0.75 * exp(-abs(dt - 0.16) * 26.0) + 0.35 * exp(-dt * 3.0);
    }
    return vec4<f32>(f * on, hash11(slot * 9.3) * 6.2831853, dt, on * select(0.0, 1.0, dt > 0.0 && dt < 0.45));
}

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    q += dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
}

// Smooth value noise, 0..1.
fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(i), hash21(i + vec2<f32>(1.0, 0.0)), u.x), mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

// The colour of sunlight: gold when low, near white when high, a faint blue moon at night.
fn light_color() -> vec3<f32> {
    let e = clamp(frame.sun_direction.y, 0.0, 1.0);
    let day = mix(vec3<f32>(1.0, 0.55, 0.26), vec3<f32>(1.0, 0.95, 0.85), smoothstep(0.0, 0.45, e));
    return mix(vec3<f32>(0.08, 0.10, 0.18), day, daylight());
}

// Screen position of a world-space vector or point: xy uv (0..1 from the top left), z the clip depth (0 or less: behind).
// From Vivid Horizon (MIT, Evan-lhm).
fn project(v: vec4<f32>) -> vec3<f32> {
    let clip = frame.projection * frame.view * v;
    let ndc = clip.xy / clip.w;
    return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, clip.w);
}

// An integer hash of a pixel, 0..1 (Vivid Horizon, MIT, Evan-lhm).
fn hash2(p: vec2<f32>) -> f32 {
    var h = bitcast<u32>(vec2<i32>(p).x) * 1597334673u ^ bitcast<u32>(vec2<i32>(p).y) * 3812015801u;
    h = (h ^ (h >> 16u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    h ^= h >> 16u;
    return f32(h) / 4294967295.0;
}

// Linear light <-> the gamma space the water's colours were tuned in (Vivid Horizon's water, MIT, Evan-lhm).
fn to_gamma(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
}

fn fog_g() -> vec3<f32> {
    return to_gamma(frame.fog_color);
}

fn sky_g() -> vec3<f32> {
    return to_gamma(frame.sky_color);
}
