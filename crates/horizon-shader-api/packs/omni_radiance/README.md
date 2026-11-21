# OMNI Radiance

Built on OMNI Glow, for shader API 0.2 (linear light). Vanilla light with soft sun shadows (warm sun, cool shade),
layered clouds and a sun with glow, thunderstorm lightning, water that mirrors the sky, ambient occlusion and light
shafts, soft bloom and a clean grade.

Ambient occlusion and light shafts (`post/rays.wgsl`, helpers in `common.wgsl`) are adapted from
[Vivid Horizon](https://github.com/Evan-lhm/vivid-horizon) by Evan-lhm, MIT licence; see `LICENSE`.

Install: copy the folder (or `omni_radiance.hshader`) into `shaderpacks/` and pick it in the shader menu.
Build the file with `horizon-shader pack omni_radiance`; check it with `horizon-shader check omni_radiance`.
