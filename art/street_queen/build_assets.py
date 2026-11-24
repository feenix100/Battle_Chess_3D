"""Original adult street-fashion queen, long ponytail, pleated skirt and mini Uzi.
Game Y-up/-Z-forward maps to Blender Z-up/+Y-forward. Visual game units only.
"""
import importlib.util,json,math,struct
from pathlib import Path
import bpy,numpy as np
from mathutils import Matrix,Vector
OUT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('queen_geometry',OUT.parent/'street_pawn/build_mesh.py')
geo=importlib.util.module_from_spec(spec);spec.loader.exec_module(geo)
REST=np.array([(0,.605,.010),(0,1.060,0),(.235,.690,-.075),
 (-.182,.985,0),(-.250,.820,.020),(.182,.985,0),(.225,.805,-.010),
 (-.085,.605,.012),(-.094,.305,-.005),(.085,.605,.012),(.094,.305,-.005),(0,.675,.010)],dtype=float)
GRIP=REST[2].copy();CARRY=-.35
R=np.array([[1,0,0],[0,math.cos(CARRY),-math.sin(CARRY)],[0,math.sin(CARRY),math.cos(CARRY)]])
MUZZLE=GRIP+R@np.array([0,.057,-.326]);EJECT=GRIP+R@np.array([.048,.065,-.056])
COLORS={
 'team':((.33,.22,.42,1),.02,.74),'skin':((.56,.31,.20,1),0,.72),
 'hair':((.025,.013,.009,1),.04,.55),'dark':((.017,.024,.033,1),.15,.43),
 'gold':((.79,.52,.15,1),.76,.27),'steel':((.31,.36,.40,1),.75,.29),
 'rose':((.45,.035,.092,1),.12,.46),'ivory':((.82,.80,.73,1),.02,.65),
}
def weights(points,kind):
 w=np.zeros((len(points),len(REST)))
 if isinstance(kind,int):w[:,kind]=1
 elif kind=='torso':
  h=geo.smoothstep(1.025,1.098,points[:,1]);w[:,0]=1-h;w[:,1]=h
 elif kind.startswith('leg'):
  upper=7 if kind.endswith('L')else 9;shin=1-geo.smoothstep(.268,.343,points[:,1]);w[:,upper]=1-shin;w[:,upper+1]=shin
 else:raise ValueError(kind)
 return w
def loft(n,p,s='team',b=0,sides=32,power=1):return geo.loft(n,p,s,b,sides,power)
def oval(n,c,r,s='skin',b=1,sides=28,stacks=15):return geo.ellipsoid(n,c,r,s,b,sides,stacks)
def box(n,c,size,s='dark',b=0,bevel=.004):return geo.box(n,c,size,s,b,bevel)
def tube(n,p,r=.003,s='gold',b=0,sides=8,closed=False):return geo.tube(n,p,r,s,b,sides,closed)
def crown(n,c,width=.060,height=.040,s='gold',b=0):
 x,y,z=c;outline=[(-.5,-.4),(.5,-.4),(.56,.40),(.26,.08),(0,.65),(-.26,.08),(-.56,.40)]
 v=[(x+a*width,y+h*height,z+d)for d in [-.003,.003]for a,h in outline];k=len(outline)
 return geo.mesh_object(n,v,[tuple(reversed(range(k))),tuple(range(k,2*k))]+[(i,(i+1)%k,k+(i+1)%k,k+i)for i in range(k)],s,b)
def muscle(n,a,b,profile,bone):
 axis=b-a;axis/=np.linalg.norm(axis);side=np.cross(axis,[0,0,1]);side/=np.linalg.norm(side);front=np.cross(axis,side)
 vertices=[];faces=[];sides=24
 for t,r in profile:
  c=a+(b-a)*t;vertices.extend([tuple(c+side*r*math.cos(v)+front*r*.88*math.sin(v))for v in np.linspace(0,math.tau,sides,endpoint=False)])
 faces.append(tuple(reversed(range(sides))))
 for row in range(len(profile)-1):
  for i in range(sides):
   j=(i+1)%sides;faces.append((row*sides+i,row*sides+j,(row+1)*sides+j,(row+1)*sides+i))
 faces.append(tuple((len(profile)-1)*sides+i for i in range(sides)))
 return geo.mesh_object(n,vertices,faces,'skin',bone)
def sculpt_body():
 # Adult proportions, cropped T-shirt, visible midriff and a lined pleated skirt.
 loft('Queen cropped shirt',[(.785,0,.006,.127,.081),(.805,0,.005,.141,.087),(.865,0,.002,.146,.092),
  (.932,0,.004,.167,.100),(.977,0,.003,.180,.088),(1.019,0,.002,.111,.068),(1.042,0,.001,.058,.049)],'team','torso',44,.88)
 loft('Crop top rolled hem',[(.784,0,.006,.132,.084),(.797,0,.005,.144,.091),(.811,0,.005,.141,.089)],'team',0,40,.83)
 loft('Shirt bound neckline',[(1.034,0,.001,.067,.054),(1.045,0,.001,.066,.054),(1.051,0,.001,.057,.046)],'ivory','torso',32)
 loft('Waist and midriff',[(.646,0,.012,.134,.088),(.687,0,.010,.123,.079),(.737,0,.008,.115,.077),(.803,0,.007,.126,.081)],'skin',0,40,.90)
 tube('Subtle navel crease',[(0,.728,-.070),(0,.737,-.073)],.002,'skin',0,7)
 for side in [-1,1]:
  tube('Crop top drape '+str(side),[(side*.088,.815,-.067),(side*.101,.840,-.068),(side*.099,.873,-.071)],.0025,'team',0)
  box('Sleeve shoulder seam '+str(side),(side*.178,.979,.004),(.046,.013,.061),'team',3 if side<0 else 5)
 crown('Cropped shirt crown',(0,.913,-.102),.062,.042)
 oval('Crown shirt jewel',(0,.919,-.109),(.006,.008,.004),'rose',0,16,10)
 # Sixteen folds in one continuous skirt, with a dark opaque inner lining.
 vertices=[];faces=[];sides=64
 for y,width,depth in [(.402,.218,.144),(.421,.214,.141),(.510,.190,.126),(.595,.157,.108),(.673,.135,.092)]:
  for i in range(sides):
   a=math.tau*i/sides;fold=1.0+(.035 if i%4 in [0,1]else-.035)
   vertices.append((width*math.cos(a)*fold,y,.012+depth*math.sin(a)*fold))
 for row in range(4):
  for i in range(sides):
   j=(i+1)%sides;faces.append((row*sides+i,(row+1)*sides+i,(row+1)*sides+j,row*sides+j))
 faces.append(tuple(reversed([4*sides+i for i in range(sides)])))
 skirt=geo.mesh_object('Continuous pleated skirt',vertices,faces,'team',11)
 solid=skirt.modifiers.new('Fabric thickness','SOLIDIFY');solid.thickness=.004;bpy.context.view_layer.objects.active=skirt;bpy.ops.object.modifier_apply(modifier=solid.name)
 loft('Opaque skirt lining',[(.407,0,.012,.200,.132),(.495,0,.012,.173,.115),(.668,0,.012,.128,.088)],'dark',11,40,.88)
 loft('Skirt waistband',[(.653,0,.012,.138,.096),(.681,0,.010,.135,.094)],'team',0,40,.88)
 loft('Thin skirt belt',[(.658,0,.012,.140,.098),(.672,0,.011,.140,.097)],'dark',0,40,.86)
 box('Queen belt buckle',(0,.665,-.088),(.039,.023,.010),'gold',0)
 for i in range(16):
  a=math.tau*(i+.5)/16
  tube('Skirt pressed pleat '+str(i),[(.135*math.cos(a),.660,.012+.093*math.sin(a)),(.183*math.cos(a),.525,.012+.123*math.sin(a)),(.218*math.cos(a),.411,.012+.145*math.sin(a))],.0018,'team',11,6)
 tube('Skirt stitched hem',[(.219*math.cos(a),.408,.012+.145*math.sin(a))for a in np.linspace(0,math.tau,80,endpoint=False)],.002,'gold',11,7,True)
 # Legs, street boots, laces, soles, and side zippers.
 for side,letter in [(-1,'L'),(1,'R')]:
  leg='leg'+letter;shin=8 if side<0 else 10
  loft('Queen leg '+letter,[(.130,side*.096,.007,.035,.039),(.221,side*.095,.006,.039,.044),(.305,side*.094,-.003,.042,.049),
   (.387,side*.090,.006,.054,.059),(.482,side*.085,.014,.063,.065),(.607,side*.079,.016,.054,.060)],'skin',leg,32,.95)
  loft('Street boot '+letter,[(.025,side*.097,-.021,.053,.101),(.061,side*.097,-.022,.056,.104),(.103,side*.096,.002,.044,.068),
   (.167,side*.095,.014,.041,.049),(.190,side*.095,.015,.040,.046)],'dark',shin,32,.72)
  loft('Street boot sole '+letter,[(.006,side*.097,-.025,.058,.108),(.030,side*.097,-.025,.059,.109),(.040,side*.097,-.024,.056,.106)],'ivory',shin,32,.66)
  loft('Boot cuff '+letter,[(.171,side*.095,.015,.043,.051),(.188,side*.095,.015,.043,.051)],'team',shin,28,.85)
  for j in range(4):
   y=.080+j*.023
   tube('Boot cross lace '+letter+str(j),[(side*.097-.022,y,-.069),(side*.097+.022,y+.016,-.066)],.0024,'ivory',shin)
   tube('Boot cross lace reverse '+letter+str(j),[(side*.097+.022,y,-.069),(side*.097-.022,y+.016,-.066)],.0024,'ivory',shin)
   for sign in [-1,1]:oval('Boot eyelet '+letter+str(j)+str(sign),(side*.097+sign*.027,y,-.064),(.0035,.0035,.003),'gold',shin,10,7)
  tube('Side boot zipper '+letter,[(side*.140,.073,.010),(side*.137,.166,.016)],.0023,'gold',shin)
  box('Boot zipper pull '+letter,(side*.139,.165,.017),(.005,.015,.009),'gold',shin,.001)
 # Neck, mature face, understated makeup and shaped brows.
 loft('Queen neck',[(1.015,0,.002,.048,.044),(1.119,0,0,.049,.045)],'skin','torso',32)
 loft('Adult queen face',[(1.075,0,-.025,.030,.036),(1.099,0,-.029,.051,.052),(1.144,0,-.021,.070,.065),
  (1.203,0,-.008,.081,.076),(1.266,0,.002,.080,.078),(1.316,0,.008,.064,.067),(1.347,0,.010,.027,.032)],'skin',1,40,.88)
 loft('Queen nose',[(1.165,0,-.071,.012,.014),(1.182,0,-.082,.018,.021),(1.198,0,-.082,.014,.019),(1.245,0,-.064,.010,.011)],'skin',1,20)
 for side in [-1,1]:
  oval('Queen ear '+str(side),(side*.082,1.208,.010),(.015,.028,.017),'skin',1,24,14)
  oval('Queen eye white '+str(side),(side*.035,1.242,-.069),(.020,.007,.0035),'ivory',1,24,12)
  oval('Queen iris '+str(side),(side*.035,1.242,-.073),(.0055,.006,.002),'hair',1,18,10)
  tube('Queen upper eyelid '+str(side),[(side*.016,1.247,-.069),(side*.035,1.252,-.073),(side*.054,1.248,-.067)],.0028,'hair',1,7)
  tube('Queen eyebrow '+str(side),[(side*.014,1.270,-.066),(side*.035,1.279,-.070),(side*.057,1.269,-.064)],.004,'hair',1,8)
  tube('Gold hoop earring '+str(side),[(side*.091,1.171+.026*math.cos(a),.005+.024*math.sin(a))for a in np.linspace(0,math.tau,32,endpoint=False)],.0032,'gold',1,9,True)
  oval('Earring stud '+str(side),(side*.091,1.198,.007),(.004,.005,.004),'rose',1,14,8)
 tube('Queen upper lip',[(-.021,1.151,-.080),(-.008,1.156,-.086),(0,1.152,-.087),(.008,1.156,-.086),(.021,1.151,-.080)],.0035,'rose',1,9)
 tube('Queen lower lip',[(-.019,1.148,-.081),(0,1.143,-.086),(.019,1.148,-.081)],.004,'rose',1,9)
 # Long ponytail reaches the middle of the back; cap and swept bangs frame the face.
 loft('Queen hair crown',[(1.273,0,.022,.084,.077),(1.315,0,.024,.087,.080),(1.358,0,.027,.073,.070),(1.391,0,.030,.036,.039)],'hair',1,44)
 for side in [-1,1]:
  tube('Swept temple hair '+str(side),[(side*.015,1.377,-.014),(side*.068,1.338,-.034),(side*.084,1.277,-.008),(side*.087,1.233,.026)],.018,'hair',1,14)
 loft('Long tied ponytail',[(.690,.038,.183,.011,.014),(.770,.044,.202,.023,.024),(.875,.040,.216,.036,.035),(.973,.038,.217,.045,.043),
  (1.068,.027,.202,.050,.047),(1.162,.014,.178,.050,.048),(1.258,.003,.145,.047,.043),(1.338,0,.117,.035,.034)],'hair',1,32)
 oval('Ponytail tie',(0,1.327,.124),(.041,.020,.040),'team',1,24,14)
 for j in range(-3,4):
  tube('Long ponytail strand '+str(j),[(j*.008,1.307,.150),(j*.012,1.182,.214),(j*.012+.024,1.033,.259),(j*.007+.041,.899,.249),(j*.004+.043,.762,.225)],.0018,'dark',1,6)
 # Five-point tiara makes the chess role readable above the ponytail.
 tube('Tiara headband',[(.089*math.cos(a),1.342,.021+.077*math.sin(a))for a in np.linspace(math.pi,math.tau,42)],.006,'gold',1,10)
 for i in range(5):
  x=(i-2)*.030;top=1.420+(2-abs(i-2))*.023;z=-.050+abs(i-2)*.008
  tube('Tiara prong '+str(i),[(x,1.343,z),(x,top,z-.006)],.0045,'gold',1,9)
  oval('Tiara point jewel '+str(i),(x,top,z-.006),(.010,.014,.008),'rose'if i==2 else'gold',1,18,12)
 crown('Tiara center crest',(0,1.371,-.064),.082,.041,'gold',1)
 oval('Tiara central jewel',(0,1.387,-.070),(.009,.012,.004),'rose',1,18,12)
 # Jewelry, a small hip bag, chain strap, and belt details.
 tube('Queen necklace',[(-.048,1.040,-.038),(-.035,1.006,-.061),(0,.990,-.076),(.035,1.006,-.061),(.048,1.040,-.038)],.0028,'gold',0,9)
 oval('Necklace pendant',(0,.984,-.081),(.010,.014,.004),'rose',0,18,10)
 box('Queen hip bag',(-.164,.624,.041),(.067,.099,.087),'dark',0,.010)
 box('Bag flap',(-.171,.659,.033),(.066,.031,.091),'team',0,.006)
 oval('Bag clasp',(-.202,.645,.030),(.004,.006,.006),'gold',0,12,8)
 for i in range(13):
  t=i/12;c=(-.146-.027*math.sin(math.pi*t),.675-.109*math.sin(math.pi*t),-.031+.115*t)
  tube('Bag chain link '+str(i),[(c[0]+.007*math.cos(a),c[1]+.006*math.sin(a),c[2])for a in np.linspace(0,math.tau,12,endpoint=False)],.002,'gold',0,6,True)

def sculpt_arms():
 for side,upper,lower in [(-1,3,4),(1,5,6)]:
  shoulder=REST[upper];elbow=REST[lower];wrist=np.array([-.160,.718,-.075])if side<0 else GRIP
  muscle('Queen upper arm '+str(side),shoulder,elbow,[(0,.045),(.3,.049),(.75,.037),(1,.032)],upper)
  oval('Queen shoulder '+str(side),tuple(shoulder),(.056,.052,.056),'skin',upper,28,16)
  muscle('Queen forearm '+str(side),elbow,wrist,[(0,.032),(.3,.035),(.75,.028),(1,.023)],lower)
  oval('Queen elbow '+str(side),tuple(elbow),(.033,.033,.034),'skin',lower,22,12)
  # Short sleeves cover only the upper-arm section.
  muscle('Short sleeve '+str(side),shoulder,shoulder+(elbow-shoulder)*.40,[(0,.052),(.5,.056),(1,.053)],upper)
  sleeve=geo.OBJECTS[-1];sleeve['surface']='team';sleeve.data.materials.clear();sleeve.data.materials.append(bpy.data.materials['team'])
  if side<0:
   oval('Queen hand on hip',tuple(wrist),(.029,.036,.022),'skin',lower,24,14)
   for j in range(4):
    x=wrist[0]+(j-1.5)*.011
    tube('Queen left finger '+str(j),[(x,.705,-.082),(x,.683,-.092),(x,.678,-.078)],.005,'skin',lower,8)
    oval('Queen fingernail '+str(j),(x,.685,-.096),(.004,.005,.0015),'rose',lower,10,6)
   tube('Queen thumb',[(-.181,.735,-.078),(-.190,.713,-.095),(-.175,.701,-.097)],.006,'skin',lower)
   for j in range(3):
    c=elbow+(wrist-elbow)*(.77+j*.055);axis=(wrist-elbow)/np.linalg.norm(wrist-elbow);u=np.cross(axis,[0,0,1]);u/=np.linalg.norm(u);v=np.cross(axis,u)
    tube('Queen bangle '+str(j),[tuple(c+.030*u*math.cos(a)+.027*v*math.sin(a))for a in np.linspace(0,math.tau,24,endpoint=False)],.0023,'gold',lower,8,True)

def sculpt_uzi():
 first=len(geo.OBJECTS)
 box('Mini Uzi receiver',(0,.055,-.045),(.080,.087,.235),'dark',2,.007)
 box('Uzi top cover',(0,.102,-.045),(.076,.014,.224),'steel',2,.003)
 box('Uzi receiver side',(.042,.058,-.055),(.005,.058,.174),'steel',2,.003)
 box('Uzi open ejection port',(.046,.064,-.057),(.004,.023,.056),'dark',2,.001)
 box('Uzi bolt in port',(.049,.064,-.062),(.003,.013,.037),'gold',2,.001)
 box('Uzi charging handle',(0,.122,-.046),(.027,.026,.032),'dark',2,.003)
 for side in [-1,1]:
  for i in range(4):box('Uzi cooling slot '+str(side)+str(i),(side*.042,.066,-.103-i*.014),(.004,.012,.008),'dark',2,.001)
  oval('Uzi receiver pin '+str(side),(side*.046,.042,.029),(.003,.005,.005),'gold',2,12,8)
 tube('Short Uzi barrel',[(0,.057,-.160),(0,.057,-.309)],.020,'dark',2,24)
 for z in [-.184,-.207,-.230]:tube('Barrel collar '+str(z),[(0,.057,z-.005),(0,.057,z+.005)],.024,'steel',2,24)
 # Open annular muzzle and a recessed dark bore.
 vertices=[];faces=[]
 for z,r in [(-.309,.022),(-.325,.022),(-.325,.014),(-.300,.014)]:
  vertices.extend([(r*math.cos(a),.057+r*math.sin(a),z)for a in np.linspace(0,math.tau,24,endpoint=False)])
 for row in range(3):
  for i in range(24):
   j=(i+1)%24;faces.append((row*24+i,(row+1)*24+i,(row+1)*24+j,row*24+j))
 geo.mesh_object('Hollow mini Uzi muzzle',vertices,faces,'steel',2)
 tube('Uzi recessed bore',[(0,.057,-.285),(0,.057,-.300)],.014,'dark',2,24)
 box('Uzi pistol grip',(0,-.029,.010),(.058,.096,.064),'dark',2,.006)
 box('Uzi stick magazine',(0,-.112,.013),(.037,.145,.053),'steel',2,.004)
 box('Uzi magazine floor plate',(0,-.183,.013),(.047,.011,.064),'gold',2,.002)
 for side in [-1,1]:
  for i in range(5):box('Uzi grip checkering '+str(side)+str(i),(side*.030,-.006-i*.012,.011),(.003,.004,.043),'steel',2,.001)
 tube('Uzi trigger guard',[(-.014,.009,-.030),(-.014,-.038,-.035),(-.014,-.039,-.082),(-.014,.009,-.088)],.004,'steel',2,9)
 tube('Uzi trigger',[(-.014,.009,-.055),(-.014,-.019,-.059),(-.014,-.022,-.070)],.0028,'dark',2)
 # Folded wire stock and sights are part of the compact silhouette.
 for side in [-1,1]:
  tube('Folded Uzi stock arm '+str(side),[(side*.030,.058,.069),(side*.041,.108,.127),(side*.041,.139,-.125)],.005,'steel',2,10)
 tube('Folded stock butt',[(-.041,.139,-.125),(-.045,.149,-.146),(.045,.149,-.146),(.041,.139,-.125)],.005,'dark',2,10)
 for z in [-.163,.055]:
  tube('Uzi raised sight '+str(z),[(-.017,.108,z),(-.017,.135,z),(.017,.135,z),(.017,.108,z)],.004,'dark',2,9)
 crown('Uzi queen crest',(-.043,.060,-.033),.032,.025,'gold',2)
 # The hand stays attached to the gun's own joint through the aiming motion.
 tube('Queen right wrist bridge',[(0,0,0),(.012,-.013,.009),(.014,-.037,.016)],.021,'skin',2,14)
 oval('Queen Uzi palm',(.015,-.034,.015),(.028,.033,.022),'skin',2,24,14)
 for j in range(3):
  tube('Queen Uzi gripping finger '+str(j),[(.031,-.018-j*.014,.023),(.034,-.018-j*.014,-.013),(.004,-.021-j*.014,-.023),(-.020,-.021-j*.014,-.009)],.0055,'skin',2,8)
 tube('Queen trigger finger',[(.032,-.008,.008),(.031,-.009,-.042),(.003,-.014,-.055)],.005,'skin',2,8)
 tube('Queen grip thumb',[(-.016,-.002,.031),(-.024,-.020,.025),(-.011,-.032,.001)],.006,'skin',2,8)
 convert=np.array([[1,0,0],[0,0,-1],[0,1,0]]);transform=Matrix((convert@R@convert.T).tolist()).to_4x4();transform.translation=Vector(geo.coord(GRIP))
 for obj in geo.OBJECTS[first:]:
  obj.matrix_world=transform
  if obj['surface']!='skin':obj['equipment']='mini-uzi'

def export_skin(objects,path):
 report=geo.export_skin(objects,path);data=path.read_bytes();data=b'BCM2'+data[4:12]+struct.pack('<I',len(REST))+data[12:];path.write_bytes(data);report['bytes']=len(data);return report
def main():
 OUT.mkdir(parents=True,exist_ok=True);bpy.ops.wm.read_factory_settings(use_empty=True);bpy.context.preferences.filepaths.save_version=0
 geo.REST=REST;geo.influences=weights
 for name,(color,metal,rough)in COLORS.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;shader=m.node_tree.nodes['Principled BSDF']
  shader.inputs['Base Color'].default_value=color;shader.inputs['Metallic'].default_value=metal;shader.inputs['Roughness'].default_value=rough
 sculpt_body();sculpt_arms();sculpt_uzi();bpy.context.view_layer.update()
 report={'parts':{},'rest_joints':REST.tolist(),'muzzle':MUZZLE.tolist(),'ejection':EJECT.tolist(),'carry_tilt':CARRY,
  'source':'Original adult queen, cropped shirt, pleated skirt, long ponytail and mini Uzi.'}
 for surface in COLORS:report['parts'][surface]=export_skin([o for o in geo.OBJECTS if o['surface']==surface],OUT/f'{surface}.mesh')
 report['combined']=export_skin(geo.OBJECTS,OUT/'street-queen.mesh')
 bpy.ops.object.select_all(action='DESELECT')
 for obj in geo.OBJECTS:obj.select_set(True)
 bpy.ops.wm.stl_export(filepath=str(OUT/'street-queen.stl'),export_selected_objects=True)
 bpy.ops.object.select_all(action='DESELECT')
 for obj in geo.OBJECTS:obj.select_set(obj.get('equipment')=='mini-uzi')
 bpy.ops.wm.stl_export(filepath=str(OUT/'queen-mini-uzi.stl'),export_selected_objects=True)
 rig=geo.make_rig();rig.name='GTA Street queen rig';rig.data.name='Queen - 12 joints with skirt'
 bpy.context.view_layer.objects.active=rig;rig.select_set(True);bpy.ops.object.mode_set(mode='EDIT');rig.data.edit_bones['joint_11'].parent=rig.data.edit_bones['joint_00'];bpy.ops.object.mode_set(mode='OBJECT')
 lines=['pub(super) const REST: [Vec3; 12] = [']
 lines.extend('    Vec3::new('+','.join(repr(float(v))for v in p)+'),'for p in REST);lines+= ['];']
 for name,p in [('MUZZLE',MUZZLE),('EJECT',EJECT)]:lines+=['pub(super) const '+name+': Vec3 = Vec3::new('+','.join(repr(float(v))for v in p)+');']
 (OUT/'anchors.rs').write_text('\n'.join(lines)+'\n')
 bpy.context.scene['design']=report['source'];bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'street-queen.blend'))
 (OUT/'build-report.json').write_text(json.dumps(report,indent=2));print('STREET_QUEEN_BUILT',json.dumps(report['combined']),flush=True)
if __name__=='__main__':main()
