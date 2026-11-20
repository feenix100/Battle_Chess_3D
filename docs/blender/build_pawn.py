"""Build the modern infantry pawn and its Bevy skin with Blender."""

import math
import struct
from pathlib import Path

import bpy
import numpy as np
from mathutils.bvhtree import BVHTree


PROJECT_ROOT = Path(__file__).resolve().parents[2]
OUTPUT = PROJECT_ROOT / 'art' / 'pawn'
STL_OUTPUT = PROJECT_ROOT / 'docs' / 'stl'
OUTPUT.mkdir(parents=True, exist_ok=True)
STL_OUTPUT.mkdir(parents=True, exist_ok=True)

# The model faces -Z with Y up, as expected by the character movement system.
REST_JOINTS = [
    (0.0, 0.555, 0.0), (0.0, 0.995, -0.002), (0.255, 0.552, -0.052),
    (-0.202, 0.919, 0.0), (-0.227, 0.738, -0.011),
    (0.202, 0.919, 0.0), (0.227, 0.738, -0.026),
    (-0.099, 0.553, 0.013), (-0.103, 0.292, -0.017),
    (0.099, 0.553, 0.013), (0.103, 0.292, -0.017),
]


def build_shapes():
    vertices = []
    faces = []

    def loft(rings, sides=20, power=1.0):
        """Each ring has height, x and z centers, and two half widths."""
        base = len(vertices)
        for height, x, z, width, depth in rings:
            for index in range(sides):
                angle = math.tau * index / sides
                c, s = math.cos(angle), math.sin(angle)
                vertices.append((x + width * math.copysign(abs(c) ** power, c),
                                 height, z + depth * math.copysign(abs(s) ** power, s)))
        faces.append(tuple(base + i for i in range(sides)))
        for ring in range(len(rings) - 1):
            lower = base + ring * sides
            upper = lower + sides
            for i in range(sides):
                j = (i + 1) % sides
                faces.append((lower + i, upper + i, upper + j, lower + j))
        top = base + (len(rings) - 1) * sides
        faces.append(tuple(top + i for i in reversed(range(sides))))

    def panel(center, size, bevel=0.012):
        x, y, z = center
        width, height, depth = (dimension * 0.5 for dimension in size)
        b = min(bevel, width * 0.6, height * 0.55, depth * 0.6)
        loft([(y - height, x, z, width - b, depth - b),
              (y - height + b, x, z, width, depth),
              (y + height - b, x, z, width, depth),
              (y + height, x, z, width - b, depth - b)], sides=16, power=0.34)

    # Tapered field uniform and pelvis, with a shaped collar and shoulders.
    loft([(0.465, 0, 0.014, 0.121, 0.079), (0.521, 0, 0.017, 0.166, 0.112),
          (0.617, 0, 0.013, 0.171, 0.110), (0.693, 0, 0.012, 0.155, 0.099),
          (0.814, 0, 0.013, 0.178, 0.113), (0.910, 0, 0.014, 0.200, 0.100),
          (0.958, 0, 0.010, 0.171, 0.081), (0.983, 0, 0.004, 0.097, 0.070)], power=0.79)
    loft([(0.929, 0, 0.002, 0.074, 0.068), (0.984, 0, -0.005, 0.068, 0.064),
          (1.061, 0, -0.008, 0.057, 0.059)])

    # The head has a distinct jaw, face plane and nose under a high-cut helmet.
    loft([(1.006, 0, -0.024, 0.038, 0.057), (1.039, 0, -0.025, 0.063, 0.073),
          (1.096, 0, -0.014, 0.078, 0.078), (1.151, 0, -0.006, 0.086, 0.084),
          (1.203, 0, 0.003, 0.085, 0.084), (1.245, 0, 0.012, 0.065, 0.071)], power=0.78)
    loft([(1.081, 0, -0.075, 0.019, 0.022), (1.098, 0, -0.092, 0.025, 0.028),
          (1.115, 0, -0.090, 0.021, 0.030), (1.156, 0, -0.072, 0.016, 0.021)], sides=12)
    loft([(1.142, 0, 0.029, 0.091, 0.083), (1.171, 0, 0.015, 0.105, 0.110),
          (1.213, 0, 0.016, 0.108, 0.109), (1.259, 0, 0.020, 0.096, 0.096),
          (1.299, 0, 0.024, 0.073, 0.074), (1.321, 0, 0.026, 0.034, 0.040)], power=0.79)
    panel((0, 1.173, -0.072), (0.176, 0.036, 0.074))

    # Flat vest plates, two pouches and a small back plate read at board zoom.
    panel((0, 0.810, -0.082), (0.321, 0.237, 0.133), bevel=0.024)
    panel((0, 0.817, 0.094), (0.301, 0.236, 0.099), bevel=0.023)
    for side in [-1, 1]:
        panel((side * 0.075, 0.743, -0.141), (0.116, 0.075, 0.074))
        panel((side * 0.110, 0.917, -0.067), (0.051, 0.085, 0.093))
    panel((0, 0.622, -0.071), (0.282, 0.062, 0.122))

    for side in [-1, 1]:
        front = -0.024 if side < 0 else -0.049
        loft([(0.507, side * 0.239, front, 0.027, 0.032),
              (0.539, side * 0.243, front, 0.040, 0.037),
              (0.579, side * 0.246, front + 0.007, 0.041, 0.039),
              (0.643, side * 0.233, front + 0.012, 0.044, 0.047),
              (0.725, side * 0.226, front + 0.020, 0.049, 0.050),
              (0.779, side * 0.221, 0.000, 0.054, 0.056),
              (0.855, side * 0.213, 0.006, 0.065, 0.071),
              (0.927, side * 0.189, 0.006, 0.071, 0.068),
              (0.958, side * 0.171, 0.010, 0.043, 0.047)], power=0.85)
        loft([(0.018, side * 0.106, 0.003, 0.048, 0.050),
              (0.090, side * 0.104, 0.014, 0.049, 0.047),
              (0.169, side * 0.103, 0.010, 0.051, 0.055),
              (0.242, side * 0.104, -0.001, 0.058, 0.061),
              (0.299, side * 0.104, -0.010, 0.064, 0.068),
              (0.377, side * 0.100, 0.001, 0.069, 0.076),
              (0.462, side * 0.098, 0.017, 0.077, 0.082),
              (0.547, side * 0.090, 0.016, 0.083, 0.087),
              (0.597, side * 0.084, 0.013, 0.066, 0.080)], power=0.86)
        loft([(0.000, side * 0.107, -0.033, 0.055, 0.087),
              (0.018, side * 0.107, -0.035, 0.060, 0.095),
              (0.054, side * 0.106, -0.034, 0.057, 0.089),
              (0.084, side * 0.105, -0.003, 0.052, 0.062),
              (0.151, side * 0.104, 0.008, 0.053, 0.056)], sides=16, power=0.60)
        panel((side * 0.105, 0.294, -0.054), (0.099, 0.100, 0.063))

    # A compact revolver, facing -Z. Its angled grip runs through the palm,
    # behind and below the cylinder; the hand never attaches to the barrel.
    weapon_faces_start = len(faces)
    loft([(0.493, 0.249, -0.020, 0.027, 0.024),
          (0.507, 0.250, -0.022, 0.031, 0.027),
          (0.575, 0.254, -0.055, 0.029, 0.028),
          (0.609, 0.255, -0.078, 0.027, 0.027)], sides=16, power=0.55)
    panel((0.255, 0.621, -0.112), (0.064, 0.056, 0.120), bevel=0.008)

    def axial(rings, sides=24, fluted=False):
        """Closed rings along -Z, with optional six chamber flutes."""
        start = len(vertices)
        for z, radius, groove in rings:
            for index in range(sides):
                angle = math.tau * index / sides
                scallop = ((1 + math.cos(6 * angle)) * 0.5) ** 4
                r = radius - groove * scallop if fluted else radius
                vertices.append((0.255 + r * math.cos(angle),
                                 0.642 + r * math.sin(angle), z))
        faces.append(tuple(start + i for i in range(sides)))
        for ring in range(len(rings) - 1):
            rear = start + ring * sides
            front = rear + sides
            for index in range(sides):
                next_index = (index + 1) % sides
                faces.append((rear + index, front + index,
                              front + next_index, rear + next_index))
        front = start + (len(rings) - 1) * sides
        faces.append(tuple(front + i for i in reversed(range(sides))))

    # Broad cylinder, short barrel, top strap and a small hammer make the
    # silhouette read as a revolver at board scale without fine ornament.
    axial([(-0.112, 0.043, 0), (-0.120, 0.052, 0),
           (-0.132, 0.052, 0.008), (-0.200, 0.052, 0.008),
           (-0.212, 0.052, 0), (-0.220, 0.043, 0)], sides=36, fluted=True)
    axial([(-0.210, 0.022, 0), (-0.225, 0.025, 0),
           (-0.344, 0.025, 0), (-0.354, 0.020, 0)], sides=8)
    panel((0.255, 0.616, -0.277), (0.037, 0.024, 0.135), bevel=0.006)
    panel((0.255, 0.676, -0.329), (0.020, 0.026, 0.028), bevel=0.004)
    panel((0.255, 0.693, -0.164), (0.038, 0.016, 0.112), bevel=0.004)
    panel((0.255, 0.666, -0.062), (0.022, 0.040, 0.026), bevel=0.005)

    # An open trigger guard connects the receiver to the front of the grip.
    panel((0.255, 0.558, -0.185), (0.026, 0.081, 0.020), bevel=0.004)
    panel((0.255, 0.526, -0.139), (0.026, 0.018, 0.109), bevel=0.004)
    panel((0.255, 0.574, -0.128), (0.017, 0.043, 0.016), bevel=0.004)
    return vertices, faces, weapon_faces_start


def ease(start, end, value):
    t = np.clip((value - start) / (end - start), 0, 1)
    return t * t * (3 - 2 * t)


def weights_for(positions, rigid_weapon):
    x, y, z = positions.T
    weights = np.zeros((len(x), len(REST_JOINTS)), dtype=np.float32)
    weights[:, 1] = ease(0.966, 1.043, y)
    # The palm wraps the grip and follows its joint. Blend at the wrist, while
    # the cylinder, frame, trigger guard and barrel remain rigid together.
    palm = ease(0.190, 0.215, x) * (1 - ease(0.582, 0.650, y))
    weights[:, 2] = np.maximum(palm, rigid_weapon)
    left = x < 0
    free = 1 - weights[:, 1] - weights[:, 2]
    arms = free * ease(0.155, 0.224, abs(x)) * ease(0.456, 0.516, y)
    forearm = 1 - ease(0.698, 0.779, y)
    legs = (free - arms) * (1 - ease(0.495, 0.632, y))
    shin = 1 - ease(0.249, 0.333, y)
    for side, upper_arm, upper_leg in [(left, 3, 7), (~left, 5, 9)]:
        weights[:, upper_arm] = side * arms * (1 - forearm)
        weights[:, upper_arm + 1] = side * arms * forearm
        weights[:, upper_leg] = side * legs * (1 - shin)
        weights[:, upper_leg + 1] = side * legs * shin
    weights[:, 0] = np.maximum(0, free - arms - legs)
    ids = np.argsort(-weights, axis=1)[:, :4]
    weights = np.take_along_axis(weights, ids, axis=1)
    weights /= weights.sum(axis=1, keepdims=True)
    return ids, weights


def build():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    vertices, faces, weapon_faces_start = build_shapes()
    # Preserve the original part surfaces across voxel fusion. Nearest-surface
    # ownership identifies the whole revolver, including its rear hammer and
    # grip, without accidentally weighting the adjacent forearm as a weapon.
    body_surface = BVHTree.FromPolygons(vertices, faces[:weapon_faces_start])
    weapon_surface = BVHTree.FromPolygons(vertices, faces[weapon_faces_start:])
    mesh = bpy.data.meshes.new('Infantry pawn')
    mesh.from_pydata([(x, -z, y) for x, y, z in vertices], [], faces)
    mesh.update()
    pawn = bpy.data.objects.new('Infantry pawn', mesh)
    bpy.context.collection.objects.link(pawn)
    bpy.context.view_layer.objects.active = pawn
    pawn.select_set(True)

    # Fuse the uniform, body and held equipment into one continuous sculpt.
    mesh.remesh_voxel_size = 0.0065
    bpy.ops.object.voxel_remesh()
    smooth = pawn.modifiers.new('Soften seams', 'SMOOTH')
    smooth.factor = 0.55
    smooth.iterations = 3
    bpy.ops.object.modifier_apply(modifier=smooth.name)

    # Two shallow sockets and a small mouth crease survive the board's lighting.
    for vertex in pawn.data.vertices:
        x, front, height = vertex.co
        if 1.045 < height < 1.160 and front > 0.050:
            eye = math.exp(-((abs(x) - 0.038) / 0.019) ** 2 - ((height - 1.132) / 0.012) ** 2)
            mouth = math.exp(-(x / 0.033) ** 4 - ((height - 1.072) / 0.005) ** 2)
            vertex.co.y -= 0.013 * eye + 0.006 * mouth
    pawn.data.update()
    decay = pawn.modifiers.new('Board scale detail', 'DECIMATE')
    decay.ratio = 0.18
    bpy.ops.object.modifier_apply(modifier=decay.name)

    mesh = pawn.data
    floor = min(vertex.co.z for vertex in mesh.vertices)
    for vertex in mesh.vertices:
        vertex.co.z -= floor
    mesh.update()
    mesh.calc_loop_triangles()
    coordinates = np.array([(v.co.x, v.co.z, -v.co.y) for v in mesh.vertices])
    normals = np.array([(v.normal.x, v.normal.z, -v.normal.y) for v in mesh.vertices])
    triangles = np.array([tri.vertices[:] for tri in mesh.loop_triangles], dtype='<u4')
    rigid_weapon = np.array([
        float(weapon_surface.find_nearest((x, y + floor, z))[3]
              <= body_surface.find_nearest((x, y + floor, z))[3])
        for x, y, z in coordinates
    ], dtype=np.float32)
    ids, weights = weights_for(coordinates, rigid_weapon)
    with (OUTPUT / 'pawn.mesh').open('wb') as file:
        file.write(struct.pack('<4sII', b'BCM1', len(coordinates), triangles.size))
        joints = np.array(REST_JOINTS, dtype='<f4')
        joints[:, 1] -= floor
        file.write(joints.tobytes())
        vertex = struct.Struct('<6f4H4f')
        for position, normal, joint_ids, joint_weights in zip(coordinates, normals, ids, weights):
            file.write(vertex.pack(*position, *normal, *joint_ids, *joint_weights))
        file.write(triangles.tobytes())

    for vertex in mesh.vertices:
        vertex.co *= 35
    mesh.update()
    bpy.ops.wm.stl_export(filepath=str(STL_OUTPUT / 'pawn.stl'), export_selected_objects=True)
    print(f'Pawn: {len(coordinates):,} vertices, {len(triangles):,} triangles')


if __name__ == '__main__':
    build()
