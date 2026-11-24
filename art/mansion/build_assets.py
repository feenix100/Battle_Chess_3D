"""Original miniature mansion rook and a stylized in-game Molotov. Blender 5.2.
Coordinates are visual game units, with no physical fabrication dimensions.
"""
import importlib.util
import json
import math
from pathlib import Path
import bpy
from mathutils import Vector

OUT=Path(__file__).resolve().parent
# Reuse the project's existing original geometry/export helpers, not car shapes.
spec=importlib.util.spec_from_file_location('asset_geometry',OUT.parent/'lowrider/build_assets.py')
geo=importlib.util.module_from_spec(spec); spec.loader.exec_module(geo)
BODY=(0,.43,0)
HATCH=(0,.887,.118)
LAUNCH=(0,1.035,-.025)
GATES=[(-.268,.085,-.405),(.268,.085,-.405)]
PIVOTS={'body':BODY,'hatch':HATCH,'gate-0':GATES[0],'gate-1':GATES[1],
        'bottle':(0,0,0),'wick':(0,0,0)}
for i,x in enumerate([-.180,.180]):
    PIVOTS[f'leg-{i}-upper']=(x,.350,0)
    PIVOTS[f'leg-{i}-lower']=(x,.200,0)
    PIVOTS[f'leg-{i}-foot']=(x,.055,0)
SURFACES={
    'paint':((.69,.65,.50,1),.02,.68),
    'stone':((.76,.70,.57,1),.02,.77),
    'ivory':((.91,.86,.73,1),.02,.58),
    'tile':((.46,.15,.075,1),.03,.75),
    'glass':((.045,.12,.17,1),.30,.21),
    'iron':((.033,.043,.050,1),.58,.37),
    'gold':((.77,.49,.10,1),.78,.27),
    'wood':((.18,.082,.040,1),.03,.67),
    'green':((.14,.29,.12,1),.08,.52),
    'bottle-glass':((.12,.34,.22,.70),.10,.20),
    'label':((.88,.78,.52,1),.01,.79),
    'cloth':((.47,.20,.075,1),0,.90),
}

def box(name,c,s,surface='stone',part='body',bevel=.004):
    return geo.box(name,c,s,surface,part,bevel)
def tube(name,points,radius=.004,surface='gold',part='body',sides=8):
    return geo.tube(name,points,radius,surface,part,sides)
def cyl(name,c,r,d,surface='stone',part='body',axis=(0,1,0),sides=24):
    return geo.cylinder(name,c,r,d,surface,part,axis,sides)

def arch(name,origin,width,height,orientation=0,surface='glass',part='body'):
    normals=[Vector((0,0,-1)),Vector((1,0,0)),Vector((0,0,1)),Vector((-1,0,0))]
    normal=normals[orientation]; right=Vector((-normal.z,0,normal.x)); center=Vector(origin)
    def point(u,v,d=0): return tuple(center+right*u+Vector((0,v,0))+normal*d)
    radius=width/2; shoulder=height/2-radius
    outline=[(-radius,-height/2),(radius,-height/2),(radius,shoulder)]
    outline.extend((radius*math.cos(a),shoulder+radius*math.sin(a))
                   for a in [math.pi*i/16 for i in range(1,17)])
    points=[point(u,v,.004) for u,v in outline]
    pane=geo.panel(name,points,surface,part,.006)
    frame=[point(u,v,.013) for u,v in outline]+[point(*outline[0],.013)]
    tube(name+' arch surround',frame,.0065,'ivory',part)
    tube(name+' vertical mullion',[point(0,-height/2,.016),point(0,height/2-.006,.016)],.0025,'gold',part,6)
    tube(name+' transom',[point(-radius,shoulder-.02,.016),point(radius,shoulder-.02,.016)],.0025,'gold',part,6)
    return pane

def hip_roof(name,x,z,width,depth,bottom,height):
    verts=[(x-width/2,bottom,z-depth/2),(x+width/2,bottom,z-depth/2),
           (x+width/2,bottom,z+depth/2),(x-width/2,bottom,z+depth/2),
           (x-width*.25,bottom+height,z),(x+width*.25,bottom+height,z)]
    geo.mesh(name,verts,[(0,1,5,4),(1,2,5),(2,3,4,5),(3,0,4),(3,2,1,0)],'tile','body')
    for i in range(12):
        u=(i+.5)/12
        px=x-width/2+width*u
        tube(name+' terracotta tile seam',[(px,bottom+.003,z-depth/2-.002),
             (x+(px-x)*.52,bottom+height+.003,z)],.002,'tile')
    tube(name+' ridge cap',[(x-width*.25,bottom+height+.006,z),(x+width*.25,bottom+height+.006,z)],.006,'tile')

def mansion():
    # Layered miniature plinth and entrance steps, all inside a board square.
    box('Beveled chess plinth',(0,.024,.005),(.864,.048,.836),'stone',bevel=.035)
    box('Ivory foundation course',(0,.055,.018),(.790,.028,.740),'ivory',bevel=.020)
    box('Lower entry step',(0,.060,-.361),(.348,.023,.160),'stone',bevel=.007)
    box('Upper entry step',(0,.080,-.310),(.294,.022,.112),'ivory',bevel=.006)
    box('Two-storey mansion',(0,.419,.060),(.604,.670,.482),'paint',bevel=.012)
    box('Ground floor sill',(0,.110,.057),(.643,.035,.529),'stone',bevel=.006)
    box('Second storey belt course',(0,.431,.058),(.649,.032,.527),'ivory',bevel=.006)
    box('Roof cornice',(0,.773,.050),(.710,.053,.605),'ivory',bevel=.008)
    box('Roof terrace',(0,.801,.050),(.680,.020,.578),'stone',bevel=.008)
    # Corner pavilions give the house a broad, turreted rook silhouette.
    for side in [-1,1]:
        for z in [-.196,.294]:
            box('Corner pavilion',(side*.280,.434,z),(.126,.694,.126),'paint',bevel=.007)
            box('Pavilion foot',(side*.280,.112,z),(.149,.049,.149),'stone',bevel=.006)
            for y in [.180,.350,.519,.690]:
                box('Corner quoin block',(side*.280,y,z),(.133,.040,.133),'ivory',bevel=.003)
            box('Corner crown cornice',(side*.280,.816,z),(.164,.055,.164),'ivory',bevel=.007)
            box('Rook turret drum',(side*.280,.873,z),(.142,.076,.142),'paint',bevel=.006)
            for sx in [-1,1]:
                for sz in [-1,1]:
                    box('Turret battlement',(side*.280+sx*.052,.947,z+sz*.052),(.049,.074,.049),'ivory',bevel=.005)
        # Arched windows on each exterior side, with stone sills.
        for z in [-.090,.146]:
            for y in [.275,.610]:
                arch('Side arched window',(side*.309,y,z),.113,.192,1 if side>0 else 3)
                box('Side window sill',(side*.318,y-.104,z),(.029,.018,.147),'ivory',bevel=.004)
        for y in [.275,.615]:
            arch('Front pavilion window',(side*.280,y,-.264),.087,.174)
    # Front entrance, pilasters, balcony doors and an ornamental fanlight.
    arch('Grand arched entry',(0,.252,-.188),.165,.287,surface='wood')
    tube('Double entry door seam',[(0,.111,-.211),(0,.319,-.211)],.0025,'gold')
    for side in [-1,1]:
        cyl('Door pull',(side*.027,.232,-.216),.005,.048,'gold')
        for y in [.188,.270]:
            box('Raised door panel',(side*.044,y,-.210),(.054,.057,.008),'wood',bevel=.004)
        arch('Upper arched French door',(side*.063,.609,-.188),.103,.232)
        for x in [side*.198]:
            cyl('Portico column',(x,.265,-.286),.020,.318,'ivory')
            for y in [.113,.410]: cyl('Column capital',(x,y,-.286),.029,.026,'stone')
        box('Upper facade pilaster',(side*.195,.609,-.200),(.037,.248,.037),'ivory',bevel=.003)
    hip_roof('Tiled portico canopy',0,-.247,.480,.235,.429,.064)
    box('Balcony slab',(0,.493,-.230),(.459,.028,.182),'stone',bevel=.006)
    # Balustrade with sculpted turned spindles and gold rail accents.
    for i in range(11):
        x=-.204+i*.0408
        cyl('Balcony spindle',(x,.557,-.314),.006,.110,'ivory',sides=12)
        cyl('Turned baluster bead',(x,.550,-.314),.011,.029,'ivory',sides=12)
    box('Balcony handrail',(0,.620,-.314),(.470,.017,.025),'ivory',bevel=.005)
    tube('Gold balcony rail', [(-.230,.632,-.315),(.230,.632,-.315)],.0025,'gold')
    # Rear facade and luxury estate details visible when the board is flipped.
    for x in [-.198,0,.198]:
        for y in [.272,.612]: arch('Rear arched window',(x,y,.309),.112,.195,2)
    for side in [-1,1]:
        box('Entry lantern backplate',(side*.130,.309,-.205),(.023,.040,.011),'iron',bevel=.004)
        box('Entry lantern glass',(side*.130,.308,-.223),(.021,.032,.023),'gold',bevel=.005)
        cyl('Terracotta planter',(side*.351,.110,-.258),.035,.070,'tile',sides=20)
        geo.mesh('Topiary shrub',[(side*.351+dx,.152+dy,-.258+dz) for dx,dy,dz in
            [(-.035,0,-.035),(.035,0,-.035),(.035,0,.035),(-.035,0,.035),
             (-.020,.068,-.020),(.020,.068,-.020),(.020,.068,.020),(-.020,.068,.020)]],
            [(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7),(3,2,1,0)],'green','body')
    # Rook parapet: crenellations around a flat rooftop terrace.
    for z in [-.248,.347]:
        box('Roof parapet',(0,.865,z),(.538,.101,.029),'paint',bevel=.004)
        box('Parapet coping',(0,.920,z),(.552,.018,.036),'ivory',bevel=.004)
        for x in [-.218,-.109,0,.109,.218]:
            box('Rook roof merlon',(x,.957,z),(.060,.065,.038),'ivory',bevel=.004)
    for x in [-.321,.321]:
        box('Side parapet',(x,.863,.050),(.029,.095,.421),'paint',bevel=.004)
        box('Side roof coping',(x,.918,.050),(.036,.018,.433),'ivory',bevel=.004)
        for z in [-.120,.050,.220]: box('Side roof merlon',(x,.953,z),(.038,.060,.061),'ivory',bevel=.004)
    # A recessed roof hatch disguises the animated release port as architecture.
    for side in [-1,1]:
        box('Roof hatch side rim',(side*.105,.846,.014),(.025,.068,.235),'iron',bevel=.004)
        box('Roof hatch end rim',(0,.846,.014+side*.105),(.190,.068,.025),'iron',bevel=.004)
    box('Recessed dark launch floor',(0,.817,.014),(.190,.012,.190),'iron',bevel=.004)
    box('Roof hatch lid',(0,.892,.014),(.225,.015,.225),'tile','hatch',.007)
    for x in [-.078,.078]:
        tube('Roof hatch tile seam',[(x,.902,-.090),(x,.902,.113)],.002,'stone','hatch')
    box('Hatch gold handle',(0,.908,-.057),(.044,.008,.009),'gold','hatch',.003)
    box('Chimney',(-.166,.891,.220),(.075,.158,.075),'stone',bevel=.006)
    box('Chimney cap',(-.166,.976,.220),(.100,.024,.100),'ivory',bevel=.004)
    for y in [.843,.881,.919,.954]:
        tube('Chimney masonry course',[(-.206,y,.180),(-.206,y,.260),(-.126,y,.260),(-.126,y,.180)],.0015,'wood')
    # Gated frontage: separate hinges, individual rods, scrolls and spear finials.
    for side in [-1,1]:
        box('Estate gate pier',(side*.288,.193,-.405),(.067,.239,.068),'paint',bevel=.006)
        box('Pier cap',(side*.288,.319,-.405),(.086,.024,.086),'ivory',bevel=.006)
        cyl('Gold pier ball',(side*.288,.346,-.405),.019,.028,'gold',sides=20)
    for i,side in enumerate([-1,1]):
        part=f'gate-{i}'; left=-.260 if side<0 else .004
        right=-.004 if side<0 else .260
        for y in [.112,.246]: tube('Gate crossrail',[(left,y,-.413),(right,y,-.413)],.0035,'iron',part)
        for j in range(6):
            x=left+.018+j*.043
            top=.265+.022*math.sin(math.pi*j/5)
            tube('Gate wrought iron bar',[(x,.097,-.413),(x,top,-.413)],.0028,'iron',part,6)
            geo.mesh('Gate gold spear',[(x-.006,top,-.413),(x,top+.017,-.413),
                 (x+.006,top,-.413),(x,top-.006,-.420)],[(0,1,3),(1,2,3),(2,0,3),(0,2,1)],'gold',part)
        mid=(left+right)/2
        for mirror in [-1,1]:
            points=[]
            for j in range(21):
                a=math.pi*.25+math.pi*1.6*j/20; radius=.030*(1-j/30)
                points.append((mid+mirror*(.025+radius*math.cos(a)),.190+radius*math.sin(a),-.416))
            tube('Gate curled scroll',points,.0023,'gold',part,6)

def walking_legs():
    # Concealed in the foundation at rest; stone armor and iron pistons deploy.
    for i,x in enumerate([-.180,.180]):
        upper=f'leg-{i}-upper';lower=f'leg-{i}-lower';foot=f'leg-{i}-foot'
        box('Mansion upper leg',(x,.277,0),(.092,.146,.098),'stone',upper,.010)
        box('Upper leg painted armor',(x,.278,-.052),(.078,.111,.012),'paint',upper,.006)
        cyl('Hip iron joint',(x,.345,0),.045,.108,'iron',upper,(1,0,0),20)
        cyl('Hip gold pin',(x-.056,.345,0),.020,.008,'gold',upper,(1,0,0),16)
        box('Mansion lower leg',(x,.127,.001),(.074,.142,.081),'iron',lower,.008)
        box('Shin stone armor',(x,.128,-.044),(.080,.115,.024),'stone',lower,.006)
        cyl('Knee iron joint',(x,.197,0),.040,.100,'iron',lower,(1,0,0),20)
        cyl('Knee gold cap',(x,.197,-.042),.025,.012,'gold',lower,(0,0,1),20)
        for side in [-1,1]:
            cyl('Leg piston',(x+side*.041,.132,.026),.007,.112,'gold',lower,sides=12)
            cyl('Piston housing',(x+side*.041,.162,.026),.011,.052,'iron',lower,sides=12)
        box('Mansion walking foot',(x,.033,-.035),(.132,.058,.191),'stone',foot,.012)
        box('Foot painted toe',(x,.037,-.127),(.111,.038,.017),'paint',foot,.005)
        box('Foot iron sole',(x,.011,-.035),(.140,.019,.199),'iron',foot,.005)
        for j in range(4):box('Foot sole tread',(x,.006,-.105+j*.047),(.132,.010,.018),'gold',foot,.002)

def bottle():
    profile=[(-.090,.024),(-.084,.032),(.020,.032),(.045,.025),(.060,.013),(.096,.012),(.105,.014)]
    sides=28; verts=[]; faces=[]
    for y,r in profile:
        for i in range(sides):
            a=math.tau*i/sides; verts.append((r*math.cos(a),y,r*math.sin(a)))
    for j in range(len(profile)-1):
        for i in range(sides):
            a=j*sides+i; b=j*sides+(i+1)%sides; faces.append((a,b,b+sides,a+sides))
    faces.extend([tuple(reversed(range(sides))),tuple((len(profile)-1)*sides+i for i in range(sides))])
    geo.mesh('Stylized green bottle',verts,faces,'bottle-glass','bottle')
    cyl('Bottle base rim',(0,-.082,0),.033,.010,'green','bottle')
    # A raised label and short cloth end are visual props only.
    box('Bottle paper label',(0,-.025,-.032),(.045,.064,.003),'label','bottle',.004)
    for y in [-.044,-.025,-.006]: tube('Label gold stripes',[(-.016,y,-.035),(.016,y,-.035)],.001,'gold','bottle',6)
    tube('Cloth wick',[(0,.098,0),(.004,.120,.002),(.014,.140,-.001)],.006,'cloth','wick',8)

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.preferences.filepaths.save_version=0
    for name,(color,metal,rough) in SURFACES.items():
        material=bpy.data.materials.new(name); material.diffuse_color=color; material.use_nodes=True
        node=material.node_tree.nodes['Principled BSDF']; node.inputs['Base Color'].default_value=color
        node.inputs['Metallic'].default_value=metal; node.inputs['Roughness'].default_value=rough
    mansion(); walking_legs(); bottle(); bpy.context.view_layer.update()
    report={'parts':{},'pivots':PIVOTS,'launch':LAUNCH,'source':'Original mansion and visual game prop geometry.'}
    entries=[]
    for part,pivot in PIVOTS.items():
        for surface in SURFACES:
            objects=[o for o in geo.OBJECTS if o['part']==part and o['surface']==surface]
            if not objects: continue
            name=f'{part}-{surface}.mesh'
            report['parts'][name]=geo.export_mesh(objects,OUT/name,pivot)
            entries.append(f'    ("{part}", "{surface}", include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/art/mansion/{name}"))),')
    house=[o for o in geo.OBJECTS if o['part'] not in ['bottle','wick']]
    projectile=[o for o in geo.OBJECTS if o['part'] in ['bottle','wick']]
    report['mansion']=geo.export_mesh(house,OUT/'mansion.mesh')
    report['molotov']=geo.export_mesh(projectile,OUT/'molotov.mesh')
    geo.export_stl(house,OUT/'mansion-rook.stl'); geo.export_stl(projectile,OUT/'molotov.stl')
    leg_parts=[o for o in house if o['part'].startswith('leg-')]
    geo.export_stl(leg_parts,OUT/'mansion-legs.stl')
    building=[o for o in house if not o['part'].startswith('leg-')]
    for obj in building:obj.location.z+=.350
    bpy.context.view_layer.update();geo.export_stl(house,OUT/'mansion-walking.stl')
    for obj in building:obj.location.z-=.350
    bpy.context.view_layer.update()
    (OUT/'assets.rs').write_text('pub(super) const PARTS: &[(&str, &str, &[u8])] = &[\n'+'\n'.join(entries)+'\n];\n',encoding='utf-8')
    groups={}
    for part,pivot in PIVOTS.items():
        group=bpy.data.objects.new(part,None); bpy.context.collection.objects.link(group)
        group.location=geo.xyz(pivot); group.empty_display_size=.025; groups[part]=group
    bpy.context.view_layer.update()
    for obj in geo.OBJECTS:
        matrix=obj.matrix_world.copy(); obj.parent=groups[obj['part']]; obj.matrix_world=matrix
    # Keep the projectile visibly beside the mansion in the editable source.
    for part in ['bottle','wick']: groups[part].location=geo.xyz((.60,.15,0))
    bpy.context.scene['axes']='STL: Z up, +Y forward. Runtime BCM2: Y up, -Z forward.'
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'mansion-rook.blend'))
    (OUT/'build-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print('MANSION_BUILT',json.dumps({'mansion':report['mansion'],'molotov':report['molotov']}),flush=True)

if __name__=='__main__': main()
