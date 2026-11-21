# OMNI Radiance

A lightweight SEUS-style shader pack for the OMNI Client: ray-marched ambient occlusion and contact shadows, colour bounce,
volumetric clouds, light shafts, lightning, and water with refraction and reflections.

Written for shader API 0.2 (linear light).

Install: copy the folder (or `omni_radiance.hshader`) into `shaderpacks/` and pick it in the shader menu.
Build the file with `horizon-shader pack omni_radiance`. Checked with `horizon-shader check`; 4 to 11 texture reads per pass.
