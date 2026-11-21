// Shared by every stage: noise, the sun's colour, the sky without clouds (water reflects it) and lightning.
fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// 1 while the sun is up in the Overworld, fading out around the horizon.
fn daylight() -> f32 {
    return select(0.0, smoothstep(-0.12, 0.12, frame.sun_direction.y), frame.dimension == 0);
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
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// The colour of the light on the ground: warm and strong high up, deep orange near the horizon, a faint blue moon at night.
fn light_color() -> vec3<f32> {
    let e = clamp(frame.sun_direction.y, 0.0, 1.0);
    let day = mix(vec3<f32>(1.0, 0.52, 0.22), vec3<f32>(1.0, 0.93, 0.80), smoothstep(0.0, 0.5, e));
    let night = vec3<f32>(0.20, 0.27, 0.50);
    return mix(night * 0.35, day, daylight());
}

// The sky with no clouds, sun or stars: a deep blue overhead, a pale haze at the horizon, a warm glow around
// the sun and a dark blue night. Water reflects it, the sky stage adds clouds on top.
fn sky_gradient(dir: vec3<f32>) -> vec3<f32> {
    let d = normalize(dir);
    let day = daylight();
    let up = clamp(d.y, 0.0, 1.0);
    let horizon = pow(1.0 - up, 3.0);
    let zenith = vec3<f32>(0.10, 0.30, 0.78);
    let haze = vec3<f32>(0.62, 0.78, 0.97);
    var c = mix(zenith, haze, horizon);
    let toward = max(dot(d, frame.sun_direction), 0.0);
    // Warm scattering toward the sun, strongest when it is low.
    let glow = pow(toward, 6.0) * 0.35 + pow(toward, 48.0) * 0.6;
    c += vec3<f32>(1.0, 0.62, 0.30) * glow * mix(0.35, 1.0, sun_low()) * day;
    // Sunset band along the horizon.
    c = mix(c, vec3<f32>(1.0, 0.50, 0.28) * 0.9, horizon * sun_low() * 0.55 * day * (0.35 + 0.65 * toward));
    let night = mix(vec3<f32>(0.01, 0.02, 0.06), vec3<f32>(0.04, 0.07, 0.16), horizon);
    c = mix(night, c, day);
    // Storms grey the sky.
    let grey = vec3<f32>(luminance(c)) * vec3<f32>(0.80, 0.84, 0.92);
    return mix(c, grey, clamp(frame.rain * 0.75 + frame.thunder * 0.2, 0.0, 0.9));
}

// Lightning, the same in every stage. x: the flash's strength (0 none), y: the bolt's compass angle, z: how long since the
// strike in seconds (negative before it), w: 1 while there is a bolt to draw. A strike every few seconds in a thunderstorm.
override lightning: bool = true;

fn lightning_state() -> vec4<f32> {
    if !lightning || frame.thunder < 0.05 || frame.dimension != 0 {
        return vec4<f32>(0.0, 0.0, -1.0, 0.0);
    }
    let period = 6.5;
    let slot = floor(frame.time / period);
    let t = frame.time - slot * period;
    let t0 = 0.4 + hash11(slot * 1.7) * 4.0;
    let dt = t - t0;
    // Some slots stay quiet when the storm is light.
    let on = select(0.0, 1.0, hash11(slot * 3.1 + 5.0) < 0.35 + frame.thunder * 0.65);
    var f = 0.0;
    if dt > 0.0 {
        // A hard first flash, a dip, a second return stroke, then a fade.
        f = exp(-dt * 14.0) + 0.75 * exp(-abs(dt - 0.16) * 26.0) + 0.35 * exp(-dt * 3.0);
    }
    return vec4<f32>(f * on, hash11(slot * 9.3) * 6.2831853, dt, on * select(0.0, 1.0, dt > 0.0 && dt < 0.45));
}
