"""Build the knight STL and Bevy skin from the attributed warhorse sculpt.

Run inside Blender with `blender --background --factory-startup --python
docs/blender/build_knight.py -- --source /path/to/original.stl` from the project root.
"""

import argparse
import sys
import math
import struct
from pathlib import Path

import bpy
import numpy as np


PROJECT_ROOT = Path(__file__).resolve().parents[2]
OUTPUT = PROJECT_ROOT / 'art' / 'knight'
STL_OUTPUT = PROJECT_ROOT / 'docs' / 'stl'
OUTPUT.mkdir(parents=True, exist_ok=True)
STL_OUTPUT.mkdir(parents=True, exist_ok=True)
parser = argparse.ArgumentParser(description="Adapt MZ4250's original warhorse STL.")
parser.add_argument('--source', type=Path, required=True,
                    help='Path to the original Warhorse Armored no stand.stl download.')
arguments = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
SOURCE = parser.parse_args(arguments).source.expanduser().resolve()
if not SOURCE.is_file():
    raise FileNotFoundError(f'Original warhorse STL not found: {SOURCE}')

# Source-space shoulder/hip and knee/hock pivots. Forelegs come first.
LEGS = [
    ((4.0, -12.7, 22.0), (4.2, -20.2, 10.7), 8.2),
    ((-4.1, -13.5, 22.0), (-3.9, -23.6, 11.1), 6.2),
    ((4.2, 12.9, 22.9), (2.3, 19.7, 10.1), -5.5),
    ((-4.2, 11.9, 22.9), (-3.1, 11.3, 10.5), 8.9),
]


def ease(start, end, value):
    t = np.clip((value - start) / (end - start), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def leg_zones(points):
    x, y, _ = points.T
    fore = 1.0 - ease(-8.5, -3.0, y)
    hind = ease(1.5, 4.2, y)
    near = ease(-0.4, 0.4, x)
    return np.column_stack((fore * near, fore * (1 - near), hind * near, hind * (1 - near)))


def sculpt(points):
    """Level the stride, raise the head and shorten the tail for a square."""
    result = points.copy()
    x, y, z = points.T
    feet = 1.0 - ease(2.0, 22.5, z)
    result[:, 1] += feet * (leg_zones(points) @ np.array([leg[2] for leg in LEGS]))

    neck = ease(14.5, 28.0, -y) * ease(16.0, 27.0, z)
    angle = math.radians(-27.0) * neck
    uy, uz = y + 13.0, z - 25.0
    result[:, 1] += uy * (np.cos(angle) - 1.0) - uz * np.sin(angle)
    result[:, 2] += uy * np.sin(angle) + uz * (np.cos(angle) - 1.0)

    tail = ease(19.0, 27.0, y) * (1.0 - ease(2.5, 5.0, abs(x)))
    result[:, 1] -= 0.42 * np.maximum(0.0, y - 19.0) * tail
    return result


def skin(points):
    x, y, z = points.T
    result = np.zeros((len(points), 11), dtype=np.float32)
    result[:, 1] = ease(12.0, 24.0, -y) * ease(17.0, 31.5, z)
    result[:, 2] = ease(16.0, 24.0, y) * ease(16.0, 21.0, z) * (1.0 - ease(1.8, 5.0, abs(x)))
    leg_share = (1.0 - ease(13.3, 23.2, z)) * (1.0 - result[:, 1] - result[:, 2])
    for leg, region in enumerate(leg_zones(points).T):
        knee_height = LEGS[leg][1][2]
        lower_share = 1.0 - ease(knee_height - 3.2, knee_height + 3.2, z)
        weight = region * leg_share
        result[:, 3 + leg * 2] = weight * (1.0 - lower_share)
        result[:, 4 + leg * 2] = weight * lower_share
    result[:, 0] = 1 - result[:, 1:].sum(axis=1)
    ids = np.argsort(-result, axis=1)[:, :4]
    weights = np.take_along_axis(result, ids, axis=1)
    weights /= weights.sum(axis=1, keepdims=True)
    return ids, weights


def bevy_axes(positions):
    # Blender forward is -Y and up is +Z. This rotation preserves winding.
    return positions[:, [0, 2, 1]] * [-1, 1, 1]


def build():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.wm.stl_import(filepath=str(SOURCE), use_mesh_validate=True)
    horse = bpy.context.object
    horse.name = 'BattleChess Warhorse'
    decimate = horse.modifiers.new('Board scale detail', 'DECIMATE')
    decimate.ratio = 0.036
    bpy.ops.object.modifier_apply(modifier=decimate.name)
    mesh = horse.data

    original = np.empty(len(mesh.vertices) * 3, dtype=np.float32)
    mesh.vertices.foreach_get('co', original)
    original = original.reshape((-1, 3))
    ids, weights = skin(original)
    shaped = sculpt(original)
    floor = float(shaped[:, 2].min())
    center = float((shaped[:, 1].max() + shaped[:, 1].min()) * 0.5)
    scale = 1.24 / float(np.ptp(shaped[:, 1]))

    def normalize(points):
        return (points - [0.0, center, floor]) * [scale, scale, scale * 1.14]

    joints = [(0.0, 0.0, 22.5), (0.0, -14.5, 25.8), (0.0, 17.5, 25.0)]
    joints += [point for leg in LEGS for point in leg[:2]]
    joints = bevy_axes(normalize(sculpt(np.array(joints))))
    coordinates = normalize(shaped)
    mesh.vertices.foreach_set('co', np.asarray(coordinates, dtype=np.float32).ravel())
    mesh.update()
    mesh.calc_loop_triangles()
    normals = np.array([v.normal[:] for v in mesh.vertices])
    triangles = np.array([triangle.vertices[:] for triangle in mesh.loop_triangles], dtype='<u4')

    # Each little-endian vertex packs position, smooth normal, four joint
    # indices, and four weights. Triangle indices follow the vertex array.
    with (OUTPUT / 'warhorse.mesh').open('wb') as file:
        file.write(struct.pack('<4sII', b'BCM1', len(coordinates), triangles.size))
        file.write(np.asarray(joints, dtype='<f4').tobytes())
        vertex = struct.Struct('<6f4H4f')
        for position, normal, joint_ids, joint_weights in zip(bevy_axes(coordinates), bevy_axes(normals), ids, weights):
            file.write(vertex.pack(*position, *normal, *joint_ids, *joint_weights))
        file.write(triangles.tobytes())

    # Export the identical rest shape at miniature scale, Z up in millimeters.
    mesh.vertices.foreach_set('co', np.asarray(coordinates * 45.0, dtype=np.float32).ravel())
    mesh.update()
    bpy.ops.wm.stl_export(filepath=str(STL_OUTPUT / 'warhorse.stl'), export_selected_objects=True)
    print(f'Warhorse: {len(coordinates)} vertices, {len(triangles)} triangles')


if __name__ == '__main__':
    build()
