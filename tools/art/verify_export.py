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
# The face/neck/hands now carry authored linear skin colors. Losing COLOR_0
# would silently turn this material white (the failure seen in close-up review).
skin_index=next(i for i,m in enumerate(d['materials']) if m['name']=='Warm matte skin')
for mesh in d['meshes']:
    for primitive in mesh['primitives']:
        if primitive.get('material')!=skin_index:continue
        a=primitive['attributes'];assert 'COLOR_0' in a, 'skin color attribute lost'
        accessor=d['accessors'][a['COLOR_0']]
        colors=list(values(a['COLOR_0']))
        scale={5121:255,5123:65535}.get(accessor['componentType'],1) if accessor.get('normalized') else 1
        colors=[[c/scale for c in color] for color in colors]
        assert all(all(0<=c<=1 for c in color) for color in colors)
        mean=[sum(c[i] for c in colors)/len(colors) for i in range(3)]
        assert .08<mean[0]<.5 and mean[0]>mean[1]>mean[2], f'washed out skin {mean}'
        assert max(c[0] for c in colors)-min(c[0] for c in colors)>.03, 'skin variation lost'
for image in d['images']:assert 'bufferView' in image and 'uri' not in image
print(f'Validated {vertices} vertices, {len(joints)} joints, normalized weights/normals, authored skin colors and embedded textures')
