"""Original GTA Street police bishop, editable skin, STLs and game mesh layers.
Visual game units only. Run with Blender 5.2 in background mode.
"""
import importlib.util
import json
import math
from pathlib import Path
import bpy
import numpy as np

OUT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('bishop_geometry',OUT.parent/'street_pawn/build_mesh.py')
geo=importlib.util.module_from_spec(spec);spec.loader.exec_module(geo)
REST=np.array([(0,.585,.008),(0,1.065,0),(.248,.678,-.073),
    (-.197,.967,0),(-.236,.787,-.012),(.197,.967,0),(.236,.787,-.012),
    (-.092,.582,.010),(-.105,.306,-.006),(.092,.582,.010),(.105,.306,-.006)],dtype=float)
MUZZLE=(.248,.716,-.302)
COLORS={
    'team':((.19,.30,.43,1),.03,.70),
    'skin':((.57,.32,.19,1),0,.74),
    'navy':((.037,.061,.099,1),.02,.84),
    'dark':((.018,.024,.032,1),.13,.50),
    'gold':((.76,.49,.11,1),.76,.28),
    'steel':((.45,.53,.59,1),.78,.28),
    'yellow':((.90,.61,.025,1),.10,.38),
    'visor':((.021,.062,.086,1),.32,.18),
}

def weights(points,kind):
    w=np.zeros((len(points),11))
    if isinstance(kind,int):w[:,kind]=1
    elif kind=='torso':
        head=geo.smoothstep(1.025,1.095,points[:,1]);w[:,0]=1-head;w[:,1]=head
    elif kind.startswith('arm'):
        upper=5 if kind.endswith('R')else 3
        lower=1-geo.smoothstep(.747,.825,points[:,1]);w[:,upper]=1-lower;w[:,upper+1]=lower
    elif kind.startswith('leg'):
        upper=9 if kind.endswith('R')else 7
        lower=1-geo.smoothstep(.271,.349,points[:,1]);w[:,upper]=1-lower;w[:,upper+1]=lower
    else:raise ValueError(kind)
    return w

def loft(name,rings,surface='team',bone=0,sides=32,power=1):return geo.loft(name,rings,surface,bone,sides,power)
def box(name,c,size,surface='dark',bone=0,bevel=.006):return geo.box(name,c,size,surface,bone,bevel)
def tube(name,points,r=.003,surface='steel',bone=0,sides=8,closed=False):return geo.tube(name,points,r,surface,bone,sides,closed)
def oval(name,c,r,surface='skin',bone=1,sides=28,stacks=15):return geo.ellipsoid(name,c,r,surface,bone,sides,stacks)

def shield(name,center,width,height,surface='gold',bone=0):
    x,y,z=center;outline=[(-.5,.5),(.5,.5),(.45,-.15),(0,-.55),(-.45,-.15)]
    vertices=[(x+u*width,y+v*height,z+d)for d in [-.003,.003]for u,v in outline]
    faces=[tuple(reversed(range(5))),tuple(range(5,10))]+[(i,(i+1)%5,5+(i+1)%5,5+i)for i in range(5)]
    return geo.mesh_object(name,vertices,faces,surface,bone)

def sculpt_officer():
    # Tailored uniform, collar, tie, epaulettes, chest pockets and police badge.
    loft('Tailored patrol shirt',[(.573,0,.012,.137,.090),(.626,0,.010,.151,.101),
        (.722,0,.006,.150,.101),(.825,0,.002,.160,.104),(.913,0,.002,.183,.103),
        (.976,0,.001,.189,.090),(1.018,0,0,.118,.070),(1.040,0,0,.066,.055)],'team','torso',40,.81)
    for side in [-1,1]:
        box('Chest pocket',(side*.078,.875,-.101),(.098,.074,.018),'team',0,.008)
        box('Pocket flap',(side*.078,.914,-.115),(.105,.024,.010),'navy',0,.004)
        oval('Pocket button',(side*.078,.909,-.124),(.005,.005,.003),'gold',0,14,8)
        geo.mesh_object('Pointed shirt collar',[(side*.017,1.041,-.053),(side*.083,1.014,-.073),
            (side*.046,.968,-.102),(side*.012,1.002,-.083)],[(0,1,2,3)],'navy','torso')
        box('Shoulder epaulette',(side*.168,.979,.005),(.074,.014,.076),'navy',5 if side>0 else 3,.004)
        oval('Epaulette button',(side*.143,.987,-.019),(.005,.003,.005),'gold',5 if side>0 else 3,12,8)
    geo.mesh_object('Uniform necktie',[(-.015,1.010,-.085),(.015,1.010,-.085),(.023,.856,-.119),
        (0,.825,-.123),(-.023,.856,-.119)],[(0,1,2,3,4)],'navy',0)
    box('Tie knot',(0,1.002,-.084),(.027,.026,.012),'navy',0,.004)
    shield('Chest shield badge',(-.075,.945,-.112),.039,.051)
    shield('Badge blue inset',(-.075,.946,-.117),.024,.034,'navy')
    oval('Badge center seal',(-.075,.948,-.121),(.008,.008,.003),'gold',0,16,8)
    box('Officer name plate',(.073,.947,-.103),(.074,.013,.007),'gold',0,.003)
    for y in [.761,.810,.955]:oval('Shirt button',(0,y,-.109),(.004,.005,.003),'gold',0,12,8)
    # Neck and a clean-shaven adult face, brows, lip and aviator visor.
    loft('Neck',[(1.003,0,0,.054,.049),(1.120,0,-.001,.055,.051)],'skin','torso')
    loft('Sculpted face',[(1.082,0,-.022,.035,.044),(1.109,0,-.024,.058,.061),
        (1.156,0,-.017,.075,.073),(1.222,0,-.003,.084,.083),
        (1.278,0,.009,.080,.080),(1.312,0,.014,.058,.062),(1.329,0,.016,.022,.030)],'skin',1,40,.82)
    loft('Nose',[(1.157,0,-.083,.014,.015),(1.174,0,-.094,.021,.024),
        (1.193,0,-.094,.018,.022),(1.230,0,-.080,.011,.013)],'skin',1,20)
    tube('Mouth crease',[(-.024,1.140,-.097),(0,1.136,-.102),(.024,1.140,-.097)],.0022,'dark',1,6)
    tube('Lower lip',[(-.021,1.130,-.095),(0,1.126,-.099),(.021,1.130,-.095)],.0035,'skin',1)
    for side in [-1,1]:
        oval('Ear',(side*.083,1.214,.003),(.017,.029,.019))
        box('Aviator lens',(side*.041,1.245,-.081),(.067,.039,.013),'visor',1,.008)
        tube('Aviator gold frame',[(side*.013,1.261,-.091),(side*.069,1.261,-.083),
            (side*.074,1.232,-.085),(side*.020,1.229,-.094)],.0025,'gold',1)
        tube('Aviator temple',[(side*.074,1.253,-.078),(side*.092,1.250,.010)],.003,'dark',1)
    tube('Glasses bridge',[(-.014,1.251,-.096),(0,1.256,-.099),(.014,1.251,-.096)],.003,'gold',1)
    # A tapered motorcycle-police helmet with a true diagonal bishop notch.
    helmet=loft('Notched bishop police helmet',[(1.257,0,.008,.094,.092),(1.288,0,.011,.103,.099),
        (1.344,0,.014,.091,.088),(1.405,0,.017,.077,.074),(1.465,0,.020,.055,.055),
        (1.508,0,.022,.024,.031),(1.527,0,.023,.004,.014)],'team',1,48)
    bpy.ops.mesh.primitive_cube_add(size=1,location=geo.coord((.030,1.440,.015)))
    cutter=bpy.context.object;cutter.dimensions=(.020,.270,.180)
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    cutter.rotation_euler.y=.55
    bpy.context.view_layer.objects.active=helmet
    cut=helmet.modifiers.new('Diagonal chess bishop notch','BOOLEAN');cut.operation='DIFFERENCE';cut.object=cutter
    bpy.ops.object.modifier_apply(modifier=cut.name);bpy.data.objects.remove(cutter,do_unlink=True)
    helmet.vertex_groups.clear();helmet.vertex_groups.new(name='joint_01').add(list(range(len(helmet.data.vertices))),1,'REPLACE')
    loft('Helmet dark lower band',[(1.255,0,.008,.095,.093),(1.274,0,.009,.101,.098)],'dark',1,40)
    shield('Helmet police badge',(0,1.312,-.091),.040,.052,'gold',1)
    shield('Helmet badge inset',(0,1.313,-.096),.024,.034,'navy',1)
    for side in [-1,1]:
        tube('Helmet side gold trim',[(side*.080,1.275,.043),(side*.097,1.287,-.005),
            (side*.081,1.303,-.049)],.0025,'gold',1)
        tube('Helmet chin strap',[(side*.083,1.233,.013),(side*.073,1.123,-.024),
            (side*.037,1.096,-.052)],.004,'dark',1)
    # Straight tailored trousers, pressed seams and boots with separate sole detail.
    loft('Uniform pelvis',[(.494,0,.014,.132,.094),(.552,0,.012,.148,.100),
        (.611,0,.010,.139,.095)],'team',0,36,.78)
    loft('Duty belt',[(.584,0,.010,.148,.101),(.632,0,.008,.151,.102)],'dark',0,40,.72)
    box('Belt buckle',(0,.607,-.100),(.050,.034,.013),'gold',0,.004)
    for side,letter in [(-1,'L'),(1,'R')]:
        bone='leg'+letter;shin=8 if side<0 else 10;arm='arm'+letter;hand=4 if side<0 else 6
        loft('Tailored trouser '+letter,[(.165,side*.106,.010,.054,.058),(.220,side*.107,.004,.056,.063),
            (.305,side*.105,-.006,.062,.067),(.379,side*.102,.004,.067,.074),
            (.472,side*.097,.012,.073,.082),(.562,side*.087,.013,.074,.083),
            (.603,side*.079,.010,.060,.074)],'team',bone,32,.84)
        tube('Pressed trouser crease',[(side*.105,.225,-.060),(side*.105,.308,-.074),
            (side*.099,.453,-.071),(side*.091,.550,-.070)],.0023,'dark',bone)
        tube('Uniform side piping',[(side*.162,.220,.007),(side*.166,.317,.002),
            (side*.164,.433,.010),(side*.152,.555,.018)],.003,'gold',bone)
        loft('Patrol boot '+letter,[(.025,side*.107,-.026,.061,.105),(.062,side*.107,-.025,.064,.103),
            (.108,side*.107,-.005,.054,.068),(.176,side*.106,.009,.052,.056),
            (.254,side*.105,.009,.055,.059)],'dark',shin,32,.62)
        loft('Boot sole '+letter,[(.000,side*.107,-.027,.061,.107),(.015,side*.107,-.029,.066,.111),
            (.031,side*.107,-.028,.065,.109)],'dark',shin,32,.58)
        loft('Boot cuff '+letter,[(.235,side*.105,.010,.058,.061),(.258,side*.105,.010,.059,.062)],'dark',shin,28)
        for j in range(5):box('Boot sole tread',(side*.107,.012,-.112+j*.042),(.130,.007,.008),'steel',shin,.001)
        for j in range(4):tube('Boot laces',[(side*.107-.025,.105+j*.012,-.079+j*.012),
            (side*.107+.025,.105+j*.012,-.073+j*.012)],.002,'steel',shin,6)
        loft('Bare forearm '+letter,[(.630,side*.247,-.043,.029,.030),(.671,side*.245,-.033,.035,.038),
            (.730,side*.242,-.021,.039,.041),(.787,side*.236,-.012,.043,.045),
            (.850,side*.223,-.002,.049,.052),(.926,side*.207,.001,.059,.061),
            (.974,side*.191,.001,.049,.054)],'skin',arm,32)
        loft('Short uniform sleeve '+letter,[(.856,side*.223,.001,.060,.062),
            (.876,side*.221,.001,.068,.067),(.937,side*.203,.002,.076,.069),
            (.975,side*.181,.001,.055,.054)],'team',arm,32,.82)
        loft('Sleeve cuff '+letter,[(.853,side*.223,0,.062,.064),(.873,side*.222,0,.067,.068)],'navy',arm)
        box('Shoulder patch '+letter,(side*.270,.921,-.001),(.009,.063,.053),'navy',5 if side>0 else 3,.005)
        for j in range(2):tube('Patch gold rank stripe',[(side*.276,.917+j*.012,-.018),
            (side*.278,.924+j*.012,0),(side*.276,.917+j*.012,.018)],.0025,'gold',5 if side>0 else 3)
        oval('Hand '+letter,(side*.247,.632,-.047),(.031,.046,.029),'skin',hand)
        for j in range(3):tube('Hand knuckle '+letter,[(side*.271,.612+j*.012,-.067),
            (side*.245,.611+j*.012,-.076)],.0023,'skin',hand,6)
    # Duty radio, antenna, microphone and belt equipment.
    box('Belt radio',(-.180,.675,.023),(.055,.111,.055),'dark',0,.008)
    tube('Radio antenna',[(-.185,.727,.025),(-.191,.807,.023)],.0035,'dark',0)
    for j in range(4):box('Radio speaker ridge',(-.211,.680+j*.012,.009),(.003,.003,.031),'steel',0,.001)
    box('Shoulder microphone',(-.147,.960,-.072),(.034,.040,.018),'dark',0,.005)
    tube('Radio coiled lead',[(-.170,.720,.010),(-.177,.780,-.045),(-.162,.860,-.091),(-.147,.944,-.086)],.003,'dark',0)
    box('Closed belt holster',(.178,.587,.018),(.053,.116,.062),'dark',0,.008)
    for side in [-1,1]:box('Belt utility pouch',(side*.116,.615,-.077),(.052,.074,.046),'dark',0,.006)
    for i in range(2):
        center=(-.160+i*.042,.561,.096)
        points=[(center[0]+.025*math.cos(a),center[1]+.027*math.sin(a),center[2])
            for a in np.linspace(0,math.tau,24,endpoint=False)]
        tube('Handcuff ring',points,.0035,'steel',0,8,True)
    tube('Cuff short link',[(-.136,.563,.096),(-.142,.563,.104)],.004,'steel',0)
    # A chunky fictional yellow taser; every part follows the rigid weapon joint.
    first=len(geo.OBJECTS)
    box('Taser yellow housing',(.248,.707,-.202),(.056,.057,.174),'yellow',2,.007)
    box('Taser dark upper rail',(.248,.739,-.199),(.042,.011,.156),'dark',2,.003)
    box('Taser grip',(.248,.637,-.133),(.045,.107,.047),'dark',2,.006)
    box('Taser yellow grip inset',(.273,.642,-.135),(.004,.071,.025),'yellow',2,.003)
    box('Taser cartridge face',(.248,.707,-.292),(.055,.049,.010),'dark',2,.004)
    for side in [-1,1]:
        box('Twin taser muzzle',(.248+side*.012,.716,-.299),(.010,.013,.003),'steel',2,.002)
    box('Taser status light',(.277,.716,-.182),(.003,.011,.023),'visor',2,.001)
    tube('Taser trigger guard',[(.226,.683,-.204),(.226,.654,-.201),(.226,.649,-.169),(.226,.684,-.162)],.004,'dark',2)
    for obj in geo.OBJECTS[first:]:obj['equipment']='taser'
    for j in range(3):tube('Right grip finger',[(.270,.612+j*.014,-.069),(.275,.617+j*.014,-.107),
        (.248,.617+j*.014,-.112)],.0055,'skin',6)

def main():
    OUT.mkdir(parents=True,exist_ok=True);bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.preferences.filepaths.save_version=0
    geo.REST=REST;geo.influences=weights
    for name,(color,metal,rough)in COLORS.items():
        m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True
        node=m.node_tree.nodes['Principled BSDF'];node.inputs['Base Color'].default_value=color
        node.inputs['Metallic'].default_value=metal;node.inputs['Roughness'].default_value=rough
    sculpt_officer();bpy.context.view_layer.update()
    report={'parts':{},'rest_joints':REST.tolist(),'muzzle':MUZZLE,'source':'Original police bishop geometry.'}
    for surface in COLORS:
        objects=[o for o in geo.OBJECTS if o['surface']==surface]
        report['parts'][surface]=geo.export_skin(objects,OUT/f'{surface}.mesh')
    report['combined']=geo.export_skin(geo.OBJECTS,OUT/'police-bishop.mesh')
    bpy.ops.object.select_all(action='DESELECT')
    for obj in geo.OBJECTS:obj.select_set(True)
    bpy.ops.wm.stl_export(filepath=str(OUT/'police-bishop.stl'),export_selected_objects=True)
    bpy.ops.object.select_all(action='DESELECT')
    for obj in geo.OBJECTS:obj.select_set(obj.get('equipment')=='taser')
    bpy.ops.wm.stl_export(filepath=str(OUT/'taser.stl'),export_selected_objects=True)
    geo.make_rig()
    lines=['pub(super) const REST: [Vec3; 11] = [']
    lines.extend('    Vec3::new('+','.join(repr(float(v))for v in p)+'),'for p in REST)
    lines+= ['];','pub(super) const MUZZLE: Vec3 = Vec3::new('+','.join(repr(float(v))for v in MUZZLE)+');']
    (OUT/'anchors.rs').write_text('\n'.join(lines)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'police-bishop.blend'))
    (OUT/'build-report.json').write_text(json.dumps(report,indent=2))
    print('POLICE_BISHOP_BUILT',json.dumps(report['combined']),flush=True)

if __name__=='__main__':main()
