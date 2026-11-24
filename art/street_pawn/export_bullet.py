"""Export the same small stylized round used by projectiles/street.rs as STL.
This is a visual game mesh in board units, with no mechanical dimensions.
"""
import math
import struct
from pathlib import Path

def export():
    sides=12
    rings=[(-.035,.008),(-.030,.010),(.010,.010),(.025,.007),(.036,.001)]
    vertices=[(r*math.cos(math.tau*i/sides),z,-r*math.sin(math.tau*i/sides))
              for z,r in rings for i in range(sides)]
    triangles=[]
    for j in range(len(rings)-1):
        for i in range(sides):
            a=j*sides+i; b=j*sides+(i+1)%sides
            triangles.extend([(a,b,b+sides),(a,b+sides,a+sides)])
    for i in range(1,sides-1):
        triangles.append((0,i+1,i))
        top=(len(rings)-1)*sides
        triangles.append((top,top+i,top+i+1))
    output=Path(__file__).resolve().parent/'bullet.stl'
    with output.open('wb') as f:
        f.write(b'BattleChess street pawn visual round; Z-up, +Y forward'.ljust(80,b'\0'))
        f.write(struct.pack('<I',len(triangles)))
        for triangle in triangles:
            p,q,r=[vertices[i] for i in triangle]
            u=[q[i]-p[i] for i in range(3)]; v=[r[i]-p[i] for i in range(3)]
            n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            length=math.sqrt(sum(x*x for x in n)) or 1
            f.write(struct.pack('<12fH',*[x/length for x in n],*p,*q,*r,0))
    print(output)

if __name__=='__main__': export()
