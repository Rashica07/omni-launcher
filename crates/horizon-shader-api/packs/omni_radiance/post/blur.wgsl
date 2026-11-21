// A wide, soft copy of the picture at quarter size: what the light around a pixel looks like. The compose pass reads it twice:
// as bloom (the bright parts) and as colour bounce (the hue of the neighbourhood).
fn post(input: PostInput) -> vec4<f32> {
    let step = frame.resolution.y / 1080.0 / frame.resolution;
    var sum = textureSampleLevel(previous, linear_sampler, input.uv, 0.0).rgb * 2.0;
    var w = 2.0;
    for (var i = 0; i < 8; i++) {
        let a = f32(i) * 0.7853982 + 0.3;
        let r = select(7.0, 19.0, (i & 1) == 1);
        let o = vec2<f32>(cos(a), sin(a)) * r * step;
        sum += textureSampleLevel(previous, linear_sampler, input.uv + o, 0.0).rgb;
        w += 1.0;
    }
    return vec4<f32>(sum / w, 1.0);
}
