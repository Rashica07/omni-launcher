// The last pass: a soft shoulder so bright lights never clip harshly, vibrance, a gentle S-curve, a warm cast in the lights
// and a cool one in the shadows, and an optional vignette.
override vibrance: f32 = 0.5;
override vignette: bool = true;

fn post(input: PostInput) -> vec4<f32> {
    var c = max(input.color.rgb, vec3<f32>(0.0));
    // Shoulder: linear to 0.7, then a smooth roll-off toward 1.
    let hi = max(c - vec3<f32>(0.7), vec3<f32>(0.0));
    c = min(c, vec3<f32>(0.7)) + 0.3 * (1.0 - exp(-hi / 0.3));
    let l = luminance(c);
    let mx = max(c.r, max(c.g, c.b));
    let mn = min(c.r, min(c.g, c.b));
    let sat = select(0.0, (mx - mn) / mx, mx > 0.0001);
    c = mix(vec3<f32>(l), c, 1.0 + vibrance * 0.6 * (1.0 - sat));
    c = mix(c, c * c * (3.0 - 2.0 * c), 0.3);
    let lum = luminance(c);
    c = mix(c, c * vec3<f32>(0.90, 0.97, 1.10), (1.0 - smoothstep(0.0, 0.4, lum)) * 0.5);
    c = mix(c, c * vec3<f32>(1.06, 1.01, 0.92), smoothstep(0.4, 0.9, lum) * 0.5);
    if vignette {
        let aspect = frame.resolution.x / frame.resolution.y;
        let d = length((input.uv - 0.5) * vec2<f32>(aspect, 1.0)) / length(vec2<f32>(aspect, 1.0) * 0.5);
        c *= 1.0 - smoothstep(0.55, 1.15, d) * 0.28;
    }
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), input.color.a);
}
