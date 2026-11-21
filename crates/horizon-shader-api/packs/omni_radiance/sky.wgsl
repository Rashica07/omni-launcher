// Sky, volumetric-looking clouds and lightning. The clouds are a slab a few hundred blocks up: the ray crosses it in a
// handful of steps, each step reads a cheap fractal noise shaped by height (flat bottoms, puffy tops), and one
// extra read toward the sun gives the self-shadowing that makes them look thick.
override cloud_amount: f32 = 0.5;

const CLOUD_BASE: f32 = 230.0;
const CLOUD_TOP: f32 = 330.0;

fn fbm(p: vec2<f32>) -> f32 {
    var a = 0.5;
    var s = 0.0;
    var q = p;
    for (var i = 0; i < 4; i++) {
        s += a * vnoise(q);
        q = q * 2.03 + vec2<f32>(17.1, 9.2);
        a *= 0.5;
    }
    return s;
}

// Cloud density at a world point (0 none).
fn density(p: vec3<f32>, coverage: f32) -> f32 {
    let h = clamp((p.y - CLOUD_BASE) / (CLOUD_TOP - CLOUD_BASE), 0.0, 1.0);
    // Flat bottom, rounded top.
    let shape = smoothstep(0.0, 0.15, h) * (1.0 - smoothstep(0.55, 1.0, h));
    let wind = vec2<f32>(frame.time * 1.6, frame.time * 0.7);
    let uv = (p.xz + wind) * 0.0042;
    let n = fbm(uv) * 0.75 + vnoise(uv * 6.0 + 3.0) * 0.25;
    return clamp((n - (1.0 - coverage)) * 3.2, 0.0, 1.0) * shape;
}

// Returns (rgb colour of the cloud, alpha). `lit` is the light colour reaching the tops.
fn clouds(dir: vec3<f32>, coverage: f32, flash: f32) -> vec4<f32> {
    if dir.y < 0.015 {
        return vec4<f32>(0.0);
    }
    let cam_y = frame.camera_position.y;
    let t0 = (CLOUD_BASE - cam_y) / dir.y;
    let t1 = (CLOUD_TOP - cam_y) / dir.y;
    if t1 <= 0.0 {
        return vec4<f32>(0.0);
    }
    let start = max(t0, 0.0);
    let steps = 6;
    let dt = (t1 - start) / f32(steps);
    let origin = vec3<f32>(frame.camera_position.x, cam_y, frame.camera_position.z);
    let jitter = hash21(dir.xz * 713.0 + frame.time) * 0.0;
    let sun = frame.sun_direction;
    let day = daylight();
    var transmit = 1.0;
    var light = vec3<f32>(0.0);
    let toward = max(dot(dir, sun), 0.0);
    // Silver lining: forward scattering toward the sun.
    let phase = 0.55 + 1.6 * pow(toward, 8.0);
    let sun_col = light_color() * 1.15;
    let storm = clamp(frame.rain * 0.7 + frame.thunder * 0.5, 0.0, 1.0);
    let ambient = mix(vec3<f32>(0.04, 0.05, 0.09), mix(vec3<f32>(0.46, 0.55, 0.70), vec3<f32>(0.30, 0.33, 0.38), storm), day);
    for (var i = 0; i < steps; i++) {
        let p = origin + dir * (start + (f32(i) + 0.5 + jitter) * dt);
        let d = density(p, coverage);
        if d > 0.01 {
            // One step toward the light: how much cloud lies between this point and the sun.
            let shadow_d = density(p + sun * 38.0, coverage) + density(p + sun * 90.0, coverage) * 0.6;
            let beer = exp(-shadow_d * 2.4) * mix(1.0, 0.45, storm);
            let h = clamp((p.y - CLOUD_BASE) / (CLOUD_TOP - CLOUD_BASE), 0.0, 1.0);
            let direct = sun_col * beer * phase * day;
            let amb = ambient * (0.55 + 0.45 * h);
            var c = direct + amb;
            // Lightning lights the cloud from inside.
            c += vec3<f32>(0.75, 0.82, 1.0) * flash * 2.2;
            let a = 1.0 - exp(-d * dt * 0.045);
            light += transmit * a * c;
            transmit *= 1.0 - a;
            if transmit < 0.03 {
                break;
            }
        }
    }
    return vec4<f32>(light, 1.0 - transmit);
}

// A jagged bolt from a cloud down toward the horizon, at compass angle `az`.
fn bolt(dir: vec3<f32>, az: f32, since: f32) -> f32 {
    if dir.y < 0.03 || dir.y > 0.75 {
        return 0.0;
    }
    let ang = atan2(dir.x, dir.z);
    var da = ang - az;
    da = atan2(sin(da), cos(da));
    // The path wanders, with a few branches.
    let y = dir.y;
    let wander = (vnoise(vec2<f32>(y * 18.0, az * 5.0)) - 0.5) * 0.10 + (vnoise(vec2<f32>(y * 50.0, az)) - 0.5) * 0.035;
    let branch = (vnoise(vec2<f32>(y * 22.0 + 4.0, az * 3.0)) - 0.5) * 0.22 * smoothstep(0.35, 0.05, y);
    let core = exp(-pow(abs(da - wander) * 190.0, 2.0));
    let glow = exp(-abs(da - wander) * 22.0) * 0.25;
    let side = exp(-pow(abs(da - wander - branch) * 260.0, 2.0)) * 0.5 * step(0.12, y);
    let fade = smoothstep(0.75, 0.55, y) * smoothstep(0.03, 0.10, y);
    return (core + glow + side) * fade * exp(-since * 7.0);
}

fn sky(input: SkyInput) -> vec4<f32> {
    let d = normalize(input.direction);
    var c = sky_gradient(d);
    let ls = lightning_state();
    let flash = ls.x * frame.thunder;
    let cover = clamp(0.25 + cloud_amount * 0.55 + frame.rain * 0.35 + frame.thunder * 0.2, 0.0, 0.95);
    let cl = clouds(d, cover, flash);
    // Haze softens clouds far down the sky.
    let haze = smoothstep(0.02, 0.28, d.y);
    c = mix(c, cl.rgb, cl.a * haze);
    // Sky lit by a flash.
    c += vec3<f32>(0.45, 0.52, 0.75) * flash * 0.55;
    if ls.w > 0.5 {
        c += vec3<f32>(0.85, 0.90, 1.0) * bolt(d, ls.y, ls.z) * 2.5;
    }
    return vec4<f32>(max(c, vec3<f32>(0.0)), input.color.a);
}
