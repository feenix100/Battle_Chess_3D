"""Original lowrider chess knight and trunk drone, in game units. Blender 5.2.
Builds editable parts, runtime BCM2 meshes and independent car/drone STL exports.
"""
import json
import math
import struct
from pathlib import Path
import bpy
import bmesh
import numpy as np
from mathutils import Vector

OUT=Path(__file__).resolve().parent
BODY=(0,.245,0)
HINGE=(0,.337,.272)
BAY=(0,.235,.423)
WHEELS=[(-.294,.137,-.355),(.294,.137,-.355),(-.294,.137,.355),(.294,.137,.355)]
ARMS=[(-.025,.014,-.022),(.025,.014,-.022),(-.025,.014,.022),(.025,.014,.022)]
MOTORS=[(-.091,.026,-.071),(.091,.026,-.071),(-.091,.026,.071),(.091,.026,.071)]
PIVOTS={'body':BODY,'trunk':HINGE,'drone':(0,0,0)}
PIVOTS.update({f'wheel-{i}':p for i,p in enumerate(WHEELS)})
PIVOTS.update({f'arm-{i}':p for i,p in enumerate(ARMS)})
PIVOTS.update({f'rotor-{i}':p for i,p in enumerate(MOTORS)})
SURFACES={
    'paint':((.18,.37,.26,1),.45,.28),
    'chrome':((.68,.74,.78,1),.88,.19),
    'gold':((.73,.46,.10,1),.78,.26),
    'rubber':((.016,.020,.025,1),.04,.83),
    'interior':((.065,.055,.048,1),0,.88),
    'ivory':((.91,.87,.75,1),.1,.39),
    'glass':((.075,.17,.23,.74),.35,.16),
    'red':((.7,.025,.035,1),.2,.25),
    'cyan':((.06,.7,.8,1),.25,.21),
}
OBJECTS=[]
def xyz(p): return (p[0],-p[2],p[1])
def game(p): return (p[0],p[2],-p[1])

def tag(obj,name,surface,part):
    obj.name=name
    obj['surface']=surface; obj['part']=part
    obj.data.materials.append(bpy.data.materials[surface])
    for poly in obj.data.polygons: poly.use_smooth=True
    OBJECTS.append(obj)
    return obj

def mesh(name,vertices,faces,surface='paint',part='body'):
    data=bpy.data.meshes.new(name)
    data.from_pydata([xyz(p) for p in vertices],[],faces); data.update()
    bm=bmesh.new(); bm.from_mesh(data); bmesh.ops.recalc_face_normals(bm,faces=bm.faces); bm.to_mesh(data); bm.free()
    obj=bpy.data.objects.new(name,data); bpy.context.collection.objects.link(obj)
    return tag(obj,name,surface,part)

def box(name,center,size,surface='chrome',part='body',bevel=.004):
    bpy.ops.mesh.primitive_cube_add(size=1,location=xyz(center))
    obj=bpy.context.object; obj.dimensions=(size[0],size[2],size[1])
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    if bevel:
        mod=obj.modifiers.new('Soft manufactured edges','BEVEL'); mod.width=bevel; mod.segments=3
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return tag(obj,name,surface,part)

def cylinder(name,center,radius,depth,surface='chrome',part='body',axis=(1,0,0),sides=32):
    bpy.ops.mesh.primitive_cylinder_add(vertices=sides,radius=radius,depth=depth,location=xyz(center))
    obj=bpy.context.object
    obj.rotation_mode='QUATERNION'; obj.rotation_quaternion=Vector((0,0,1)).rotation_difference(Vector(xyz(axis)))
    return tag(obj,name,surface,part)

def torus(name,center,major,minor,surface='chrome',part='body',axis='x',steps=48,sides=8):
    vertices=[]; faces=[]
    x,y,z=center
    for i in range(steps):
        a=math.tau*i/steps
        for j in range(sides):
            b=math.tau*j/sides; r=major+minor*math.cos(b); h=minor*math.sin(b)
            if axis=='x': vertices.append((x+h,y+r*math.cos(a),z+r*math.sin(a)))
            elif axis=='y': vertices.append((x+r*math.cos(a),y+h,z+r*math.sin(a)))
            else: vertices.append((x+r*math.cos(a),y+r*math.sin(a),z+h))
    for i in range(steps):
        for j in range(sides):
            faces.append((i*sides+j,((i+1)%steps)*sides+j,
                          ((i+1)%steps)*sides+(j+1)%sides,i*sides+(j+1)%sides))
    return mesh(name,vertices,faces,surface,part)

def tube(name,points,radius,surface='chrome',part='body',sides=8):
    pts=[Vector(p) for p in points]; vertices=[]; faces=[]
    for i,p in enumerate(pts):
        d=(pts[min(i+1,len(pts)-1)]-pts[max(0,i-1)]).normalized()
        n=d.cross(Vector((0,1,0)))
        if n.length<.01: n=d.cross(Vector((1,0,0)))
        n.normalize(); b=d.cross(n).normalized()
        for j in range(sides):
            a=math.tau*j/sides; vertices.append(tuple(p+radius*(n*math.cos(a)+b*math.sin(a))))
    for i in range(len(pts)-1):
        for j in range(sides):
            a=i*sides+j; b=i*sides+(j+1)%sides
            faces.append((a,b,b+sides,a+sides))
    faces.extend([tuple(reversed(range(sides))),tuple((len(pts)-1)*sides+j for j in range(sides))])
    return mesh(name,vertices,faces,surface,part)

def panel(name,points,surface='glass',part='body',thickness=.003):
    obj=mesh(name,points,[tuple(range(len(points)))],surface,part)
    bpy.context.view_layer.objects.active=obj
    mod=obj.modifiers.new('Panel thickness','SOLIDIFY'); mod.thickness=thickness
    bpy.ops.object.modifier_apply(modifier=mod.name)
    return obj

def cut(obj,cutter):
    bpy.context.view_layer.objects.active=obj
    mod=obj.modifiers.new('Designed opening','BOOLEAN'); mod.operation='DIFFERENCE'; mod.solver='EXACT'; mod.object=cutter
    bpy.ops.object.modifier_apply(modifier=mod.name)
    OBJECTS.remove(cutter); bpy.data.objects.remove(cutter,do_unlink=True)

def horse(name,side=0):
    # A sculpted side silhouette: ears, curved mane, forehead and forward muzzle.
    profile=[(.352,-.377),(.370,-.344),(.431,-.341),(.482,-.356),
             (.523,-.384),(.551,-.389),(.539,-.408),(.548,-.432),
             (.517,-.430),(.499,-.452),(.483,-.490),(.463,-.495),
             (.454,-.472),(.465,-.439),(.417,-.408),(.365,-.404)]
    if side:
        profile=[(.205+(y-.352)*.56,.025+(z+.410)*.56) for y,z in profile]
    x=side*.322
    thickness=.006 if side else .008
    vertices=[(x+sign*thickness,y,z) for sign in [-1,1] for y,z in profile]
    n=len(profile)
    faces=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]
    faces.extend((i,(i+1)%n,n+(i+1)%n,n+i) for i in range(n))
    return mesh(name,vertices,faces,'gold' if side else 'chrome')

def sculpt_car():
    sections=[(-.583,.240,.170,.271),(-.547,.294,.124,.312),(-.435,.314,.108,.333),
              (-.220,.318,.103,.338),(.130,.318,.102,.344),(.430,.309,.110,.329),
              (.550,.281,.132,.304),(.580,.244,.172,.275)]
    vertices=[]
    for z,w,low,high in sections:
        vertices.extend([(-w*.90,low,z),(-w,low+.036,z),(-w,high-.025,z),(-w*.93,high,z),
                         (w*.93,high,z),(w,high-.025,z),(w,low+.036,z),(w*.90,low,z)])
    faces=[tuple(reversed(range(8)))]
    for j in range(len(sections)-1):
        for i in range(8): faces.append((j*8+i,j*8+(i+1)%8,(j+1)*8+(i+1)%8,(j+1)*8+i))
    faces.append(tuple((len(sections)-1)*8+i for i in range(8)))
    body=mesh('Long sculpted coupe body',vertices,faces)
    for i,(x,y,z) in enumerate(WHEELS):
        cut(body,cylinder('Wheel arch cutter',(x,y,z),.149,.20,'rubber',axis=(1,0,0),sides=48))
    cut(body,box('Recessed trunk opening',(0,.370,.422),(.512,.43,.282),'interior',bevel=.013))
    bpy.context.view_layer.objects.active=body
    mod=body.modifiers.new('Body edge softness','BEVEL'); mod.width=.0045; mod.segments=3
    bpy.ops.object.modifier_apply(modifier=mod.name)
    # Hood creases and gold pinstripe panel, with chess knight ornament.
    for side in [-1,1]:
        tube('Hood pinstripe',[(side*.188,.339,-.514),(side*.212,.344,-.405),(side*.202,.346,-.260)],.0018,'gold')
        tube('Continuous beltline',[(side*.285,.296,-.533),(side*.321,.316,-.225),
             (side*.322,.316,.245),(side*.292,.287,.532)],.0035,'chrome')
        tube('Lower rocker trim',[(side*.313,.135,-.190),(side*.321,.129,.195)],.005,'chrome')
        box('Door handle',(side*.329,.300,.160),(.012,.012,.049),'chrome',bevel=.003)
        tube('Door shut line',[(side*.320,.314,-.192),(side*.324,.190,-.160),
             (side*.322,.156,.181),(side*.322,.307,.208)],.0018,'rubber')
        horse('Door knight crest',side)
        box('Wing mirror',(side*.357,.363,-.157),(.060,.027,.040),'chrome',bevel=.009)
        box('Mirror glass',(side*.358,.366,-.137),(.043,.016,.003),'glass',bevel=.003)
        tube('Mirror support',[(side*.298,.339,-.153),(side*.343,.353,-.153)],.005)
    cylinder('Ornament round plinth',(0,.345,-.399),.027,.009,'chrome',axis=(0,1,0))
    horse('Chess knight hood ornament')
    # Cabin, roof, chrome window surrounds, seats and dashboard.
    roof=box('Low coupe roof',(0,.523,.028),(.443,.034,.340),'paint',bevel=.022)
    for side in [-1,1]:
        tube('Front windshield pillar',[(side*.267,.342,-.242),(side*.207,.514,-.135)],.009)
        tube('Rear sail pillar',[(side*.204,.514,.191),(side*.266,.342,.287)],.012,'paint')
        tube('Chrome roof gutter',[(side*.223,.518,-.132),(side*.228,.530,.025),(side*.220,.517,.190)],.004)
        tube('Window lower chrome',[(side*.274,.345,-.238),(side*.278,.345,.281)],.005)
        panel('Open cabin side glazing',[(side*.270,.352,-.227),(side*.210,.506,-.130),
             (side*.211,.506,.178),(side*.265,.352,.273)],'glass')
        tube('Window divider',[(side*.236,.349,.100),(side*.215,.512,.095)],.004)
    panel('Wide sloped windshield',[(-.264,.345,-.238),(.264,.345,-.238),
          (.205,.507,-.135),(-.205,.507,-.135)])
    panel('Rear glass',[(-.206,.509,.190),(.206,.509,.190),(.263,.346,.284),(-.263,.346,.284)])
    for z,y,w in [(-.239,.345,.264),(-.135,.508,.206),(.192,.509,.206),(.285,.347,.263)]:
        tube('Chrome glass edge',[(-w,y,z),(w,y,z)],.0045)
    box('Dashboard',(0,.349,-.179),(.460,.050,.074),'interior',bevel=.012)
    for side in [-1,1]:
        box('Bucket seat base',(side*.111,.266,.001),(.160,.060,.148),'interior',bevel=.021)
        seat=box('Bucket seat back',(side*.111,.358,.092),(.160,.160,.039),'interior',bevel=.019)
        for j in range(3):
            tube('Seat stitching',[(side*.111-.05,.315+j*.024,.069),(side*.111+.05,.315+j*.024,.069)],.0015,'gold')
    torus('Steering wheel',(-.117,.368,-.145),.038,.004,'rubber',axis='z',steps=28)
    # Chrome face: headlights, egg-crate grille, overriders and license plates.
    box('Front bumper',(0,.174,-.594),(.585,.038,.038),'chrome',bevel=.012)
    box('Rear bumper',(0,.176,.590),(.567,.038,.035),'chrome',bevel=.012)
    box('Grille shadow',(0,.262,-.590),(.332,.074,.014),'rubber',bevel=.006)
    for i in range(13):
        box('Vertical grille bar',(-.153+i*.0255,.262,-.601),(.005,.067,.006),'chrome',bevel=.0015)
    for j in [-1,0,1]: box('Horizontal grille bar',(0,.262+j*.025,-.604),(.323,.004,.006),'chrome',bevel=.001)
    for side in [-1,1]:
        for j in [0,1]:
            x=side*(.199+j*.052)
            cylinder('Headlight chrome bezel',(x,.271,-.585),.025,.016,'chrome',axis=(0,0,1),sides=32)
            cylinder('Headlight lens',(x,.271,-.596),.020,.004,'ivory',axis=(0,0,1),sides=24)
            for stripe in [-1,0,1]: tube('Lens ridges',[(x+stripe*.007,.255,-.600),(x+stripe*.007,.287,-.600)],.001,'chrome')
        box('Rear lamp chrome',(side*.209,.258,.584),(.091,.039,.013),'chrome',bevel=.007)
        box('Ruby rear lamp',(side*.209,.258,.594),(.078,.027,.006),'red',bevel=.005)
        for j in [-1,1]: box('Rear lamp divider',(side*.209+j*.018,.258,.599),(.003,.027,.002),'chrome',bevel=.001)
        cylinder('Twin exhaust',(side*.219,.114,.590),.015,.047,'chrome',axis=(0,0,1),sides=20)
        cylinder('Exhaust dark opening',(side*.219,.114,.615),.011,.002,'rubber',axis=(0,0,1),sides=20)
        box('Bumper overrider',(side*.197,.182,-.615),(.021,.074,.022),'chrome',bevel=.006)
    box('Rear license plate',(0,.239,.589),(.122,.043,.006),'ivory',bevel=.004)
    bpy.ops.object.text_add(location=xyz((0,.233,.595)))
    text=bpy.context.object; text.data.body='KN1GHT'; text.data.align_x='CENTER'; text.data.size=.022
    text.data.extrude=.0004
    # Text lies in Blender XY with normal +Z; face toward game rear (+Z).
    text.rotation_euler=(math.pi/2,0,0)
    bpy.ops.object.convert(target='MESH'); tag(bpy.context.object,'KN1GHT plate lettering','rubber','body')
    # A recessed carpet-lined drone bay, rails, and decorative hydraulic gear.
    box('Trunk carpet',(0,.164,.421),(.493,.020,.260),'interior',bevel=.008)
    for side in [-1,1]:
        box('Carpeted trunk wheelhouse',(side*.225,.224,.423),(.071,.112,.251),'interior',bevel=.018)
        box('Trunk deployment rail',(side*.116,.182,.424),(.014,.017,.234),'chrome',bevel=.003)
        cylinder('Hydraulic reservoir',(side*.207,.217,.440),.022,.107,'chrome',axis=(0,0,1),sides=20)
        tube('Colored trunk hose',[(side*.207,.211,.385),(side*.184,.194,.355),
             (side*.154,.190,.370)],.004,'red' if side<0 else 'cyan')
    box('Trunk lid',(0,.342,.421),(.510,.017,.288),'paint','trunk',.010)
    box('Trunk inner lining',(0,.330,.421),(.472,.008,.250),'interior','trunk',.006)
    for side in [-1,1]:
        tube('Trunk pinstripe',[(side*.202,.352,.309),(side*.220,.352,.413),(side*.202,.352,.540)],.0018,'gold','trunk')
        tube('Trunk hinge arm',[(side*.185,.331,.280),(side*.185,.316,.317)],.006,'chrome','trunk')
    cylinder('Trunk keyhole',(0,.341,.544),.007,.004,'chrome','trunk',axis=(0,0,1),sides=16)
    bpy.context.view_layer.update()
    for obj in [o for o in OBJECTS if o['part']=='trunk']:
        matrix=obj.matrix_world.copy(); inverse=matrix.inverted()
        for v in obj.data.vertices:
            p=matrix@v.co
            p.z-=.10*(-p.y-HINGE[2])
            v.co=inverse@p
    for i,(x,y,z) in enumerate(WHEELS):
        part=f'wheel-{i}'; side=1 if x>0 else -1
        torus('Low profile tire',(x,y,z),.104,.029,'rubber',part,steps=56,sides=12)
        outer=x+side*.030
        torus('Whitewall insert',(outer,y,z),.101,.011,'ivory',part,steps=56)
        torus('Polished wheel rim',(outer+side*.004,y,z),.081,.006,'chrome',part,steps=56)
        cylinder('Wire rim shadow',(x+side*.025,y,z),.079,.011,'rubber',part,sides=40)
        for j in range(24):
            a=math.tau*j/24
            for layer in [-1,1]:
                b=a+layer*.35
                tube('Cross laced gold wire',[(outer+side*.005,y+.019*math.cos(a),z+.019*math.sin(a)),
                     (outer,y+.076*math.cos(b),z+.076*math.sin(b))],.0017,'gold',part,6)
        cylinder('Chrome center knockoff',(outer+side*.007,y,z),.022,.015,'chrome',part,sides=24)
        for j in range(3):
            a=math.tau*j/3
            tube('Three ear knockoff',[(outer+side*.019,y,z),
                 (outer+side*.020,y+.030*math.cos(a),z+.030*math.sin(a))],.003,'gold',part)
        for j in range(28):
            a=math.tau*j/28
            tube('Tire tread shoulder',[(x-.012,y+.131*math.cos(a),z+.131*math.sin(a)),
                 (x+.012,y+.131*math.cos(a+.025),z+.131*math.sin(a+.025))],.0015,'rubber',part,5)

def sculpt_drone():
    box('Drone armored core',(0,0,0),(.090,.039,.075),'rubber','drone',.012)
    box('Drone upper shell',(0,.021,0),(.079,.014,.064),'chrome','drone',.009)
    box('Drone cyan status stripe',(0,.030,-.010),(.044,.003,.009),'cyan','drone',.001)
    cylinder('Drone camera housing',(0,-.014,-.043),.016,.024,'chrome','drone',axis=(0,0,1),sides=24)
    cylinder('Drone camera lens',(0,-.014,-.057),.011,.004,'glass','drone',axis=(0,0,1),sides=24)
    for i,(pivot,motor) in enumerate(zip(ARMS,MOTORS)):
        part=f'arm-{i}'
        tube('Folding drone boom',[pivot,motor],.008,'chrome',part)
        cylinder('Drone motor',motor,.014,.027,'rubber',part,axis=(0,1,0),sides=20)
        torus('Drone rotor guard',(motor[0],motor[1]+.008,motor[2]),.039,.0025,'chrome',part,axis='y',steps=28,sides=6)
        for angle in [0,math.pi/2]:
            direction=Vector((math.cos(angle)*.035,0,math.sin(angle)*.035))
            tube('Rotor guard strut',[tuple(Vector(motor)-direction),tuple(Vector(motor)+direction)],.0018,'rubber',part,6)
        box('Spinning drone blade',(motor[0],motor[1]+.014,motor[2]),(.073,.003,.009),'rubber',f'rotor-{i}',.001)
        box('Rotor gold tip',(motor[0]+.030,motor[1]+.016,motor[2]),(.010,.002,.009),'gold',f'rotor-{i}',.001)

def export_mesh(objects,path,pivot=(0,0,0)):
    positions=[]; normals=[]; triangles=[]
    for obj in objects:
        obj.data.calc_loop_triangles(); offset=len(positions)
        matrix=obj.matrix_world
        normal_matrix=matrix.to_3x3().inverted().transposed()
        for v in obj.data.vertices:
            p=game(matrix@v.co); n=game((normal_matrix@v.normal).normalized())
            positions.append(tuple(p[i]-pivot[i] for i in range(3))); normals.append(n)
        triangles.extend(tuple(offset+i for i in tri.vertices) for tri in obj.data.loop_triangles)
    with path.open('wb') as f:
        f.write(struct.pack('<4sIII',b'BCM2',len(positions),len(triangles)*3,0))
        for p,n in zip(positions,normals): f.write(struct.pack('<6f',*p,*n))
        f.write(np.asarray(triangles,dtype='<u4').tobytes())
    return {'vertices':len(positions),'triangles':len(triangles),'bytes':path.stat().st_size}

def export_stl(objects,path):
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects: obj.select_set(True)
    bpy.ops.wm.stl_export(filepath=str(path),export_selected_objects=True)

def parent_editable():
    bpy.ops.object.select_all(action='DESELECT')
    groups={}
    for part,pivot in PIVOTS.items():
        obj=bpy.data.objects.new(part,None); bpy.context.collection.objects.link(obj)
        obj.location=xyz(pivot); obj.empty_display_size=.035; groups[part]=obj
    bpy.context.view_layer.update()
    for obj in OBJECTS:
        matrix=obj.matrix_world.copy(); obj.parent=groups[obj['part']]; obj.matrix_world=matrix
    # Drone groups form one editable deployable assembly, initially parked in bay.
    for i in range(4):
        arm=groups[f'arm-{i}']; rotor=groups[f'rotor-{i}']
        matrix=rotor.matrix_world.copy(); rotor.parent=arm; rotor.matrix_world=matrix
        matrix=arm.matrix_world.copy(); arm.parent=groups['drone']; arm.matrix_world=matrix
    groups['drone'].location=xyz(BAY)
    return groups

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.preferences.filepaths.save_version=0
    for name,(color,metal,rough) in SURFACES.items():
        m=bpy.data.materials.new(name); m.diffuse_color=color; m.use_nodes=True
        p=m.node_tree.nodes['Principled BSDF']; p.inputs['Base Color'].default_value=color
        p.inputs['Metallic'].default_value=metal; p.inputs['Roughness'].default_value=rough
    sculpt_car(); sculpt_drone(); bpy.context.view_layer.update()
    report={'parts':{},'pivots':PIVOTS,'bay':BAY,'source':'Original procedural geometry; no downloaded vehicle or drone assets.'}
    entries=[]
    for part in PIVOTS:
        for surface in SURFACES:
            objects=[o for o in OBJECTS if o['part']==part and o['surface']==surface]
            if not objects: continue
            name=f'{part}-{surface}.mesh'
            report['parts'][name]=export_mesh(objects,OUT/name,PIVOTS[part])
            entries.append(f'    ("{part}", "{surface}", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/art/lowrider/{name}"))),')
    car=[o for o in OBJECTS if o['part'] in ['body','trunk'] or o['part'].startswith('wheel-')]
    drone=[o for o in OBJECTS if o not in car]
    report['car']=export_mesh(car,OUT/'lowrider.mesh')
    report['drone']=export_mesh(drone,OUT/'drone.mesh')
    export_stl(car,OUT/'lowrider-knight.stl'); export_stl(drone,OUT/'trunk-drone.stl')
    (OUT/'assets.rs').write_text('pub(super) const PARTS: &[(&str, &str, &[u8])] = &[\n'+'\n'.join(entries)+'\n];\n',encoding='utf-8')
    report['runtime_meshes']=len(entries)
    parent_editable()
    bpy.context.scene['axes']='STL/Blender: Z up, +Y forward. BCM2: Y up, -Z forward.'
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'lowrider-knight.blend'))
    (OUT/'build-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print('LOWRIDER_BUILT',json.dumps({'car':report['car'],'drone':report['drone']}),flush=True)

if __name__=='__main__': main()
