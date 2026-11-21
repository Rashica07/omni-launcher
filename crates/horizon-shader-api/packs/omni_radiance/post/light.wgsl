// Puts the occlusion and light shafts from the half-size `rays` pass onto the picture. The shafts take the sun's colour.
override ambient_strength: f32 = 1.0;

fn post(input: PostInput) -> vec4<f32> {
    var c = input.color.rgb;
    if input.depth > 0.0 {
        // A small blur of the half-size result hides its noise.
        let o = 1.5 / frame.resolution;
        let ao = (textureSampleLevel(rays, linear_sampler, input.uv, 0.0).g * 2.0
            + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(o.x, o.y), 0.0).g
            + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(-o.x, o.y), 0.0).g
            + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(o.x, -o.y), 0.0).g
            + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(-o.x, -o.y), 0.0).g) / 6.0;
        c *= mix(1.0, ao, ambient_strength);
    }
    let o = 2.0 / frame.resolution;
    let shafts = (textureSampleLevel(rays, linear_sampler, input.uv, 0.0).r * 2.0
        + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(o.x, o.y), 0.0).r
        + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(-o.x, o.y), 0.0).r
        + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(o.x, -o.y), 0.0).r
        + textureSampleLevel(rays, linear_sampler, input.uv + vec2<f32>(-o.x, -o.y), 0.0).r) / 6.0;
    c += light_color() * shafts * 0.9;
    return vec4<f32>(c, input.color.a);
}
