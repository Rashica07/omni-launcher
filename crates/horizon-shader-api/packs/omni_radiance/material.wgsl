// Vanilla's own lighting, kept, with three additions: soft sun shadows from the shadow map, the light of held and mod
// lights (`colored_light`), and a mirror on water. Nothing replaces the light the game worked out, so blocks keep their
// familiar look and only gain shadow.
override shadows: bool = true;
override reflections: bool = true;
override shadow_strength: f32 = 0.85;

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
    // Held items, dropped items and mods' lights.
    c = vec4<f32>(c.rgb + input.albedo.rgb * input.tint.rgb * input.colored_light, c.a);

    if input.fluid == 1u && frame.eye_in != 1u && reflections && frame.has_reflection == 1u {
        let view = normalize(-input.position);
        let fres = 0.04 + 0.96 * pow(1.0 - clamp(dot(input.normal, view), 0.0, 1.0), 5.0);
        let ruv = clamp(input.screen + input.normal.xz * vec2<f32>(0.02, 0.04), vec2<f32>(0.002), vec2<f32>(0.998));
        let m = textureSampleLevel(reflection, reflection_sampler, ruv, 0.0).rgb;
        c = vec4<f32>(mix(c.rgb, m, clamp(fres * 0.9, 0.0, 0.8)), max(c.a, mix(c.a, 1.0, fres)));
    }
    return Surface(c);
}
