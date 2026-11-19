"""OMNI Launcher opening, rendered headless:
/Applications/Blender.app/Contents/MacOS/Blender -b -P omni_intro.py -- <out dir> [frames]

Our own scene, nothing from Mojang: flat-coloured voxel cubes (made here, no textures) fly in
from the dark and lock into the OMNI ring (red, cream, one gold dot), then the camera pushes in.
Brand: charcoal #1A1A1D, cream #E8DCC8, red #C83246, gold #E8B44F.
"""
import bpy, math, random, sys

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
OUT = argv[0] if argv else "/tmp/omni-intro"
FRAMES = int(argv[1]) if len(argv) > 1 else 84
FPS = 24
random.seed(7)

def hexcol(h, a=1.0):
    h = h.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))
    f = lambda c: ((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92
    return (f(r), f(g), f(b), a)

CHARCOAL, CREAM, RED, GOLD = map(hexcol, ("#1A1A1D", "#E8DCC8", "#C83246", "#E8B44F"))

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 960, 540
scene.render.fps = FPS
scene.frame_start, scene.frame_end = 1, FRAMES
scene.render.image_settings.file_format = "PNG"
scene.render.filepath = OUT.rstrip("/") + "/f_"
try:
    scene.eevee.taa_render_samples = 48
except Exception:
    pass
try:
    scene.view_settings.view_transform = "Standard"
except Exception:
    pass

world = bpy.data.worlds.new("w")
world.use_nodes = True
bg = world.node_tree.nodes["Background"]
bg.inputs[0].default_value = CHARCOAL
bg.inputs[1].default_value = 1.0
scene.world = world

def material(name, color, rough=0.45, emit=0.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    p = m.node_tree.nodes["Principled BSDF"]
    p.inputs["Base Color"].default_value = color
    p.inputs["Roughness"].default_value = rough
    if "Emission Color" in p.inputs:
        p.inputs["Emission Color"].default_value = color
        p.inputs["Emission Strength"].default_value = emit
    return m

mats = {
    "red": material("red", RED, 0.4, 0.25),
    "cream": material("cream", CREAM, 0.55, 0.04),
    "gold": material("gold", GOLD, 0.3, 0.9),
}

def ease(t):  # ease-out quint
    return 1 - (1 - t) ** 5

def key(obj, frame, **vals):
    for attr, v in vals.items():
        setattr(obj, attr, v)
        obj.keyframe_insert(attr, frame=frame)

# the ring: cubes on a circle facing the camera (XZ plane)
N, R, SIZE = 48, 1.25, 0.162
ring = bpy.data.objects.new("ring", None)
scene.collection.objects.link(ring)
cubes = []
for layer in (-1, 0, 1):
  for i in range(N):
    theta = 2 * math.pi * i / N
    # red on the left half, cream on the right, one gold cube at the upper right
    deg = math.degrees(theta) % 360
    kind = "gold" if (i == 6 and layer == 0) else ("red" if 100 <= deg <= 260 else "cream")
    bpy.ops.mesh.primitive_cube_add(size=SIZE)
    c = bpy.context.active_object
    c.name = f"cube{layer}_{i}"
    c.data.materials.append(mats[kind])
    c.parent = ring
    bevel = c.modifiers.new("bevel", "BEVEL")
    bevel.width = 0.012
    bevel.segments = 2
    end = (R * math.cos(theta), layer * SIZE, R * math.sin(theta))
    rot_end = (0.0, -theta, 0.0)
    far = 3.5 + random.random() * 4.5
    ang = random.random() * math.tau
    start = (far * math.cos(ang), random.uniform(-4, 3), far * math.sin(ang))
    rot_start = tuple(random.uniform(-3, 3) for _ in range(3))
    f0 = 3 + (i + (layer + 1) * 5) * 0.34
    f1 = f0 + 24
    s_big = 1.6 if kind == "gold" else 1.0
    key(c, 1, location=start, rotation_euler=rot_start, scale=(0.01, 0.01, 0.01))
    key(c, f0, location=start, rotation_euler=rot_start, scale=(0.01, 0.01, 0.01))
    for k in range(1, 7):
        t = k / 6
        e = ease(t)
        key(c, f0 + (f1 - f0) * t,
            location=tuple(s + (d - s) * e for s, d in zip(start, end)),
            rotation_euler=tuple(s + (d - s) * e for s, d in zip(rot_start, rot_end)),
            scale=(s_big * min(1, 0.01 + e),) * 3)
    cubes.append(c)

# a slow turn of the whole ring once it is built, a small settle
key(ring, 1, rotation_euler=(0, 0, 0))
key(ring, int(FRAMES * 0.45), rotation_euler=(0, 0, 0))
key(ring, FRAMES, rotation_euler=(0, math.radians(-40), 0))

# camera: pushes in, with a small drift
target = bpy.data.objects.new("target", None)
scene.collection.objects.link(target)
target.location = (0, 0, -0.5)  # the ring sits high in frame; the name goes under it
cam_data = bpy.data.cameras.new("cam")
cam_data.lens = 35
cam = bpy.data.objects.new("cam", cam_data)
scene.collection.objects.link(cam)
scene.camera = cam
tt = cam.constraints.new("TRACK_TO")
tt.target, tt.track_axis, tt.up_axis = target, "TRACK_NEGATIVE_Z", "UP_Y"
key(cam, 1, location=(1.6, -11.0, 1.0))
key(cam, FRAMES, location=(0.25, -7.8, -0.3))

# lights: a soft key from the front-top, a red rim from behind, a cream fill
def area(name, loc, energy, color, size):
    d = bpy.data.lights.new(name, "AREA")
    d.energy, d.color, d.size = energy, color[:3], size
    o = bpy.data.objects.new(name, d)
    o.location = loc
    scene.collection.objects.link(o)
    c = o.constraints.new("TRACK_TO")
    c.target, c.track_axis, c.up_axis = target, "TRACK_NEGATIVE_Z", "UP_Y"
area("key", (2.5, -6, 4), 520, CREAM, 4)
area("rim", (-3, 4, 2), 1400, RED, 3)
area("fill", (-4, -5, 0.5), 200, CREAM, 5)

import os
os.makedirs(OUT, exist_ok=True)
still = os.environ.get("STILL_FRAME")
if still:
    scene.frame_set(int(still))
    scene.render.filepath = OUT.rstrip("/") + f"/still_{int(still):04d}.png"
    bpy.ops.render.render(write_still=True)
else:
    bpy.ops.render.render(animation=True)
print("DONE", OUT)
