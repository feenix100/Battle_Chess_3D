"""Mini Uzi in-game round and spent case as separate Z-up binary STLs."""
import math,struct
from pathlib import Path
def lathe(profile,sides=20):
 v=[(r*math.cos(i*math.tau/sides),-z,r*math.sin(i*math.tau/sides))for z,r in profile for i in range(sides)];t=[]
 for row in range(len(profile)-1):
  for i in range(sides):
   a=row*sides+i;b=row*sides+(i+1)%sides;c=b+sides;d=a+sides;t.extend([(a,b,d),(b,c,d)])
 cap=len(v);v.extend([(0,-profile[0][0],0),(0,-profile[-1][0],0)])
 for i in range(sides):
  j=(i+1)%sides;top=(len(profile)-1)*sides;t.extend([(cap,j,i),(cap+1,top+i,top+j)])
 return v,t
def write(name,vertices,triangles):
 data=bytearray(b'BattleChess original GTA Street queen prop'.ljust(80,b' '))+struct.pack('<I',len(triangles))
 for tri in triangles:
  a,b,c=[vertices[i]for i in tri];u=[b[i]-a[i]for i in range(3)];v=[c[i]-a[i]for i in range(3)]
  n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];length=math.sqrt(sum(x*x for x in n))
  n=[x/length for x in n];data+=struct.pack('<12fH',*n,*a,*b,*c,0)
 p=Path(__file__).with_name(name);p.write_bytes(data);print(name,len(triangles),'triangles')
write('smg-round.stl',*lathe([(-.029,.006),(-.023,.007),(.010,.007),(.023,.004),(.029,.001)],12))
write('smg-spent-case.stl',*lathe([(-.021,.007),(-.017,.008),(.020,.007)],12))
