// OMNI Glow's grade (soft shoulder, vibrance, split tone, vignette), a little gentler, with the lightning flash
// brightening the whole picture for a moment.
override vibrance: f32 = 0.5;
override warmth: f32 = 0.4;
override vignette: bool = true;

fn post(input: PostInput) -> vec4<f32> {
    var c = input.color.rgb;
    let flash = lightning_state().x * frame.thunder;
    c = c * (1.0 + flash * 0.4) + vec3<f32>(0.2, 0.24, 0.4) * flash * 0.2;
    // Shoulder, then held inside 0..1: the S-curve below turns values above 1 negative, which painted the sun blue.
    c = clamp(c / (1.0 + 0.2 * c) * 1.2, vec3<f32>(0.0), vec3<f32>(1.0));
    let l = luminance(c);
    let mx = max(c.r, max(c.g, c.b));
    let mn = min(c.r, min(c.g, c.b));
    let sat = select(0.0, (mx - mn) / mx, mx > 0.0001);
    c = mix(vec3<f32>(l), c, 1.0 + vibrance * (1.0 - sat));
    c = mix(c, c * c * (3.0 - 2.0 * c), 0.3);
    let lum = luminance(c);
    c = mix(c, c * vec3<f32>(0.90, 0.97, 1.10), (1.0 - smoothstep(0.0, 0.45, lum)) * warmth);
    c = mix(c, c * vec3<f32>(1.06, 1.01, 0.92), smoothstep(0.35, 0.9, lum) * warmth);
    if vignette {
        let aspect = frame.resolution.x / frame.resolution.y;
        let d = length((input.uv - 0.5) * vec2<f32>(aspect, 1.0)) / length(vec2<f32>(aspect, 1.0) * 0.5);
        c *= 1.0 - smoothstep(0.5, 1.1, d) * 0.3;
    }
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), input.color.a);
}
