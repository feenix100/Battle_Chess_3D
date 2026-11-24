"""Export the runtime taser dart as a matching standalone binary STL.
Game Y-up/forward +Z becomes Blender Z-up/forward -Y. Units are game units.
"""
import math
import struct
from pathlib import Path
PROFILE=[(-.052,.010),(-.043,.017),(.013,.017),(.023,.011),(.064,.001)]
def main():
 vertices=[(r*math.cos(i*math.tau/12),-z,r*math.sin(i*math.tau/12)) for z,r in PROFILE for i in range(12)]
 triangles=[]
 for row in range(len(PROFILE)-1):
  for i in range(12):
   a=row*12+i;b=row*12+(i+1)%12;c=b+12;d=a+12
   triangles.extend([(a,b,d),(b,c,d)])
 vertices.extend([(0,-PROFILE[0][0],0),(0,-PROFILE[-1][0],0)])
 for i in range(12):
  j=(i+1)%12;triangles.extend([(60,j,i),(61,48+i,48+j)])
 data=bytearray(b'BattleChess original police taser dart'.ljust(80,b' '))+struct.pack('<I',len(triangles))
 for tri in triangles:
  a,b,c=[vertices[i]for i in tri];u=[b[i]-a[i]for i in range(3)];v=[c[i]-a[i]for i in range(3)]
  n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];length=math.sqrt(sum(x*x for x in n))
  n=[x/length for x in n];data+=struct.pack('<12fH',*n,*a,*b,*c,0)
 path=Path(__file__).with_name('taser-dart.stl');path.write_bytes(data)
 print(f'{path.name}: {len(triangles)} triangles')
if __name__=='__main__':main()
