"""Build the rook's brick parapet and the brick used for its extending limbs."""

import struct
import sys
from pathlib import Path

import bpy
from mathutils import Vector


PROJECT_ROOT = Path(__file__).resolve().parents[2]
OUTPUT = PROJECT_ROOT / 'art' / 'rook'
STL_OUTPUT = PROJECT_ROOT / 'docs' / 'stl'
OUTPUT.mkdir(parents=True, exist_ok=True)
STL_OUTPUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(Path(__file__).resolve().parent))
from figure_mesh import Sculpt


def brick(sculpt, center, dimensions, bevel):
    x, y, z = center
    w, h, d = (0.5 * axis for axis in dimensions)
    bevel = min(bevel, w * 0.32, h * 0.33, d * 0.32)
    rings = []
    for height, inset in [(-h, bevel), (-h + bevel, 0), (h - bevel, 0), (h, bevel)]:
        a, b = w - inset, d - inset
        corner = bevel * (0.40 if inset else 1.0)
        outline = [
            (-a + corner, -b), (a - corner, -b), (a, -b + corner), (a, b - corner),
            (a - corner, b), (-a + corner, b), (-a, b - corner), (-a, -b + corner),
        ]
        rings.append([(x + px, y + height, z + pz) for px, pz in outline])
    sculpt.rings(rings)


def bricks_between(start, end, mid_width, stagger):
    width = end - start
    if stagger:
        cuts = [start, start + width * 0.23, start + width * 0.50, start + width * 0.77, end]
    else:
        cuts = [start, (start + end - mid_width) / 2, (start + end + mid_width) / 2, end]
    return list(zip(cuts, cuts[1:]))


def build_tower(sculpt):
    # A continuous mortar core connects the separately shaped stones.
    brick(sculpt, (0, 0.410, 0), (0.604, 0.672, 0.604), 0.015)
    brick(sculpt, (0, 0.050, 0), (0.756, 0.100, 0.756), 0.019)
    brick(sculpt, (0, 0.128, 0), (0.710, 0.081, 0.710), 0.018)

    for row, (y, size) in enumerate([(0.242, 0.694), (0.399, 0.680), (0.556, 0.688), (0.713, 0.684)]):
        half = size / 2
        depth = 0.104
        gap = 0.011
        h = 0.154
        for side in (-1, 1):
            course = bricks_between(-half, half, 0.234, row % 2 == 1)
            if side == -1 and row == 2:
                # A thin inset arrow slit identifies the face of the battlement.
                left, (begin, end), right = course
                course = [left, (begin, -0.021), (0.021, end), right]
            for start, end in course:
                x = (start + end) / 2
                brick(sculpt, (x, y, side * (half - depth / 2)), (end - start - gap, h, depth), 0.015)
            for start, end in bricks_between(-half + depth * 0.82, half - depth * 0.82, 0.189, row % 2 == 0):
                z = (start + end) / 2
                brick(sculpt, (side * (half - depth / 2), y, z), (depth, h, end - start - gap), 0.015)

    # A narrow rim surrounds the launch well below the opening battlement.
    for side in (-1, 1):
        brick(sculpt, (side * 0.323, 0.813, 0), (0.082, 0.074, 0.728), 0.013)
        brick(sculpt, (0, 0.813, side * 0.323), (0.578, 0.074, 0.082), 0.013)


def build_hatch(sculpt):
    # Each half carries its coping and merlons on a hinge along the outer wall.
    brick(sculpt, (0, 0.878, 0), (0.800, 0.073, 0.800), 0.019)
    brick(sculpt, (0, 0.940, 0), (0.758, 0.073, 0.758), 0.014)
    for ix in (-1, 0, 1):
        for iz in (-1, 0, 1):
            if ix or iz:
                brick(sculpt, (ix * 0.290, 1.052, iz * 0.290), (0.168, 0.209, 0.168), 0.017)


def make_object(sculpt, name):
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
    return figure


def write_mesh(mesh, name, smooth):
    mesh.calc_loop_triangles()
    triangles = mesh.loop_triangles
    with (OUTPUT / f'{name}.mesh').open('wb') as file:
        if smooth:
            file.write(struct.pack('<4sIII', b'BCM2', len(mesh.vertices), 3 * len(triangles), 0))
            for vert in mesh.vertices:
                x, y, z = vert.co
                nx, ny, nz = vert.normal
                file.write(struct.pack('<6f', x, z, -y, nx, nz, -ny))
            for tri in triangles:
                file.write(struct.pack('<3I', *tri.vertices))
        else:
            file.write(struct.pack('<4sIII', b'BCM2', 3 * len(triangles), 3 * len(triangles), 0))
            for tri in triangles:
                nx, ny, nz = tri.normal
                for index in tri.vertices:
                    x, y, z = mesh.vertices[index].co
                    file.write(struct.pack('<6f', x, z, -y, nx, nz, -ny))
            for i in range(len(triangles)):
                file.write(struct.pack('<3I', 3 * i, 3 * i + 1, 3 * i + 2))


def build():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    tower = Sculpt()
    build_tower(tower)
    rook = make_object(tower, 'Crenellated rook')
    rook.data.remesh_voxel_size = 0.005
    bpy.ops.object.voxel_remesh()
    finish = rook.modifiers.new('Soften stone edges', 'SMOOTH')
    finish.factor = 0.35
    finish.iterations = 2
    bpy.ops.object.modifier_apply(modifier=finish.name)
    simplify = rook.modifiers.new('Board scale masonry', 'DECIMATE')
    simplify.ratio = 0.087
    bpy.ops.object.modifier_apply(modifier=simplify.name)
    floor = min(vertex.co.z for vertex in rook.data.vertices)
    for vertex in rook.data.vertices:
        vertex.co.z -= floor
    rook.data.update()
    write_mesh(rook.data, 'rook', smooth=True)
    rook.select_set(False)

    top = Sculpt()
    build_hatch(top)
    hatch = make_object(top, 'Opening battlement')
    hatch.data.remesh_voxel_size = 0.005
    bpy.ops.object.voxel_remesh()
    finish = hatch.modifiers.new('Soften coping edges', 'SMOOTH')
    finish.factor = 0.35
    finish.iterations = 2
    bpy.ops.object.modifier_apply(modifier=finish.name)
    simplify = hatch.modifiers.new('Board scale coping', 'DECIMATE')
    simplify.ratio = 0.087
    bpy.ops.object.modifier_apply(modifier=simplify.name)
    import bmesh
    for side, name in [(-1, 'hatch-left'), (1, 'hatch-right')]:
        half = bmesh.new()
        half.from_mesh(hatch.data)
        result = bmesh.ops.bisect_plane(
            half, geom=list(half.verts) + list(half.edges) + list(half.faces),
            plane_co=(side * 0.003, 0, 0), plane_no=(1, 0, 0),
            clear_outer=side < 0, clear_inner=side > 0,
        )
        boundary = [edge for edge in result['geom_cut'] if isinstance(edge, bmesh.types.BMEdge) and edge.is_boundary]
        bmesh.ops.holes_fill(half, edges=boundary, sides=0)
        bmesh.ops.recalc_face_normals(half, faces=list(half.faces))
        mesh = bpy.data.meshes.new(name)
        half.to_mesh(mesh)
        half.free()
        for vertex in mesh.vertices:
            vertex.co.x -= side * 0.40
            vertex.co.z -= 0.85
        mesh.update()
        write_mesh(mesh, name, smooth=True)

    # The assembled STL shows the closed, resting rook.
    rook.select_set(True)
    bpy.context.view_layer.objects.active = rook
    bpy.ops.object.join()
    rook.data.remesh_voxel_size = 0.005
    bpy.ops.object.voxel_remesh()
    simplify = rook.modifiers.new('Assembled STL', 'DECIMATE')
    simplify.ratio = 0.087
    bpy.ops.object.modifier_apply(modifier=simplify.name)
    for vertex in rook.data.vertices:
        vertex.co *= 35
    rook.data.update()
    bpy.ops.wm.stl_export(filepath=str(STL_OUTPUT / 'rook.stl'), export_selected_objects=True)
    print(f'Rook with hinged battlement: {len(rook.data.vertices):,} vertices')
    rook.select_set(False)

    stone = Sculpt()
    brick(stone, Vector((0, 0, 0)), (1, 1, 1), 0.085)
    component = make_object(stone, 'Limb brick')
    write_mesh(component.data, 'brick', smooth=False)


if __name__ == '__main__':
    build()

