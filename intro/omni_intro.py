"""OMNI Launcher opening, rendered headless:
/Applications/Blender.app/Contents/MacOS/Blender -b -P omni_intro.py -- <out dir> [frames]
STILL_FRAME=<n> renders one frame instead.

Our own scene, nothing from Mojang: flat-coloured voxel cubes (made here, no textures) fly in
from the dark and lock into the OMNI Launcher logo (brand/omni-launcher-brand-sheet.png): a thick
cream ring with a notch, a four-point star inside, a red orbit that passes in front of the ring
and behind it, and a gold sparkle. Then the camera pushes in.
Brand: charcoal #1A1A1D, cream #E8DCC8, red #C83246, gold #E8B44F.
"""
import bpy, math, os, random, sys

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
OUT = argv[0] if argv else "/tmp/omni-intro"
FRAMES = int(argv[1]) if len(argv) > 1 else 84
FPS = 24
random.seed(11)

def hexcol(h):
    h = h.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))
    f = lambda c: ((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92
    return (f(r), f(g), f(b), 1.0)

CHARCOAL, CREAM, RED, GOLD = map(hexcol, ("#1A1A1D", "#E8DCC8", "#C83246", "#E8B44F"))

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 960, 540
scene.render.fps = FPS
scene.frame_start, scene.frame_end = 1, FRAMES
scene.render.image_settings.file_format = "PNG"
try:
    scene.eevee.taa_render_samples = 48
    scene.view_settings.view_transform = "Standard"
except Exception:
    pass

world = bpy.data.worlds.new("w")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = CHARCOAL
scene.world = world

def material(name, color, rough, emit):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    p = m.node_tree.nodes["Principled BSDF"]
    p.inputs["Base Color"].default_value = color
    p.inputs["Roughness"].default_value = rough
    if "Emission Color" in p.inputs:
        p.inputs["Emission Color"].default_value = color
        p.inputs["Emission Strength"].default_value = emit
    return m

MATS = {"cream": material("cream", CREAM, 0.55, 0.04),
        "red": material("red", RED, 0.4, 0.3),
        "gold": material("gold", GOLD, 0.3, 0.9)}

def ease(t):
    return 1 - (1 - t) ** 5

def key(obj, frame, **vals):
    for attr, v in vals.items():
        setattr(obj, attr, v)
        obj.keyframe_insert(attr, frame=frame)

logo = bpy.data.objects.new("logo", None)
scene.collection.objects.link(logo)

COUNT = [0]
def cube(kind, end, rot_end, size, f0, scale_end=1.0, fly=24):
    """A cube that starts scattered in the dark and lands at `end` at f0 + fly."""
    bpy.ops.mesh.primitive_cube_add(size=size)
    c = bpy.context.active_object
    COUNT[0] += 1
    c.name = f"cube{COUNT[0]}"
    c.data.materials.append(MATS[kind])
    c.parent = logo
    b = c.modifiers.new("bevel", "BEVEL")
    b.width, b.segments = size * 0.07, 2
    far = 3.5 + random.random() * 5
    ang = random.random() * math.tau
    start = (far * math.cos(ang), random.uniform(-4, 3), far * math.sin(ang))
    rot_start = tuple(random.uniform(-3, 3) for _ in range(3))
    tiny = (0.01, 0.01, 0.01)
    key(c, 1, location=start, rotation_euler=rot_start, scale=tiny)
    key(c, f0, location=start, rotation_euler=rot_start, scale=tiny)
    for k in range(1, 7):
        t = k / 6
        e = ease(t)
        key(c, f0 + fly * t,
            location=tuple(s + (d - s) * e for s, d in zip(start, end)),
            rotation_euler=tuple(s + (d - s) * e for s, d in zip(rot_start, rot_end)),
            scale=tuple(max(0.01, x * e) for x in (scale_end,) * 3) if isinstance(scale_end, float)
            else tuple(max(0.01, x * e) for x in scale_end))
    return c

S = 0.16
# 1. the ring: three rows thick, three cubes deep, with a notch at the upper right
order = 0
for row_r in (0.95, 1.11, 1.27):
    n = int(2 * math.pi * row_r / S)
    for i in range(n):
        th = 2 * math.pi * i / n
        deg = math.degrees(th)
        if row_r > 1.0 and 38 <= deg <= 62:      # the notch
            continue
        for layer in (-1, 0, 1):
            end = (row_r * math.cos(th), layer * S, row_r * math.sin(th))
            f0 = 3 + (order % 140) * 0.14 + (layer + 1) * 1.2
            cube("cream", end, (0, -th, 0), S, f0)
        order += 1

# 2. the four-point star: arms taper from the middle
star = []
for t in range(-6, 7):
    for axis in (0, 1):
        if t == 0 and axis == 1:
            continue
        w = 1 - abs(t) / 7.0
        pos = (t * 0.085 * (1 if axis == 0 else 0), 0, t * 0.085 * (1 if axis == 1 else 0))
        star.append((pos, w))
for j, (pos, w) in enumerate(star):
    sz = 0.2 * max(0.3, w)
    cube("cream", pos, (0, 0, 0), sz, 30 + j * 0.5, scale_end=1.0)

# 3. the orbit: an ellipse that passes in front of the ring below and behind it above,
#    tilted so the left end is low
A, B, D, TILT = 1.95, 0.75, 0.6, math.radians(-14)
N_ORBIT = 78
for i in range(N_ORBIT):
    ph = 2 * math.pi * i / N_ORBIT
    x, y, z = A * math.cos(ph), D * math.sin(ph) * -1.0, B * math.sin(ph)
    # rotate about the view axis (Y)
    xr = x * math.cos(TILT) + z * math.sin(TILT)
    zr = -x * math.sin(TILT) + z * math.cos(TILT)
    taper = 0.55 + 0.45 * abs(math.sin(ph))      # thin at the ends, thick in front and behind
    sz = 0.15 * taper
    cube("red", (xr, y, zr), (0, 0, TILT), sz, 38 + i * 0.28, fly=20)

# 4. the gold sparkle at the upper right
SP = [((0, 0), 1.0), ((1, 0), 0.7), ((-1, 0), 0.7), ((0, 1), 0.7), ((0, -1), 0.7),
      ((2, 0), 0.4), ((-2, 0), 0.4), ((0, 2), 0.4), ((0, -2), 0.4)]
for (dx, dz), w in SP:
    cube("gold", (1.75 + dx * 0.11, 0, 1.45 + dz * 0.11), (0, 0, 0), 0.13 * w + 0.03, 58, fly=16)

# a slow settle of the whole logo once it is built
key(logo, 1, rotation_euler=(0, 0, 0))
key(logo, int(FRAMES * 0.5), rotation_euler=(0, 0, 0))
key(logo, FRAMES, rotation_euler=(0, math.radians(-9), 0))

target = bpy.data.objects.new("target", None)
scene.collection.objects.link(target)
target.location = (0.1, 0, 0.1)
cam_data = bpy.data.cameras.new("cam")
cam_data.lens = 35
cam = bpy.data.objects.new("cam", cam_data)
scene.collection.objects.link(cam)
scene.camera = cam
tt = cam.constraints.new("TRACK_TO")
tt.target, tt.track_axis, tt.up_axis = target, "TRACK_NEGATIVE_Z", "UP_Y"
key(cam, 1, location=(1.8, -13.0, 1.2))
key(cam, FRAMES, location=(0.2, -10.2, 0.0))

def area(name, loc, energy, color, size):
    d = bpy.data.lights.new(name, "AREA")
    d.energy, d.color, d.size = energy, color[:3], size
    o = bpy.data.objects.new(name, d)
    o.location = loc
    scene.collection.objects.link(o)
    c = o.constraints.new("TRACK_TO")
    c.target, c.track_axis, c.up_axis = target, "TRACK_NEGATIVE_Z", "UP_Y"
area("key", (2.5, -7, 4), 600, CREAM, 4)
area("rim", (-3, 4, 2), 1300, RED, 3)
area("fill", (-4, -6, 0.5), 220, CREAM, 5)

os.makedirs(OUT, exist_ok=True)
still = os.environ.get("STILL_FRAME")
if still:
    scene.frame_set(int(still))
    scene.render.filepath = OUT.rstrip("/") + f"/still_{int(still):04d}.png"
    bpy.ops.render.render(write_still=True)
else:
    scene.render.filepath = OUT.rstrip("/") + "/f_"
    bpy.ops.render.render(animation=True)
print("DONE", OUT)
