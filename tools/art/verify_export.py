"""Validate the checked-in runtime skin, geometry and embedded textures without Blender."""
from pathlib import Path
import json, math, struct
p=Path(__file__).resolve().parents[2]/'assets/models/coastal-resident.glb'
b=p.read_bytes()
magic,version,total=struct.unpack_from('<III',b)
assert magic==0x46546c67 and version==2 and total==len(b)
n,kind=struct.unpack_from('<II',b,12); d=json.loads(b[20:20+n])
start=20+n; size,kind=struct.unpack_from('<II',b,start); binary=b[start+8:start+8+size]
def values(index):
    a=d['accessors'][index];v=d['bufferViews'][a['bufferView']]
    count={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}[a['type']]
    code={5121:'B',5123:'H',5125:'I',5126:'f'}[a['componentType']]
    width=struct.calcsize('<'+code)*count; stride=v.get('byteStride',width)
    offset=v.get('byteOffset',0)+a.get('byteOffset',0)
    for i in range(a['count']):yield struct.unpack_from('<'+code*count,binary,offset+i*stride)
assert len(d['skins'])==1
joints=d['skins'][0]['joints'];names={d['nodes'][i]['name'] for i in joints}
assert len(joints)==15
assert all(f'{name}.{side}' in names for name in ('upper_arm','forearm','hand','thigh','shin','foot') for side in ('L','R'))
vertices=0
for mesh in d['meshes']:
    for primitive in mesh['primitives']:
        a=primitive['attributes'];assert {'POSITION','NORMAL','JOINTS_0','WEIGHTS_0'}<=a.keys()
        for position in values(a['POSITION']): assert all(math.isfinite(x) for x in position)
        for normal in values(a['NORMAL']): assert abs(sum(x*x for x in normal)-1)<.005
        for weights in values(a['WEIGHTS_0']): assert abs(sum(weights)-1)<.001
        for joint in values(a['JOINTS_0']): assert all(0<=x<len(joints) for x in joint)
        vertices+=d['accessors'][a['POSITION']]['count']
for image in d['images']:assert 'bufferView' in image and 'uri' not in image
print(f'Validated {vertices} vertices, {len(joints)} joints, normalized weights/normals and embedded textures')
