"""Loft and export skinned BattleChess figures in Blender."""

import math
import struct

import bpy
import numpy as np
from mathutils import Vector


def soft_peak(value, center, spread):
    return math.exp(-((value - center) / spread) ** 2)


def curved_sections(sections):
    """Interpolate the robe profile without flattening its fitted curves."""
    sections = np.asarray(sections)
    y = sections[:, 0]
    profiles = sections[:, 1:]
    tangents = np.gradient(profiles, y, axis=0)
    for i, width in enumerate(np.diff(y)):
        for t in np.linspace(0, 1, max(3, math.ceil(width / 0.012)), endpoint=False):
            a = (1 + 2 * t) * (1 - t) ** 2
            b = t * (1 - t) ** 2
            c = t**2 * (3 - 2 * t)
            d = t**2 * (t - 1)
            profile = a * profiles[i] + c * profiles[i + 1]
            profile += width * (b * tangents[i] + d * tangents[i + 1])
            yield y[i] + t * width, *profile
    yield sections[-1]


class Sculpt:
    def __init__(self):
        self.vertices = []
        self.faces = []

    def rings(self, rings):
        offset = len(self.vertices)
        sides = len(rings[0])
        for ring in rings:
            self.vertices.extend(ring)
        self.faces.append(tuple(offset + i for i in range(sides)))
        for section in range(len(rings) - 1):
            lower = offset + section * sides
            upper = lower + sides
            for i in range(sides):
                j = (i + 1) % sides
                self.faces.append((lower + i, upper + i, upper + j, lower + j))
        top = offset + (len(rings) - 1) * sides
        self.faces.append(tuple(top + i for i in reversed(range(sides))))

    def loft(self, sections, sides=24, relief=None):
        rings = []
        for y, x, z, width, depth in sections:
            ring = []
            for index in range(sides):
                angle = math.tau * index / sides
                cos, sin = math.cos(angle), math.sin(angle)
                point_x = x + width * cos
                point_z = z + depth * sin
                if relief:
                    point_x, point_z = relief(point_x, y, point_z, angle)
                ring.append((point_x, y, point_z))
            rings.append(ring)
        self.rings(rings)

    def tube(self, path, sides=12):
        rings = []
        for i, (center, radius) in enumerate(path):
            tangent = Vector(path[min(i + 1, len(path) - 1)][0]) - Vector(path[max(i - 1, 0)][0])
            tangent.normalize()
            across = tangent.cross(Vector((0, 0, 1))).normalized()
            normal = tangent.cross(across)
            rings.append([
                tuple(Vector(center) + radius * (across * math.cos(angle) + normal * math.sin(angle)))
                for angle in (math.tau * j / sides for j in range(sides))
            ])
        self.rings(rings)


def ease(lo, hi, value):
    t = np.clip((value - lo) / (hi - lo), 0, 1)
    return t * t * (3 - 2 * t)


def export_figure(sculpt, rest, weights_for, output, stem, name, voxel_size, detail, stl_output=None):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([(x, -z, y) for x, y, z in sculpt.vertices], [], sculpt.faces)
    mesh.update()
    figure = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(figure)
    bpy.context.view_layer.objects.active = figure
    figure.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')

    mesh.remesh_voxel_size = voxel_size
    bpy.ops.object.voxel_remesh()
    smooth = figure.modifiers.new('Soften joins', 'SMOOTH')
    smooth.factor = 0.59
    smooth.iterations = 3
    bpy.ops.object.modifier_apply(modifier=smooth.name)
    simplify = figure.modifiers.new('Board scale detail', 'DECIMATE')
    simplify.ratio = detail
    bpy.ops.object.modifier_apply(modifier=simplify.name)

    mesh = figure.data
    floor = min(v.co.z for v in mesh.vertices)
    for vertex in mesh.vertices:
        vertex.co.z -= floor
    mesh.update()
    mesh.calc_loop_triangles()
    positions = np.array([(v.co.x, v.co.z, -v.co.y) for v in mesh.vertices])
    normals = np.array([(v.normal.x, v.normal.z, -v.normal.y) for v in mesh.vertices])
    triangles = np.array([tri.vertices[:] for tri in mesh.loop_triangles], dtype='<u4')
    ids, weights = weights_for(positions)

    with (output / f"{stem}.mesh").open('wb') as file:
        if len(rest) == 11:
            file.write(struct.pack('<4sII', b'BCM1', len(positions), triangles.size))
        else:
            file.write(struct.pack('<4sIII', b'BCM2', len(positions), triangles.size, len(rest)))
        joints = np.array(rest, dtype='<f4')
        joints[:, 1] -= floor
        file.write(joints.tobytes())
        vertex = struct.Struct('<6f4H4f')
        for position, normal, joint_ids, joint_weights in zip(positions, normals, ids, weights):
            file.write(vertex.pack(*position, *normal, *joint_ids, *joint_weights))
        file.write(triangles.tobytes())

    for vertex in mesh.vertices:
        vertex.co *= 35
    mesh.update()
    bpy.ops.wm.stl_export(filepath=str((stl_output if stl_output is not None else output) / f"{stem}.stl"), export_selected_objects=True)
    print(f'{name}: {len(positions):,} vertices, {len(triangles):,} triangles')

