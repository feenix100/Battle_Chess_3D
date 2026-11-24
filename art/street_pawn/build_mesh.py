"""Original street pawn: sculpt, editable Blender rig, STL and BattleChess skins.

Run: blender --background --factory-startup --python art/street_pawn/build_mesh.py
Game coordinates are Y up, forward -Z; Blender coordinates are Z up, forward +Y.
No downloaded character geometry is used.
"""
import json
import math
import struct
from pathlib import Path

import bpy
import numpy as np
from mathutils import Vector

OUT = Path(__file__).resolve().parent
REST = np.array([
    (0, .555, .010), (0, 1.016, .000), (.255, .646, -.090),
    (-.205, .916, .005), (-.252, .748, -.012),
    (.205, .916, .005), (.252, .748, -.012),
    (-.096, .550, .016), (-.119, .292, -.010),
    (.096, .550, .016), (.119, .292, -.010),
], dtype=float)
MUZZLE = (.255, .674, -.361)
COLORS = {
    'team': (.235, .49, .32, 1),
    'skin': (.47, .245, .135, 1),
    'denim': (.075, .105, .145, 1),
    'ivory': (.83, .82, .73, 1),
    'gold': (.72, .43, .075, 1),
    'dark': (.025, .033, .044, 1),
    'stitch': (.28, .32, .35, 1),
}
OBJECTS = []

def coord(p):
    x, y, z = p
    return (x, -z, y)

def smoothstep(a, b, value):
    t = np.clip((value - a) / (b - a), 0, 1)
    return t*t*(3-2*t)

def influences(points, kind):
    n = len(points)
    w = np.zeros((n, len(REST)))
    if isinstance(kind, int):
        w[:, kind] = 1
    elif kind == 'torso':
        head = smoothstep(.988, 1.052, points[:, 1])
        w[:, 0], w[:, 1] = 1-head, head
    elif kind.startswith('arm'):
        right = kind.endswith('R')
        upper = 5 if right else 3
        elbow = 1-smoothstep(.712, .796, points[:, 1])
        w[:, upper], w[:, upper+1] = 1-elbow, elbow
    elif kind.startswith('leg'):
        upper = 9 if kind.endswith('R') else 7
        shin = 1-smoothstep(.257, .337, points[:, 1])
        w[:, upper], w[:, upper+1] = 1-shin, shin
    else:
        raise ValueError(kind)
    return w

def mesh_object(name, vertices, faces, surface='team', bone=0, smooth=True):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([coord(p) for p in vertices], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj['surface'] = surface
    obj['detail'] = name
    obj.data.materials.append(bpy.data.materials[surface])
    for p in mesh.polygons:
        p.use_smooth = smooth
    weights = influences(np.array(vertices), bone)
    for index in range(len(REST)):
        ids = np.flatnonzero(weights[:, index] > 0)
        if len(ids):
            group = obj.vertex_groups.new(name=f'joint_{index:02d}')
            for v in ids:
                group.add([int(v)], float(weights[v, index]), 'REPLACE')
    OBJECTS.append(obj)
    return obj

def loft(name, rings, surface='team', bone=0, sides=32, power=1):
    verts, faces = [], []
    for y, x, z, width, depth in rings:
        for i in range(sides):
            a = math.tau*i/sides
            c, s = math.cos(a), math.sin(a)
            verts.append((x+width*math.copysign(abs(c)**power,c), y,
                          z+depth*math.copysign(abs(s)**power,s)))
    faces.append(tuple(range(sides)))
    for j in range(len(rings)-1):
        for i in range(sides):
            a, b = j*sides+i, j*sides+(i+1)%sides
            faces.append((a,a+sides,b+sides,b))
    faces.append(tuple((len(rings)-1)*sides+i for i in reversed(range(sides))))
    return mesh_object(name, verts, faces, surface, bone)

def ellipsoid(name, center, radius, surface='skin', bone=1, sides=28, stacks=15):
    x,y,z = center
    rx,ry,rz = radius
    rings=[]
    for j in range(stacks+1):
        a = -.5*math.pi+math.pi*j/stacks
        c=max(.002,math.cos(a))
        rings.append((y+ry*math.sin(a),x,z,rx*c,rz*c))
    return loft(name,rings,surface,bone,sides)

def box(name, center, size, surface='dark', bone=0, bevel=.008):
    x,y,z=center
    a,b,c=[s*.5 for s in size]
    r=min(bevel,a*.5,b*.5,c*.5)
    return loft(name,[(y-b,x,z,a-r,c-r),(y-b+r,x,z,a,c),
                      (y+b-r,x,z,a,c),(y+b,x,z,a-r,c-r)],
                surface,bone,24,.32)

def tube(name, points, radius, surface='gold', bone=0, sides=8, closed=False):
    points=[Vector(p) for p in points]
    verts,faces=[],[]
    for i,p in enumerate(points):
        prev=points[(i-1)%len(points)] if closed or i else p
        nxt=points[(i+1)%len(points)] if closed or i<len(points)-1 else p
        tangent=(nxt-prev).normalized()
        normal=tangent.cross(Vector((0,0,1)))
        if normal.length<.01:
            normal=tangent.cross(Vector((1,0,0)))
        normal.normalize()
        other=tangent.cross(normal).normalized()
        for j in range(sides):
            a=math.tau*j/sides
            verts.append(tuple(p+radius*(math.cos(a)*normal+math.sin(a)*other)))
    segments=len(points) if closed else len(points)-1
    for i in range(segments):
        for j in range(sides):
            a=i*sides+j
            b=i*sides+(j+1)%sides
            c=((i+1)%len(points))*sides+(j+1)%sides
            d=((i+1)%len(points))*sides+j
            faces.append((a,b,c,d))
    if not closed:
        faces.extend([tuple(reversed(range(sides))),
                      tuple((len(points)-1)*sides+j for j in range(sides))])
    return mesh_object(name,verts,faces,surface,bone)

def sculpt():
    # Oversized jersey: dropped shoulders, curved hem, raised collar and folds.
    loft('Oversized jersey',[(.508,0,.018,.154,.106),(.526,0,.019,.184,.122),
         (.558,0,.018,.186,.121),(.621,0,.013,.174,.113),
         (.716,0,.010,.166,.108),(.823,0,.008,.192,.116),
         (.913,0,.009,.209,.108),(.966,0,.004,.159,.078),
         (.985,0,.002,.074,.063)],'team','torso',40,.79)
    loft('Thick rolled neckline',[(.982,0,.001,.087,.075),
         (.994,0,.001,.085,.073),(1.004,0,.001,.070,.060)],'ivory','torso')
    loft('Double jersey hem',[(.517,0,.019,.185,.124),(.526,0,.019,.188,.125),
         (.537,0,.019,.185,.123)],'team',0,40,.79)
    for sign in [-1,1]:
        tube(f'Jersey drape {sign}',[(sign*.141,.548,-.080),
             (sign*.151,.599,-.078),(sign*.139,.667,-.079)],.004,'team',0)
        tube(f'Shoulder seam {sign}',[(sign*.070,.974,-.035),
             (sign*.134,.951,-.065),(sign*.195,.914,-.053)],.003,'team',0)
    # Neck, original face planes, ears, lips and restrained goatee.
    loft('Neck',[(.951,0,.0,.057,.052),(1.075,0,-.002,.056,.052)],'skin','torso')
    loft('Head with jaw and cheeks',[(1.022,0,-.020,.038,.047),
         (1.045,0,-.024,.062,.061),(1.087,0,-.018,.081,.074),
         (1.152,0,-.005,.089,.084),(1.205,0,.009,.084,.084),
         (1.252,0,.016,.065,.066),(1.273,0,.018,.028,.036)],'skin',1,40,.78)
    loft('Nose bridge and tip',[(1.091,0,-.078,.016,.014),
         (1.105,0,-.091,.022,.026),(1.120,0,-.096,.021,.023),
         (1.165,0,-.079,.012,.016)],'skin',1,20)
    for sign in [-1,1]:
        ellipsoid(f'Ear {sign}',(sign*.088,1.137,.004),(.019,.032,.020))
        ellipsoid(f'Ear inset {sign}',(sign*.098,1.139,-.003),(.009,.019,.011),'skin')
    tube('Lower lip',[(-.024,1.075,-.096),(0,1.071,-.102),(.024,1.075,-.096)],.004,'skin',1)
    tube('Mouth crease',[(-.024,1.082,-.097),(0,1.080,-.103),(.024,1.082,-.097)],.002,'dark',1,6)
    tube('Goatee edge',[(-.027,1.064,-.094),(-.020,1.045,-.087),
         (0,1.039,-.087),(.020,1.045,-.087),(.027,1.064,-.094)],.0035,'dark',1)
    for sign in [-1,1]:
        box(f'Sunglass lens {sign}',(sign*.043,1.172,-.086),(.068,.034,.012),'dark',1,.006)
        tube(f'Glasses temple {sign}',[(sign*.075,1.181,-.078),
             (sign*.094,1.181,-.011),(sign*.087,1.164,.028)],.004,'dark',1)
    tube('Sunglass bridge',[(-.014,1.177,-.095),(0,1.181,-.097),(.014,1.177,-.095)],.004,'gold',1)
    # Rounded cap with a broad bill, panel seams and the pawn's spherical finial.
    loft('Round pawn cap',[(1.213,0,.012,.096,.095),
         (1.236,0,.013,.101,.099),(1.273,0,.017,.091,.088),
         (1.304,0,.021,.062,.060),(1.317,0,.023,.020,.020)],'team',1,40)
    loft('Cap sweatband',[(1.211,0,.011,.097,.096),(1.227,0,.012,.100,.098)],'dark',1,40)
    brim=loft('Tilted flat cap bill',[(1.210,0,-.073,.116,.118),
              (1.218,0,-.077,.123,.124),(1.226,0,-.075,.116,.118)],'team',1,40,.65)
    for v in brim.data.vertices:
        x,y,z=v.co
        a=math.radians(-12)
        v.co.x=x*math.cos(a)-y*math.sin(a)
        v.co.y=x*math.sin(a)+y*math.cos(a)
    for n in range(6):
        a=math.tau*n/6
        pts=[]
        for j in range(12):
            t=j/11
            radius=.091*math.cos(t*math.pi*.5)
            pts.append((radius*math.cos(a),1.237+.078*math.sin(t*math.pi*.5),
                        .015+radius*math.sin(a)))
        tube(f'Cap stitched panel {n}',pts,.0018,'team',1,6)
    ellipsoid('Pawn cap button',(0,1.322,.023),(.013,.011,.013),'team',1,20,10)
    # Chunky curb chain. Alternating links are actual geometry, not a texture.
    for i in range(29):
        t=i/28
        center=Vector((-.096+.192*t,.937-.131*math.sin(math.pi*t),
                       -.116-.025*math.sin(math.pi*t)))
        tangent=Vector((.192,-.131*math.pi*math.cos(math.pi*t),
                        -.025*math.pi*math.cos(math.pi*t))).normalized()
        cross=Vector((0,0,1)) if i%2==0 else Vector((0,1,.45)).normalized()
        other=tangent.cross(cross).normalized()
        pts=[tuple(center+.009*math.cos(a)*tangent+.0065*math.sin(a)*other)
             for a in np.linspace(0,math.tau,12,endpoint=False)]
        tube(f'Chain link {i:02d}',pts,.0025,'gold',0,6,True)
    # Miniature pawn pendant explicitly connects the street character to chess.
    loft('Pendant pawn foot',[(.749,0,-.146,.024,.011),(.757,0,-.146,.027,.012),
         (.765,0,-.146,.016,.010),(.786,0,-.146,.010,.008)],'gold',0,24)
    ellipsoid('Pendant pawn head',(0,.796,-.146),(.015,.015,.011),'gold',0,20,10)
    # Baggy jeans, deep cuffs, pocket piping, belt loops, sneakers and laces.
    loft('Dropped denim waistband',[(.451,0,.014,.154,.105),
         (.509,0,.015,.165,.111),(.557,0,.016,.146,.101)],'denim',0,40,.75)
    loft('Leather belt',[(.526,0,.016,.161,.108),(.550,0,.016,.151,.106)],'dark',0,40,.72)
    box('Gold belt buckle',(0,.540,-.095),(.044,.030,.015),'gold',0,.005)
    for sign,letter in [(-1,'L'),(1,'R')]:
        leg='leg'+letter
        loft('Baggy jeans '+letter,[(.117,sign*.121,.007,.070,.068),
             (.145,sign*.123,.006,.078,.075),(.181,sign*.121,.009,.075,.072),
             (.251,sign*.119,-.006,.077,.082),(.301,sign*.119,-.010,.083,.087),
             (.356,sign*.112,.010,.086,.090),(.428,sign*.105,.016,.091,.096),
             (.501,sign*.092,.017,.087,.090),(.545,sign*.084,.014,.073,.082)],'denim',leg,32,.79)
        for y,z in [(.181,-.049),(.240,-.071),(.351,-.060)]:
            tube(f'Denim fold {letter} {y}',[(sign*.170,y-.012,z),
                 (sign*.123,y,z-.014),(sign*.075,y+.010,z+.003)],.005,'denim',leg)
        tube('Outer jean seam '+letter,[(sign*.194,.155,.004),
             (sign*.199,.279,-.005),(sign*.191,.399,.012),(sign*.169,.499,.027)],.0025,'stitch',leg)
        tube('Back pocket '+letter,[(sign*.055,.466,.107),(sign*.139,.458,.101),
             (sign*.145,.390,.100),(sign*.105,.374,.108),(sign*.058,.397,.108)],.0025,'stitch',leg)
        loft('Thick sneaker sole '+letter,[(.001,sign*.121,-.031,.074,.119),
             (.012,sign*.121,-.034,.080,.128),(.037,sign*.121,-.034,.079,.127)],'ivory',8 if sign<0 else 10,32,.55)
        loft('Sneaker upper '+letter,[(.034,sign*.121,-.035,.073,.121),
             (.060,sign*.121,-.033,.076,.117),(.089,sign*.121,-.022,.068,.105),
             (.122,sign*.121,.007,.058,.070),(.159,sign*.121,.014,.057,.060)],'ivory',8 if sign<0 else 10,32,.65)
        tube('Sneaker side stripe '+letter,[(sign*.193,.056,-.107),
             (sign*.192,.061,-.040),(sign*.184,.082,.045)],.005,'team',8 if sign<0 else 10)
        for j in range(4):
            z=-.127+j*.016
            tube(f'Lace {letter} {j}',[(sign*.121-.025,.079+j*.007,z),
                 (sign*.121+.025,.079+j*.007,z+.009)],.003,'dark',8 if sign<0 else 10,6)
        for j in range(5):
            box(f'Sole tread {letter} {j}',(sign*.121,.016,-.118+j*.045),
                (.156,.010,.008),'dark',8 if sign<0 else 10,.002)
        # Bare forearms below loose sleeves; articulated continuous loft.
        arm='arm'+letter
        loft('Arm '+letter,[(.604,sign*.255,-.052,.031,.031),
             (.635,sign*.257,-.044,.039,.039),(.692,sign*.256,-.026,.041,.045),
             (.747,sign*.252,-.012,.046,.047),(.806,sign*.239,.001,.050,.052),
             (.865,sign*.223,.005,.061,.063),(.916,sign*.205,.005,.067,.065)],'skin',arm,32)
        loft('Dropped sleeve '+letter,[(.800,sign*.241,.002,.069,.068),
             (.815,sign*.241,.002,.075,.073),(.858,sign*.226,.004,.084,.079),
             (.915,sign*.199,.007,.079,.074),(.940,sign*.174,.008,.052,.058)],'team',arm,32,.83)
        loft('Sleeve edge '+letter,[(.799,sign*.241,.001,.071,.070),
             (.812,sign*.241,.001,.076,.075),(.822,sign*.238,.002,.074,.073)],'team',arm)
        ellipsoid('Hand '+letter,(sign*.255,.603,-.058),(.034,.049,.032),'skin',4 if sign<0 else 6)
        if sign>0:
            for j in range(3):
                tube(f'Grip finger {j}',[(.278,.578+j*.016,-.083),
                     (.280,.582+j*.016,-.114),(.252,.582+j*.016,-.120)],.0065,'skin',6)
        for j in range(3):
            tube(f'Knuckle {letter} {j}',[(sign*.277,.582+j*.012,-.078),
                 (sign*.251,.581+j*.012,-.088)],.0023,'skin',4 if sign<0 else 6,6)
    # Wrist bandanna with short tails and a chunky wristwatch on the free hand.
    loft('Wrist bandanna',[(.638,.257,-.043,.043,.043),
         (.661,.257,-.037,.043,.045)],'team',6)
    ellipsoid('Bandanna knot',(.293,.649,-.027),(.014,.016,.017),'team',6,16,10)
    tube('Bandanna tail',[(.293,.647,-.024),(.310,.615,-.012),(.300,.595,-.004)],.008,'team',6)
    box('Watch face',(-.285,.659,-.040),(.016,.031,.033),'gold',4,.004)
    # Stylized pistol, detached geometric detail and rigid weapon weights.
    box('Pistol slide',(.255,.674,-.247),(.049,.048,.222),'dark',2,.006)
    box('Slide top rail',(.255,.701,-.241),(.034,.008,.193),'stitch',2,.002)
    box('Pistol lower frame',(.255,.646,-.216),(.042,.022,.149),'dark',2,.004)
    grip=box('Pistol grip',(.255,.604,-.140),(.043,.095,.044),'dark',2,.006)
    for v in grip.data.vertices:
        v.co.y += (v.co.z-.65)*.23
    for j in range(4):
        box(f'Slide serration {j}',(.281,.674,-.175+j*.010),(.003,.034,.004),'stitch',2,.001)
    for j in range(4):
        box(f'Grip rib {j}',(.278,.580+j*.014,-.144),(.003,.004,.032),'stitch',2,.001)
    tube('Trigger guard',[(.234,.645,-.214),(.234,.612,-.210),
         (.234,.604,-.176),(.234,.637,-.167)],.004,'dark',2,8)
    box('Front sight',(.255,.704,-.333),(.009,.010,.012),'ivory',2,.002)
    box('Muzzle face',(.255,.674,-.360),(.027,.027,.003),'ivory',2,.002)
    box('Muzzle opening',(.255,.674,-.362),(.014,.014,.002),'dark',2,.001)

def export_skin(objects, path):
    positions=[]; normals=[]; indices=[]; all_weights=[]
    for obj in objects:
        mesh=obj.data
        mesh.calc_loop_triangles()
        offset=len(positions)
        for v in mesh.vertices:
            p=obj.matrix_world@v.co
            positions.append((p.x,p.z,-p.y))
            n=obj.matrix_world.to_3x3()@v.normal
            normals.append((n.x,n.z,-n.y))
            weights=np.zeros(len(REST))
            for g in v.groups:
                weights[int(obj.vertex_groups[g.group].name.split('_')[1])]=g.weight
            all_weights.append(weights)
        indices.extend(tuple(offset+i for i in tri.vertices) for tri in mesh.loop_triangles)
    weights=np.array(all_weights)
    ids=np.argsort(-weights,axis=1)[:,:4]
    values=np.take_along_axis(weights,ids,axis=1)
    values/=values.sum(axis=1,keepdims=True)
    with path.open('wb') as f:
        f.write(struct.pack('<4sII',b'BCM1',len(positions),len(indices)*3))
        f.write(REST.astype('<f4').tobytes())
        fmt=struct.Struct('<6f4H4f')
        for p,n,j,w in zip(positions,normals,ids,values):
            f.write(fmt.pack(*p,*n,*j,*w))
        f.write(np.array(indices,dtype='<u4').tobytes())
    return {'vertices':len(positions),'triangles':len(indices),'bytes':path.stat().st_size}

def make_rig():
    arm=bpy.data.armatures.new('BattleChess street pawn • 11 joints')
    obj=bpy.data.objects.new('Street pawn rig',arm)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active=obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    parents=[None,0,6,0,3,0,5,0,7,0,9]
    for i,rest in enumerate(REST):
        bone=arm.edit_bones.new(f'joint_{i:02d}')
        bone.head=coord(rest)
        bone.tail=Vector(bone.head)+Vector((0,0,.07))
    for i,parent in enumerate(parents):
        if parent is not None:
            arm.edit_bones[f'joint_{i:02d}'].parent=arm.edit_bones[f'joint_{parent:02d}']
    bpy.ops.object.mode_set(mode='OBJECT')
    obj.select_set(False)
    obj.show_in_front=True
    for part in OBJECTS:
        mod=part.modifiers.new('BattleChess deformation','ARMATURE')
        mod.object=obj
    return obj

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    for name,color in COLORS.items():
        material=bpy.data.materials.new(name)
        material.diffuse_color=color
        material.use_nodes=True
        bsdf=material.node_tree.nodes.get('Principled BSDF')
        bsdf.inputs['Base Color'].default_value=color
        bsdf.inputs['Metallic'].default_value=.78 if name=='gold' else .12 if name=='dark' else 0
        bsdf.inputs['Roughness'].default_value=.28 if name=='gold' else .67
    sculpt()
    report={}
    for surface in COLORS:
        parts=[o for o in OBJECTS if o['surface']==surface]
        report[surface]=export_skin(parts,OUT/f'{surface}.mesh')
    report['combined']=export_skin(OBJECTS,OUT/'street_pawn.mesh')
    bpy.ops.object.select_all(action='DESELECT')
    for o in OBJECTS:
        o.select_set(True)
    bpy.ops.wm.stl_export(filepath=str(OUT/'street_pawn.stl'),export_selected_objects=True)
    make_rig()
    bpy.context.scene['design']='Original West Coast street pawn; cap, jersey, baggy denim, sneakers, pawn pendant'
    bpy.context.scene['game_coordinates']='Y up, -Z forward; one model unit before BOARD_SCALE'
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'street_pawn.blend'))
    report['rest_joints']=REST.tolist()
    report['muzzle']=MUZZLE
    report['coordinates']='STL and Blender: Z up, +Y forward. BCM1: Y up, -Z forward.'
    report['source']='Original procedural sculpt; no downloaded character geometry.'
    (OUT/'build-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print('STREET_PAWN_BUILT',json.dumps(report['combined']),flush=True)

if __name__=='__main__':
    main()
