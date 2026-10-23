"""Validate the production GLB and sample skinned poses (numpy, no GPU/Blender)."""
from pathlib import Path
import json,struct,math
import numpy as np
path=Path(__file__).resolve().parents[2]/'assets/models/textured-resident.glb'
b=path.read_bytes();magic,version,total=struct.unpack_from('<III',b)
assert magic==0x46546c67 and version==2 and total==len(b)
n=struct.unpack_from('<I',b,12)[0];g=json.loads(b[20:20+n]);start=20+n
size=struct.unpack_from('<I',b,start)[0];binary=b[start+8:start+8+size]
def read(index):
 a=g['accessors'][index];v=g['bufferViews'][a['bufferView']]
 dtype={5121:'<u1',5123:'<u2',5125:'<u4',5126:'<f4'}[a['componentType']]
 width={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}[a['type']]
 step=np.dtype(dtype).itemsize*width;stride=v.get('byteStride',step);offset=v.get('byteOffset',0)+a.get('byteOffset',0)
 result=np.ndarray((a['count'],width),dtype=dtype,buffer=binary,offset=offset,strides=(stride,np.dtype(dtype).itemsize)).copy()
 if a.get('normalized'):result=result.astype(float)/np.iinfo(np.dtype(dtype)).max
 return result
parents={child:i for i,node in enumerate(g['nodes']) for child in node.get('children',[])}
def quaternion(q):
 x,y,z,w=q/np.linalg.norm(q)
 return np.array([[1-2*y*y-2*z*z,2*x*y-2*z*w,2*x*z+2*y*w],[2*x*y+2*z*w,1-2*x*x-2*z*z,2*y*z-2*x*w],[2*x*z-2*y*w,2*y*z+2*x*w,1-2*x*x-2*y*y]])
def matrices(animation=None,time=0):
 overrides={}
 if animation:
  for channel in animation['channels']:
   sampler=animation['samplers'][channel['sampler']];times=read(sampler['input'])[:,0];values=read(sampler['output'])
   hi=min(len(times)-1,int(np.searchsorted(times,time)));lo=max(0,hi-1)
   t=0 if times[hi]==times[lo] else np.clip((time-times[lo])/(times[hi]-times[lo]),0,1)
   a,c=values[lo],values[hi]
   if channel['target']['path']=='rotation' and np.dot(a,c)<0:c=-c
   overrides[(channel['target']['node'],channel['target']['path'])]=a*(1-t)+c*t
 result={}
 def world(i):
  if i in result:return result[i]
  node=g['nodes'][i]
  if 'matrix' in node:m=np.array(node['matrix']).reshape(4,4).T
  else:
   t=overrides.get((i,'translation'),node.get('translation',[0,0,0]))
   q=np.array(overrides.get((i,'rotation'),node.get('rotation',[0,0,0,1])))
   s=overrides.get((i,'scale'),node.get('scale',[1,1,1]))
   m=np.eye(4);m[:3,:3]=quaternion(q)@np.diag(s);m[:3,3]=t
  result[i]=world(parents[i])@m if i in parents else m
  return result[i]
 for i in range(len(g['nodes'])):world(i)
 return result
assert 'Permission is hereby granted' in g['asset'].get('copyright',''), 'MIT notice must travel inside the embedded GLB'
animations={a['name']:a for a in g['animations']}
assert set(animations)=={'Idle','Walk','Work','Table'},set(animations)
assert len(g['skins'])==1 and len(g['skins'][0]['joints'])==80
materials={m['name']:m for m in g['materials']}
for name in ['Resident skin head','Resident skin hands']:
 m=materials[name];assert 'baseColorTexture' in m['pbrMetallicRoughness'] and 'normalTexture' in m
assert all('bufferView' in im and 'uri' not in im for im in g['images'])
assert all(n.get('scale',[1,1,1])[0]<2 for n in g['nodes']), 'centimetre-scale node leaked into output'
skin=g['skins'][0];inverse=read(skin['inverseBindMatrices']).reshape(-1,4,4).transpose(0,2,1)
meshes=[];vertex_count=0
for mesh in g['meshes']:
 for p in mesh['primitives']:
  a=p['attributes'];pos=read(a['POSITION']);norm=read(a['NORMAL']);j=read(a['JOINTS_0']);w=read(a['WEIGHTS_0'])
  assert np.isfinite(pos).all() and np.allclose(np.linalg.norm(norm,axis=1),1,atol=.005)
  assert np.allclose(w.sum(axis=1),1,atol=.001) and j.max()<len(skin['joints'])
  vertex_count+=len(pos);step=max(1,len(pos)//2000)
  meshes.append((np.column_stack([pos,np.ones(len(pos))])[::step],j[::step],w[::step]))
def joint(name):
 return next(i for i,n in enumerate(g['nodes']) if n.get('name')==name)
root=joint('Bip01 Pelvis')
# Catch the two rig-conversion regressions seen during authoring: preserving each
# clip's bind-pose deltas instead of absolute anatomical rotations, and assigning
# a zero-length Blender EditBone's matrix before its length. Both leave idle arms
# in A-pose despite valid weights and superficially plausible overall bounds.
idle=matrices(animations['Idle'],.37)
for side in ['L','R']:
 shoulder=idle[joint('Bip01 '+side+' UpperArm')][:3,3]
 wrist=idle[joint('Bip01 '+side+' Hand')][:3,3]
 assert shoulder[1]-wrist[1]>.4 and abs(shoulder[0]-wrist[0])<.15, (side,shoulder,wrist)

for label,animation in animations.items():
 duration=max(float(read(s['input'])[-1,0]) for s in animation['samplers']);root_positions=[]
 for time in np.linspace(0,duration,9):
  world=matrices(animation,time);root_positions.append(world[root][:3,3])
  palette=np.array([world[j]@inv for j,inv in zip(skin['joints'],inverse)])
  points=[]
  for pos,j,w in meshes:
   skinned=sum((np.einsum('nij,nj->ni',palette[j[:,k]],pos)*w[:,k,None] for k in range(4)))
   points.append(skinned[:,:3])
  points=np.concatenate(points)
  assert np.isfinite(points).all()
  low,high=points.min(axis=0),points.max(axis=0)
  assert low[1]>-.3 and high[1]<2.3 and np.max(np.abs(points[:,[0,2]]))<2, (label,float(time),low,high)
 roots=np.array(root_positions)
 assert np.ptp(roots[:,[0,2]],axis=0).max()<.0001, f'{label} has horizontal root motion'
 print(label,'duration',round(duration,3),'sampled skin poses within metre-scale bounds')
print('Validated',vertex_count,'vertices, 80 joints, four clips, embedded PBR maps and no horizontal root motion')
