"""Original barrel-chested GTA Street king, diagonally carried shotgun and rig.
Run with Blender 5.2. Game Y-up/-Z-forward maps to Blender Z-up/+Y-forward.
"""
import importlib.util,json,math
from pathlib import Path
import bpy
import numpy as np
from mathutils import Matrix,Vector
OUT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('king_geometry',OUT.parent/'street_pawn/build_mesh.py')
geo=importlib.util.module_from_spec(spec);spec.loader.exec_module(geo)
REST=np.array([(0,.600,.012),(0,1.095,0),(.255,.935,-.145),
    (-.235,.995,-.070),(-.285,.702,-.086),(.235,.995,0),(.295,.755,-.034),
    (-.112,.600,.015),(-.130,.320,-.005),(.112,.600,.015),(.130,.320,-.005)],dtype=float)
GRIP=REST[2].copy();D=np.array([0,.69,.724]);D/=np.linalg.norm(D)
UP=np.cross(D,[1,0,0]);X=np.array([1,0,0])
MUZZLE=GRIP+UP*.047+D*.636
EJECT=GRIP+X*.040+UP*.050+D*.035
LEFT_HAND=np.array([-.290,.418,-.102])
COLORS={
 'team':((.23,.36,.33,1),.02,.75),'skin':((.50,.265,.145,1),0,.72),
 'dark':((.018,.024,.030,1),.14,.44),'gold':((.78,.52,.15,1),.76,.27),
 'steel':((.26,.31,.34,1),.75,.28),'wood':((.20,.067,.022,1),.04,.48),
 'stitch':((.43,.47,.43,1),.0,.76),'eye':((.85,.82,.72,1),.0,.54),
 'shell':((.49,.025,.018,1),.05,.47),
}
def weights(points,kind):
 w=np.zeros((len(points),11))
 if isinstance(kind,int):w[:,kind]=1
 elif kind=='torso':
  head=geo.smoothstep(1.045,1.120,points[:,1]);w[:,0]=1-head;w[:,1]=head
 elif kind.startswith('leg'):
  upper=7 if kind.endswith('L')else 9;shin=1-geo.smoothstep(.282,.356,points[:,1]);w[:,upper]=1-shin;w[:,upper+1]=shin
 else:raise ValueError(kind)
 return w
def loft(name,rings,surface='team',bone=0,sides=32,power=1):return geo.loft(name,rings,surface,bone,sides,power)
def oval(name,c,r,surface='skin',bone=1,sides=28,stacks=15):return geo.ellipsoid(name,c,r,surface,bone,sides,stacks)
def box(name,c,size,surface='dark',bone=0,bevel=.005):return geo.box(name,c,size,surface,bone,bevel)
def tube(name,p,r=.003,surface='gold',bone=0,sides=8,closed=False):return geo.tube(name,p,r,surface,bone,sides,closed)
def muscle(name,start,end,profile,bone):
 axis=end-start;axis=axis/np.linalg.norm(axis);side=np.cross(axis,[0,0,1]);side/=np.linalg.norm(side);front=np.cross(axis,side)
 vertices=[];faces=[];sides=24
 for t,r in profile:
  c=start+(end-start)*t
  vertices.extend([tuple(c+side*r*math.cos(a)+front*r*.90*math.sin(a))for a in np.linspace(0,math.tau,sides,endpoint=False)])
 faces.append(tuple(reversed(range(sides))))
 for row in range(len(profile)-1):
  for i in range(sides):
   j=(i+1)%sides;faces.append((row*sides+i,row*sides+j,(row+1)*sides+j,(row+1)*sides+i))
 faces.append(tuple((len(profile)-1)*sides+i for i in range(sides)))
 return geo.mesh_object(name,vertices,faces,'skin',bone)
def crown(name,c,width=.07,height=.048,surface='gold',bone=0):
 x,y,z=c;outline=[(-.5,-.4),(.5,-.4),(.59,.45),(.24,.10),(0,.65),(-.24,.10),(-.59,.45)]
 vertices=[(x+a*width,y+b*height,z+d)for d in [-.004,.004]for a,b in outline];n=len(outline)
 faces=[tuple(reversed(range(n))),tuple(range(n,n*2))]+[(i,(i+1)%n,n+(i+1)%n,n+i)for i in range(n)]
 return geo.mesh_object(name,vertices,faces,surface,bone)
def chain_link(name,c,r1=.014,r2=.009,bone=0,sideways=False):
 x,y,z=c
 return tube(name,[(x+r1*math.cos(a),y+r2*math.sin(a),z+(.003*math.sin(a*2)if sideways else 0))for a in np.linspace(0,math.tau,14,endpoint=False)],.0032,'gold',bone,7,True)
def sculpt_body():
 # A rounded, heavy chest tapers to the belt and broad hips.
 loft('Barrel chest tank top',[(.579,0,.022,.179,.122),(.620,0,.017,.191,.129),
  (.705,0,.010,.211,.142),(.808,0,.008,.235,.151),(.900,0,.007,.244,.151),
  (.962,0,.012,.231,.136),(1.011,0,.012,.177,.105),(1.046,0,.010,.091,.065)],'team','torso',48,.85)
 for side in [-1,1]:
  tube('Tank armhole binding '+str(side),[(side*.179,1.008,-.052),(side*.222,.963,-.087),
   (side*.235,.901,-.076),(side*.218,.848,-.054)],.006,'team',0,10)
  tube('Tank side seam '+str(side),[(side*.183,.625,.025),(side*.205,.737,.043),(side*.231,.870,.045)],.0025,'stitch',0)
  tube('Pectoral lower fold '+str(side),[(side*.037,.861,-.148),(side*.115,.845,-.151),(side*.182,.867,-.139)],.0025,'team',0)
 loft('Tank rolled neckline',[(1.031,0,.004,.099,.072),(1.044,0,.005,.096,.069),(1.052,0,.005,.084,.060)],'team','torso',40)
 loft('Tank bottom hem',[(.582,0,.021,.180,.124),(.594,0,.020,.184,.126),(.609,0,.019,.184,.124)],'team',0,40,.85)
 for i in range(-5,6):
  x=i*.027;z=-.134+abs(i)*.003
  tube('Tank ribbing '+str(i),[(x,.635,z),(x*1.10,.708,z-.008),(x*1.15,.787,z-.012)],.0017,'team',0,6)
 loft('Powerful neck',[(1.020,0,.005,.077,.071),(1.166,0,0,.073,.067)],'skin','torso',36)
 # Long, angular adult face with a substantial handlebar mustache.
 loft('King face',[(1.101,0,-.030,.045,.051),(1.125,0,-.031,.079,.071),
  (1.177,0,-.023,.099,.085),(1.245,0,-.006,.109,.094),(1.315,0,.004,.106,.096),
  (1.374,0,.011,.097,.090),(1.424,0,.017,.068,.066),(1.450,0,.017,.020,.025)],'skin',1,44,.78)
 for side in [-1,1]:
  oval('Ear '+str(side),(side*.108,1.276,.011),(.023,.039,.024),'skin',1)
  oval('Ear inset '+str(side),(side*.123,1.279,.003),(.008,.023,.012),'skin',1,18,12)
  oval('Eye white '+str(side),(side*.047,1.314,-.091),(.022,.008,.004),'eye',1,24,12)
  oval('Dark iris '+str(side),(side*.045,1.314,-.096),(.006,.006,.002),'dark',1,18,10)
  tube('Heavy eyebrow '+str(side),[(side*.020,1.340,-.090),(side*.047,1.346,-.094),(side*.079,1.338,-.086)],.007,'dark',1,9)
  tube('Lower eyelid '+str(side),[(side*.021,1.304,-.085),(side*.046,1.301,-.092),(side*.072,1.305,-.086)],.003,'skin',1)
  tube('King mustache '+str(side),[(side*.006,1.251,-.108),(side*.024,1.257,-.117),
   (side*.048,1.248,-.114),(side*.070,1.232,-.101),(side*.076,1.215,-.092)],.011,'dark',1,12)
  for j in range(3):tube('Mustache hair detail '+str(side)+str(j),[(side*(.014+j*.013),1.257,-.119),
   (side*(.028+j*.012),1.243,-.120)],.0014,'steel',1,5)
  tube('Sideburn '+str(side),[(side*.099,1.352,.015),(side*.102,1.298,.015),(side*.097,1.267,.006)],.008,'dark',1)
 loft('King nose',[(1.251,0,-.091,.020,.025),(1.269,0,-.107,.026,.029),
  (1.290,0,-.107,.020,.026),(1.337,0,-.083,.015,.017)],'skin',1,24,.85)
 tube('Mouth crease',[(-.034,1.224,-.103),(0,1.217,-.111),(.034,1.224,-.103)],.0025,'dark',1)
 tube('Heavy lower lip',[(-.028,1.211,-.104),(0,1.207,-.109),(.028,1.211,-.104)],.004,'skin',1)
 tube('Chin cleft',[(0,1.161,-.099),(0,1.174,-.102)],.002,'skin',1)
 # Tied head-wrap: forehead band, seams, folds, knot and two fabric tails.
 loft('Crown bandanna',[(1.351,0,.011,.107,.098),(1.383,0,.014,.114,.103),
  (1.428,0,.018,.109,.101),(1.475,0,.022,.084,.077),(1.503,0,.024,.029,.031)],'team',1,44,.90)
 tube('Bandanna front folded edge',[(-.096,1.362,-.033),(-.065,1.366,-.078),(0,1.367,-.092),(.065,1.366,-.078),(.096,1.362,-.033)],.005,'team',1,10)
 for side in [-1,1]:
  tube('Bandanna crown seam '+str(side),[(side*.094,1.392,.037),(side*.064,1.450,.048),(side*.016,1.497,.033)],.0024,'stitch',1)
 oval('Bandanna tied knot',(0,1.394,.119),(.040,.034,.027),'team',1,24,14)
 for side in [-1,1]:
  vertices=[(side*.011,1.402,.132),(side*.047,1.390,.137),(side*.077,1.239,.154),
   (side*.034,1.255,.162),(side*.014,1.312,.145)]
  tail=geo.mesh_object('Bandanna fabric tail '+str(side),vertices,[(0,1,2,3,4)],'team',1)
  solid=tail.modifiers.new('Fabric thickness','SOLIDIFY');solid.thickness=.005
  bpy.context.view_layer.objects.active=tail;bpy.ops.object.modifier_apply(modifier=solid.name)
 crown('Bandanna king crown',(0,1.404,-.092),.070,.044,'gold',1)
 # Thick gold links, pendant, and a crown-shaped belt buckle.
 for i in range(25):
  t=i/24;x=-.155+.31*t;z=-.09-.085*math.sin(math.pi*t)
  y=1.025-.105*math.sin(math.pi*t)
  chain_link('Individual necklace link '+str(i),(x,y,z),.011,.008,0,i%2==0)
 tube('Pendant hanger',[(0,.931,-.122),(0,.895,-.154)],.004,'gold',0)
 crown('King medallion',(0,.875,-.157),.065,.047)
 loft('Baggy trousers waist',[(.490,0,.021,.171,.118),(.594,0,.020,.190,.128),(.629,0,.018,.184,.121)],'team',0,40,.78)
 loft('Leather duty belt',[(.589,0,.020,.191,.129),(.624,0,.019,.190,.128)],'dark',0,40,.72)
 box('King belt buckle',(0,.609,-.120),(.073,.044,.014),'gold',0)
 crown('Buckle crown inset',(0,.608,-.131),.044,.025,'dark',0)
 for side,letter in [(-1,'L'),(1,'R')]:
  leg='leg'+letter;shin=8 if side<0 else 10
  loft('Heavy trousers '+letter,[(.152,side*.130,.015,.068,.068),(.224,side*.133,.013,.075,.077),
   (.316,side*.130,-.004,.080,.083),(.405,side*.124,.012,.091,.090),
   (.510,side*.115,.023,.096,.100),(.606,side*.100,.021,.079,.092)],'team',leg,36,.75)
  box('Cargo pocket '+letter,(side*.192,.443,.033),(.028,.119,.094),'team',leg,.009)
  box('Cargo pocket flap '+letter,(side*.199,.494,.033),(.030,.025,.099),'dark',leg,.006)
  tube('Trouser outer seam '+letter,[(side*.197,.222,.022),(side*.211,.337,.028),(side*.216,.435,.021),(side*.198,.553,.030)],.0025,'stitch',leg)
  for j in range(3):tube('Knee drape '+letter+str(j),[(side*.185,.289+j*.025,-.039),(side*.132,.281+j*.025,-.087),(side*.089,.288+j*.025,-.061)],.003,'team',leg)
  box('Belt loop '+letter,(side*.104,.609,-.107),(.020,.055,.014),'team',0)
  loft('Heavy boot '+letter,[(.024,side*.133,-.031,.075,.127),(.064,side*.133,-.031,.078,.128),
   (.116,side*.132,-.005,.065,.085),(.209,side*.131,.017,.061,.067),(.238,side*.131,.019,.059,.063)],'dark',shin,36,.70)
  loft('Boot sole '+letter,[(.005,side*.133,-.033,.081,.134),(.026,side*.133,-.033,.083,.135),(.040,side*.133,-.033,.080,.132)],'dark',shin,36,.64)
  tube('Boot welt '+letter,[(side*.133+.079*math.cos(a),.037,-.033+.130*math.sin(a))for a in np.linspace(0,math.tau,30,endpoint=False)],.003,'stitch',shin,7,True)
  for j in range(5):
   y=.084+j*.024
   tube('Crossed boot lace '+letter+str(j),[(side*.133-.025,y,-.086),(side*.133+.025,y+.016,-.082)],.0025,'stitch',shin)
   tube('Crossed boot lace reverse '+letter+str(j),[(side*.133+.025,y,-.086),(side*.133-.025,y+.016,-.082)],.0025,'stitch',shin)
   for sign in [-1,1]:oval('Boot eyelet '+letter+str(j)+str(sign),(side*.133+sign*.031,y,-.080),(.004,.004,.003),'gold',shin,10,6)
  for j in range(6):box('Sole tread '+letter+str(j),(side*.133,.012,-.120+j*.037),(.149,.016,.016),'steel',shin,.002)
 # A wallet chain curves around the right hip in individual links.
 for i in range(14):
  t=i/13;chain_link('Wallet chain '+str(i),(.184+.041*math.sin(math.pi*t),.586-.111*math.sin(math.pi*t),-.043+.139*t),.010,.007,0,i%2==0)

def sculpt_arms():
 for side,upper,lower in [(-1,3,4),(1,5,6)]:
  shoulder=REST[upper];elbow=REST[lower]
  wrist=LEFT_HAND if side<0 else GRIP
  direction=elbow-shoulder
  # Each capped muscle follows one bone; elbow caps hide the articulation seam.
  muscle('Muscular upper arm '+str(side),shoulder,elbow,[(0,.059),(.18,.078),(.42,.075),(.75,.062),(1,.047)],upper)
  oval('Shoulder deltoid '+str(side),tuple(shoulder+np.array([side*.012,-.008,.003])),(.088,.082,.086),'skin',upper,32,18)
  oval('Rounded elbow '+str(side),tuple(elbow),(.052,.050,.052),'skin',lower,24,14)
  muscle('Heavy forearm '+str(side),elbow,wrist,[(0,.046),(.22,.054),(.55,.048),(.82,.035),(1,.030)],lower)
  if side<0:
   oval('Left palm',tuple(wrist+np.array([0,-.026,-.002])),(.040,.052,.027),'skin',lower,26,14)
   for j in range(4):
    x=wrist[0]+(j-1.5)*.016
    tube('Left finger '+str(j),[(x,wrist[1]-.020,wrist[2]-.020),(x,wrist[1]-.061,wrist[2]-.026),(x,wrist[1]-.074,wrist[2]-.011)],.007,'skin',lower,9)
   tube('Left thumb',[tuple(wrist+np.array([.030,-.004,-.012])),tuple(wrist+np.array([.046,-.034,-.023])),tuple(wrist+np.array([.030,-.048,-.026]))],.009,'skin',lower,10)
   box('Watch leather strap',tuple(wrist+np.array([0,.038,0])),(.099,.023,.065),'dark',lower)
   box('Square watch bezel',tuple(wrist+np.array([-.051,.038,-.005])),(.009,.031,.039),'gold',lower,.003)
   box('Watch face',tuple(wrist+np.array([-.057,.038,-.005])),(.003,.023,.029),'dark',lower,.001)
  # Original ink bands and a small geometric crown tattoo, without gang emblems.
  if side<0:
   axis=direction/np.linalg.norm(direction);u=np.cross(axis,[0,0,1]);u/=np.linalg.norm(u);v=np.cross(axis,u)
   for t in [.35,.43]:
    c=shoulder+direction*t
    tube('Arm tattoo band '+str(t),[tuple(c+u*.0758*math.cos(a)+v*.0682*math.sin(a))for a in np.linspace(0,math.tau,30,endpoint=False)],.0018,'dark',upper,7,True)
   crown('Upper-arm crown tattoo',(-.262,.914,-.147),.044,.029,'dark',upper)

def sculpt_shotgun():
 first=len(geo.OBJECTS)
 # Local Z is the bore direction; the completed gun is tilted against the shoulder.
 box('Shotgun receiver',(0,.042,.020),(.070,.079,.171),'steel',2,.008)
 box('Receiver side inset',(.037,.044,.018),(.005,.052,.122),'dark',2,.003)
 box('Open ejection port',(.041,.054,.045),(.004,.023,.054),'dark',2,.002)
 box('Bolt visible in port',(.044,.055,.041),(.004,.013,.033),'steel',2,.001)
 box('Shotgun top rib',(0,.087,.074),(.027,.013,.264),'dark',2,.002)
 tube('Shotgun barrel',[(0,.047,.103),(0,.047,.629)],.023,'dark',2,24)
 # A true annular muzzle, with its dark bore recessed behind the steel rim.
 vertices=[];faces=[]
 for z,r in [(.619,.024),(.634,.024),(.634,.017),(.609,.017)]:
  vertices.extend([(r*math.cos(a),.047+r*math.sin(a),z)for a in np.linspace(0,math.tau,24,endpoint=False)])
 for row in range(3):
  for i in range(24):
   j=(i+1)%24;faces.append((row*24+i,row*24+j,(row+1)*24+j,(row+1)*24+i))
 geo.mesh_object('Hollow shotgun muzzle',vertices,faces,'steel',2)
 tube('Recessed dark bore',[(0,.047,.596),(0,.047,.607)],.017,'dark',2,24)
 tube('Under-barrel magazine',[(0,.002,.096),(0,.002,.444)],.019,'steel',2,20)
 box('Walnut pump grip',(0,-.006,.240),(.076,.066,.194),'wood',2,.009)
 for j in range(10):
  z=.155+j*.018
  loft('Pump grip raised rib '+str(j),[(-.039,0,z,.039,.005),(-.034,0,z,.043,.007),(.018,0,z,.043,.007),(.024,0,z,.037,.005)],'dark',2,16,.35)
 box('Barrel clamp',(0,.021,.421),(.057,.069,.019),'steel',2,.003)
 box('Muzzle front sight',(0,.075,.589),(.010,.014,.022),'gold',2,.002)
 box('Rear sight',(0,.094,.089),(.030,.014,.013),'steel',2,.002)
 # Carved broad buttstock, pistol grip, rubber shoulder pad and comb.
 loft('Walnut shoulder stock',[(-.052,0,-.183,.041,.100),(-.030,0,-.184,.043,.115),
  (.050,0,-.178,.042,.111),(.083,0,-.147,.033,.085)],'wood',2,28,.40)
 box('Rubber stock butt',(0,.001,-.303),(.091,.151,.020),'dark',2,.005)
 box('Raised stock comb',(0,.078,-.199),(.062,.019,.140),'wood',2,.005)
 box('Wooden grip',(0,-.042,-.065),(.058,.100,.055),'wood',2,.010)
 for j in range(4):tube('Stock carved grain '+str(j),[(.044,-.032+j*.022,-.287),(.045,-.023+j*.022,-.227),(.043,-.019+j*.022,-.135)],.0017,'gold',2,6)
 tube('Trigger guard',[(-.016,-.004,-.022),(-.016,-.050,-.013),(-.016,-.059,.032),(-.016,-.004,.047)],.005,'steel',2,9)
 tube('Trigger',[(-.015,.003,.004),(-.015,-.027,.009),(-.015,-.034,.023)],.003,'dark',2)
 for z in [-.024,.076]:oval('Receiver pin',(.040,.035,z),(.003,.005,.005),'gold',2,12,8)
 # Wrist and gripping hand rotate with the gun, keeping the grip attached.
 tube('Right wrist bridge',[(0,0,0),(.008,-.027,-.024),(.012,-.047,-.043)],.028,'skin',2,14)
 oval('Right shotgun palm',(.012,-.057,-.057),(.037,.047,.027),'skin',2,26,16)
 for j in range(3):
  tube('Right curled grip finger '+str(j),[(.034,-.037-j*.018,-.067),(.036,-.041-j*.018,-.036),(.008,-.041-j*.018,-.024),(-.019,-.041-j*.018,-.038)],.007,'skin',2,9)
 tube('Right trigger finger',[(.034,-.022,-.056),(.031,-.020,-.006),(.005,-.026,.011)],.0065,'skin',2,9)
 tube('Right thumb',[(-.018,-.020,-.078),(-.029,-.044,-.063),(-.011,-.056,-.036)],.009,'skin',2,10)
 # Transform gun-space into the game-space diagonal carry, then into Blender.
 rotation=np.column_stack([X,UP,D]);convert=np.array([[1,0,0],[0,0,-1],[0,1,0]])
 transform=Matrix((convert@rotation@convert.T).tolist()).to_4x4();transform.translation=Vector(geo.coord(GRIP))
 for obj in geo.OBJECTS[first:]:
  obj.matrix_world=transform
  if obj['surface']!='skin':obj['equipment']='shotgun'

def main():
 OUT.mkdir(parents=True,exist_ok=True);bpy.ops.wm.read_factory_settings(use_empty=True)
 bpy.context.preferences.filepaths.save_version=0;geo.REST=REST;geo.influences=weights
 for name,(color,metal,rough)in COLORS.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True
  shader=m.node_tree.nodes['Principled BSDF'];shader.inputs['Base Color'].default_value=color
  shader.inputs['Metallic'].default_value=metal;shader.inputs['Roughness'].default_value=rough
 sculpt_body();sculpt_arms();sculpt_shotgun();bpy.context.view_layer.update()
 report={'parts':{},'rest_joints':REST.tolist(),'muzzle':MUZZLE.tolist(),'ejection':EJECT.tolist(),
  'carry_barrel_axis':D.tolist(),'left_hand':LEFT_HAND.tolist(),'source':'Original barrel-chested king and shoulder-carried shotgun.'}
 for surface in COLORS:
  objects=[o for o in geo.OBJECTS if o['surface']==surface]
  if objects:report['parts'][surface]=geo.export_skin(objects,OUT/f'{surface}.mesh')
 report['combined']=geo.export_skin(geo.OBJECTS,OUT/'street-king.mesh')
 bpy.ops.object.select_all(action='DESELECT')
 for obj in geo.OBJECTS:obj.select_set(True)
 bpy.ops.wm.stl_export(filepath=str(OUT/'street-king.stl'),export_selected_objects=True)
 bpy.ops.object.select_all(action='DESELECT')
 for obj in geo.OBJECTS:obj.select_set(obj.get('equipment')=='shotgun')
 bpy.ops.wm.stl_export(filepath=str(OUT/'king-shotgun.stl'),export_selected_objects=True)
 rig=geo.make_rig();rig.name='GTA Street king rig';rig.data.name='King - 11 joints and shoulder carry'
 bpy.context.scene['design']='Original barrel-chested king, mustache, tied bandanna and diagonally shoulder-carried shotgun'
 lines=['pub(super) const REST: [Vec3; 11] = [']
 lines.extend('    Vec3::new('+','.join(repr(float(v))for v in p)+'),'for p in REST);lines+= ['];']
 for name,p in [('MUZZLE',MUZZLE),('EJECT',EJECT),('BARREL_AXIS',D),('LEFT_HAND',LEFT_HAND)]:
  lines+=['pub(super) const '+name+': Vec3 = Vec3::new('+','.join(repr(float(v))for v in p)+');']
 (OUT/'anchors.rs').write_text('\n'.join(lines)+'\n')
 bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'street-king.blend'))
 (OUT/'build-report.json').write_text(json.dumps(report,indent=2))
 print('STREET_KING_BUILT',json.dumps(report['combined']),flush=True)
if __name__=='__main__':main()
