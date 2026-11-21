// OMNI Glow's sky (a deeper blue overhead, a warm band at the horizon), kept on top of the game's own sky and clouds,
// plus a lightning flash and bolt in thunderstorms.
override sky_depth: f32 = 0.6;

fn bolt(d: vec3<f32>, az: f32, since: f32) -> f32 {
    let a = atan2(d.x, d.z);
    var diff = abs(a - (az - 3.14159));
    diff = min(diff, 6.2831853 - diff);
    let wob = sin(d.y * 38.0 + az * 7.0) * 0.012 + sin(d.y * 91.0 + az) * 0.006;
    let w = abs(diff + wob) * (1.0 - d.y * 0.5);
    let core = exp(-w * 260.0);
    let glow = exp(-w * 28.0) * 0.25;
    return (core + glow) * smoothstep(0.02, 0.15, d.y) * (1.0 - smoothstep(0.55, 0.95, d.y)) * exp(-since * 7.0);
}

override cloud_amount: f32 = 0.55;

fn fbm(p: vec2<f32>) -> f32 {
    var a = 0.5;
    var s = 0.0;
    var q = p;
    for (var i = 0; i < 5; i++) {
        s += a * vnoise(q);
        q = q * 2.07 + vec2<f32>(17.1, 9.2);
        a *= 0.5;
    }
    return s;
}

// Cloud density at a point of the layer, 0 clear.
fn cloud_density(p: vec2<f32>, coverage: f32) -> f32 {
    let q = (p + vec2<f32>(frame.time * 1.8, frame.time * 0.8)) * 0.0030;
    let n = fbm(q) * 0.8 + vnoise(q * 9.0) * 0.2;
    return clamp((n - (1.0 - coverage)) * 7.0, 0.0, 1.0);
}

// Clouds: a layer 260 blocks up, fluffy edges, lit from the sun's side, dark where thick, silver along the rim.
fn cloud_layer(d: vec3<f32>, base: vec3<f32>, flash: f32) -> vec3<f32> {
    if d.y < 0.012 {
        return base;
    }
    let storm = clamp(frame.rain * 0.7 + frame.thunder * 0.5, 0.0, 1.0);
    let coverage = clamp(0.22 + cloud_amount * 0.45 + frame.rain * 0.3, 0.0, 0.95);
    let t = (260.0 - frame.camera_position.y) / d.y;
    if t <= 0.0 {
        return base;
    }
    let p = (frame.camera_position + d * t).xz;
    let dens = cloud_density(p, coverage);
    if dens <= 0.0 {
        return base;
    }
    let day = daylight();
    let sun = frame.sun_direction;
    // The same cloud a little toward the sun: less there means we're on the lit rim, more means we're in shade.
    let toward = normalize(vec2<f32>(sun.x, sun.z) + vec2<f32>(0.0001)) * 55.0;
    let ahead = cloud_density(p + toward, coverage);
    let rim = clamp((dens - ahead) * 1.8 + 0.55, 0.0, 1.0);
    let thick = smoothstep(0.1, 1.0, dens);
    let cos_s = max(dot(d, sun), 0.0);
    let silver = pow(cos_s, 10.0) * (1.0 - thick * 0.6);
    let lit = light_color() * 1.25 * mix(0.25, 1.0, rim) * mix(1.0, 0.45, storm);
    let shade = mix(vec3<f32>(0.05, 0.06, 0.1), mix(vec3<f32>(0.62, 0.72, 0.88), vec3<f32>(0.30, 0.33, 0.38), storm), day);
    var c = mix(shade * (1.0 - 0.45 * thick), lit, rim * day * (1.0 - 0.5 * thick)) + light_color() * silver * 0.9 * day;
    c += vec3<f32>(0.7, 0.78, 1.0) * flash * (0.8 + 0.6 * thick);
    // Far clouds melt into the horizon haze.
    let fade = smoothstep(0.02, 0.22, d.y);
    return mix(base, c, clamp(dens * 1.2, 0.0, 1.0) * fade);
}

fn sky(input: SkyInput) -> vec4<f32> {
    let d = input.direction;
    let day = daylight() * (1.0 - frame.rain * 0.8);
    let up = smoothstep(0.05, 1.0, d.y);
    let near_horizon = 1.0 - smoothstep(0.0, 0.35, abs(d.y));
    let toward = max(dot(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0001)), normalize(vec3<f32>(frame.sun_direction.x, 0.0, frame.sun_direction.z) + vec3<f32>(0.0001))), 0.0);
    var c = mix(input.color.rgb, input.color.rgb * vec3<f32>(0.62, 0.80, 1.25), up * day * sky_depth);
    let band = near_horizon * (0.25 + 0.75 * pow(toward, 3.0)) * mix(0.25, 1.0, sun_low()) * day * sky_depth;
    c += vec3<f32>(1.0, 0.70, 0.40) * band * 0.45;
    let ls = lightning_state();
    let dn = normalize(d);
    // The sun: a bright disc with a soft corona, hidden behind thick cloud by the cloud layer below.
    let cs = max(dot(dn, frame.sun_direction), 0.0);
    let disc = smoothstep(0.99935, 0.9997, cs);
    c += light_color() * (disc * 4.0 + pow(cs, 90.0) * 0.8 + pow(cs, 14.0) * 0.18) * daylight() * (1.0 - frame.rain * 0.9);
    c = cloud_layer(dn, c, ls.x * frame.thunder);
    c += vec3<f32>(0.35, 0.42, 0.7) * ls.x * frame.thunder * 0.35;
    if ls.w > 0.5 {
        c += vec3<f32>(0.8, 0.85, 1.0) * bolt(normalize(d), ls.y, ls.z) * 2.0;
    }
    return vec4<f32>(c, input.color.a);
}
