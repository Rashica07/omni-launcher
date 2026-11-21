// Puts the light together: occlusion and contact shadows over the picture, colour bounce, bloom, light shafts, water and
// lightning on the whole picture.
override bounce: f32 = 0.5;
override god_rays: f32 = 0.5;
override bloom: f32 = 0.4;

fn open_light(uv: vec2<f32>) -> vec2<f32> {
    // A small blur of the half-resolution result hides its noise.
    let o = 1.5 / frame.resolution;
    var s = textureSampleLevel(ao, linear_sampler, uv, 0.0).rg * 2.0;
    s += textureSampleLevel(ao, linear_sampler, uv + vec2<f32>(o.x, o.y), 0.0).rg;
    s += textureSampleLevel(ao, linear_sampler, uv + vec2<f32>(-o.x, o.y), 0.0).rg;
    s += textureSampleLevel(ao, linear_sampler, uv + vec2<f32>(o.x, -o.y), 0.0).rg;
    s += textureSampleLevel(ao, linear_sampler, uv + vec2<f32>(-o.x, -o.y), 0.0).rg;
    return s / 6.0;
}

fn post(input: PostInput) -> vec4<f32> {
    var c = enc(input.color.rgb);
    let depth = input.depth;
    let day = daylight();
    let flash = lightning_state().x * frame.thunder;

    if depth > 0.0 {
        let ol = open_light(input.uv);
        // Occlusion and contact shadow darken; the sun's share of the light is the part contact shadows take.
        let shade = ol.x * mix(1.0, ol.y, 0.55 * max(day, 0.2));
        c *= shade;
        // Colour bounce: the hue of what is near, without making the picture brighter.
        let b = enc(textureSampleLevel(blur, linear_sampler, input.uv, 0.0).rgb);
        let chroma = b - vec3<f32>(luminance(b));
        c += c * chroma * bounce * 1.6 * (0.5 + 0.5 * (1.0 - ol.x));
    }

    // Light shafts: march toward the sun on the screen and count the sky that shows through.
    if god_rays > 0.0 && frame.dimension == 0 && frame.sun_direction.y > -0.05 {
        let sc = frame.projection * (frame.view * vec4<f32>(frame.sun_direction * 100.0, 1.0));
        if sc.w > 0.0 {
            let sun_uv = vec2<f32>(sc.x / sc.w * 0.5 + 0.5, 0.5 - sc.y / sc.w * 0.5);
            let dir = (sun_uv - input.uv);
            let jitter = fract(52.9829189 * fract(dot(input.pixel, vec2<f32>(0.06711056, 0.00583715))));
            var acc = 0.0;
            for (var i = 0; i < 14; i++) {
                let f = (f32(i) + jitter) / 14.0;
                let q = input.uv + dir * f * 0.85;
                if q.x > 0.0 && q.x < 1.0 && q.y > 0.0 && q.y < 1.0 {
                    acc += select(0.0, 1.0, depth_at(q) <= 0.0) * (1.0 - f * 0.55);
                }
            }
            acc = acc / 14.0;
            let closeness = 1.0 - smoothstep(0.0, 1.6, length(dir));
            let amount = acc * closeness * god_rays * smoothstep(-0.05, 0.25, frame.sun_direction.y) * (1.0 - frame.rain * 0.85);
            c += light_color() * amount * 0.5;
        }
    }

    // Bloom from the soft copy.
    if bloom > 0.0 {
        let b = enc(textureSampleLevel(blur, linear_sampler, input.uv, 0.0).rgb);
        c += b * smoothstep(0.55, 1.15, luminance(b)) * bloom * 1.1;
    }

    // Under water: blue, murky, darker with depth.
    if frame.in_water > 0.0 {
        let tint = vec3<f32>(0.22, 0.58, 0.78) * mix(0.35, 1.0, max(frame.eye_light_smooth.y, 0.2));
        let mist = clamp(1.0 - exp(-length(view_position(input.uv, max(depth, 0.00001))) * 0.07), 0.0, 1.0);
        c = mix(c * vec3<f32>(0.65, 0.92, 1.0), tint, mist * 0.85 * frame.in_water);
    }

    // Lightning lights the whole picture for a moment.
    c = c * (1.0 + flash * 0.45) + vec3<f32>(0.30, 0.34, 0.50) * flash * 0.25;
    return vec4<f32>(lin(c), 1.0);
}
