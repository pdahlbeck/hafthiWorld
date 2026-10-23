"""Original coastal resident study, authored in Blender metres. Blender 4.3+.
Run: blender --background --python-exit-code 1 --python tools/art/build_resident.py
Project-authored garments/rig; the anatomical head and hands adapt the documented CC0 MakeHuman reference.
"""
import bpy, bmesh, math
from mathutils.kdtree import KDTree
from mathutils import Vector
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
bpy.context.preferences.filepaths.save_version=0
bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)

def material(name, color, roughness=0.9, texture=None):
    m=bpy.data.materials.new(name); m.diffuse_color=(*color,1); m.use_nodes=True
    bs=m.node_tree.nodes.get('Principled BSDF'); bs.inputs['Base Color'].default_value=(*color,1)
    bs.inputs['Roughness'].default_value=roughness
    if texture:
        nt=m.node_tree
        image=nt.nodes.new('ShaderNodeTexImage'); image.image=bpy.data.images.load(str(ROOT/f'assets/materials/{texture}-color.png')); image.image.pack()
        nt.links.new(image.outputs['Color'],bs.inputs['Base Color'])
        n=nt.nodes.new('ShaderNodeTexImage'); n.image=bpy.data.images.load(str(ROOT/f'assets/materials/{texture}-normal.png')); n.image.colorspace_settings.name='Non-Color'; n.image.pack()
        normal=nt.nodes.new('ShaderNodeNormalMap'); normal.inputs['Strength'].default_value=.35
        nt.links.new(n.outputs['Color'],normal.inputs['Color']); nt.links.new(normal.outputs['Normal'],bs.inputs['Normal'])
    return m
cloth=material('Undyed sage wool',(.29,.34,.25),texture='wool')
skin=material('Warm matte skin',(.28,.145,.09),.93)
leather=material('Worn brown leather',(.105,.065,.038),.92)
linen=material('Natural linen',(.27,.23,.17))
hair=material('Chestnut hair',(.026,.014,.008))
hair_strand=material('Chestnut strand highlights',(.047,.025,.012),.93)
eye=material('Small dark iris',(.035,.042,.030),.7)
white=material('Warm eye white',(.25,.235,.20),.85)
lip=material('Muted lips',(.13,.045,.032))
metal=material('Iron buckle',(.15,.16,.15),.65)
objects=[]

def finish(name, verts, faces, mat, weights, uv=None):
    me=bpy.data.meshes.new(name); me.from_pydata(verts,[],faces); me.update()
    ob=bpy.data.objects.new(name,me); bpy.context.collection.objects.link(ob); ob.data.materials.append(mat)
    for p in me.polygons: p.use_smooth=True
    layer=me.uv_layers.new(name='Surface')
    for p in me.polygons:
        for li in p.loop_indices:
            vi=me.loops[li].vertex_index
            layer.data[li].uv=uv[vi] if uv else (verts[vi][0]*3,verts[vi][2]*3)
    for i, groups in enumerate(weights):
        for bone,weight in groups.items():
            vg=ob.vertex_groups.get(bone) or ob.vertex_groups.new(name=bone); vg.add([i],weight,'REPLACE')
    if mat == skin:
        colors=me.color_attributes.new(name='Skin tone',type='FLOAT_COLOR',domain='POINT')
        for i,(x,y,z) in enumerate(verts):
            cheek=math.exp(-((abs(x)-.054)/.032)**2-((z-1.545)/.035)**2)*max(0,min(1,(-y-.035)/.05)) if z>1.4 else 0
            stubble=max(0,min(1,(1.55-z)/.04))*max(0,min(1,(-y-.035)/.05)) if 1.46<z<1.55 else 0
            variation=1+.045*math.sin(x*713+y*333+z*427)
            colors.data[i].color=((.28+.045*cheek-.045*stubble)*variation,(.145-.008*cheek-.025*stubble)*variation,(.09-.015*stubble)*variation,1)
    objects.append(ob); return ob

def ringmesh(name, rings, mat, weight, segments=32, folds=0):
    # Rings: (height, centreX, centreY, radiusX, radiusY).
    v=[]; uv=[]; w=[]
    for k,(z,cx,cy,rx,ry) in enumerate(rings):
        for j in range(segments+1):
            a=j/segments*math.tau
            f=1+folds*math.cos(a*10+.15*k)*(.5+.5*math.sin(k/(len(rings)-1)*math.pi))
            v.append((cx+rx*math.cos(a)*f,cy+ry*math.sin(a)*f,z))
            uv.append((j/segments,k/(len(rings)-1)))
            groups=weight(z)
            if name=='Wool tunic' and z<.83:
                influence=max(0,min(1,(.83-z)/.22))*.35
                groups={bone:value*(1-influence) for bone,value in groups.items()}
                groups['thigh.'+('R' if math.cos(a)>0 else 'L')]=influence
            w.append(groups)
    faces=[]
    for k in range(len(rings)-1):
        for j in range(segments):
            q=k*(segments+1)+j; faces.append((q,q+1,q+segments+2,q+segments+1))
    faces += [tuple(reversed(range(segments))),tuple((len(rings)-1)*(segments+1)+j for j in range(segments))]
    return finish(name,v,faces,mat,w,uv)

def fixed(bone): return lambda z:{bone:1}
def ball(name, center, scale, mat, bone, deform=None):
    verts=[]; weights=[]; uv=[]; faces=[]; nu=40; nv=24
    for i in range(nv+1):
        t=i/nv*math.pi
        for j in range(nu+1):
            a=j/nu*math.tau
            p=Vector((math.sin(t)*math.cos(a),math.sin(t)*math.sin(a),math.cos(t)))
            p=Vector((p.x*scale[0],p.y*scale[1],p.z*scale[2]))
            if deform: p=deform(p)
            verts.append(tuple(p+Vector(center))); weights.append({bone:1}); uv.append((j/nu,i/nv))
    for i in range(nv):
        for j in range(nu):
            q=i*(nu+1)+j; faces.append((q,q+nu+1,q+nu+2,q+1))
    return finish(name,verts,faces,mat,weights,uv)

def curve(name, coords, radius, mat,bone):
    cu=bpy.data.curves.new(name,'CURVE'); cu.dimensions='3D'; cu.bevel_depth=radius; cu.bevel_resolution=3; cu.resolution_u=6
    s=cu.splines.new('BEZIER'); s.bezier_points.add(len(coords)-1)
    for b,co in zip(s.bezier_points,coords): b.co=co; b.handle_left_type='AUTO'; b.handle_right_type='AUTO'
    ob=bpy.data.objects.new(name,cu); bpy.context.collection.objects.link(ob)
    bpy.context.view_layer.objects.active=ob; ob.select_set(True); bpy.ops.object.convert(target='MESH'); ob.select_set(False)
    ob=bpy.context.view_layer.objects.active; ob.data.materials.append(mat)
    if mat==skin:
        colors=ob.data.color_attributes.new(name='Skin tone',type='FLOAT_COLOR',domain='POINT')
        for c in colors.data:c.color=(.28,.145,.09,1)
    vg=ob.vertex_groups.new(name=bone); vg.add(list(range(len(ob.data.vertices))),1,'REPLACE'); objects.append(ob)

# Natural adult proportions; garment silhouette instead of a cylinder/capsule body.
ringmesh('Wool tunic',[(.61,0,0,.21,.125),(.65,0,0,.215,.13),(.76,0,0,.19,.12),(.87,0,0,.16,.11),(1.00,0,0,.155,.10),(1.13,0,0,.195,.115),(1.25,0,0,.205,.105),(1.31,0,0,.18,.09),(1.35,0,0,.085,.065)],cloth,lambda z:{'pelvis':1-max(0,min(1,(z-.86)/.22)),'chest':max(0,min(1,(z-.86)/.22))},folds=.045)
ringmesh('Belt',[(.943,0,0,.167,.112),(.977,0,0,.167,.112)],leather,fixed('pelvis'),folds=.005)
ringmesh('Linen neckline',[(1.334,0,0,.092,.071),(1.355,0,0,.080,.062)],linen,fixed('chest'))
# Skin color is vertex-authored in linear light, shared across continuous face/neck/hands.
vc=skin.node_tree.nodes.new('ShaderNodeVertexColor');vc.layer_name='Skin tone'
skin.node_tree.links.new(vc.outputs['Color'],skin.node_tree.nodes.get('Principled BSDF').inputs['Base Color'])
# Parse the same pinned CC0 anatomy with its body topology and original UVs.
source_v=[];source_uv=[];source_faces=[];source_groups={};group=''
for line in (ROOT/'tools/art/reference/makehuman-base.obj').read_text().splitlines():
    if line.startswith('v '):source_v.append(Vector(tuple(map(float,line.split()[1:4]))))
    elif line.startswith('vt '):source_uv.append(tuple(map(float,line.split()[1:3])))
    elif line.startswith('g '):group=line.strip()[2:]
    elif line.startswith('f '):
        face=[tuple(int(x)-1 for x in token.split('/')[:2]) for token in line.split()[1:]]
        source_groups.setdefault(group,[]).append(face)
def adapted(p):return Vector((p.x*.105,-(p.z-.35)*.105,1.40+(p.y-5.7)*.105))
def anatomy(name,faces,transform,mat,bone):
    ids=sorted({v for face in faces for v,uv in face});lookup={v:i for i,v in enumerate(ids)}
    verts=[tuple(transform(source_v[v])) for v in ids]
    ob=finish(name,verts,[tuple(lookup[v] for v,uv in face) for face in faces],mat,[{bone:1} for v in ids])
    for polygon,face in zip(ob.data.polygons,faces):
        for loop,(_,uv) in zip(polygon.loop_indices,face):ob.data.uv_layers.active.data[loop].uv=source_uv[uv]
    return ob
# Extend the natural lower neck boundary underneath the shirt; no exposed clavicle cut.
head_faces=[f for f in source_groups['body'] if all(source_v[v].y>5.7 for v,uv in f)]
head=anatomy('Anatomical head and neck',head_faces,adapted,skin,'head')
bm=bmesh.new();bm.from_mesh(head.data)
neck_edges=[e for e in bm.edges if e.is_boundary and all(v.co.z<1.445 for v in e.verts)]
assert len(neck_edges)>8, 'Missing lower neck boundary'
result=bmesh.ops.extrude_edge_only(bm,edges=neck_edges)
color_layer=bm.verts.layers.float_color.get('Skin tone')
for v in result['geom']:
    if isinstance(v,bmesh.types.BMVert):
        v.co.x*=.78;v.co.y=.005+(v.co.y-.005)*.78;v.co.z=1.325
        if color_layer:v[color_layer]=(.28,.145,.09,1)
bm.to_mesh(head.data);bm.free();head.data.update()
head.vertex_groups['head'].add(list(range(len(head.data.vertices))),1,'REPLACE')
sub=head.modifiers.new('Facial surface refinement','SUBSURF');sub.levels=1
bpy.context.view_layer.objects.active=head;bpy.ops.object.modifier_apply(modifier=sub.name)
# Small eyeballs sit behind the base mesh's sculpted eyelids.
for side in [-1,1]:
    x=side*.30775*.105; z=1.40+(7.28415-5.7)*.105; y=-(1.24535-.35)*.105
    ball('Eye',(x,y,z),(.0105,.0105,.0105),white,'head')
    ball('Iris',(x,y-.0102,z),(.004,.0015,.004),eye,'head')
# Hair follows the anatomical scalp instead of a separate spherical helmet.
hair_faces=[]
for f in head_faces:
    c=sum((adapted(source_v[v]) for v,uv in f),Vector())/len(f)
    threshold=1.625 if c.y<-.035 else 1.565
    threshold+=.006*math.sin(c.x*71)
    if c.z>threshold:hair_faces.append(f)
def hair_position(p):
    q=adapted(p); centre=Vector((0,-.005,1.59));d=q-centre
    return q+d.normalized()*.0025
hair_ob=anatomy('Close fitted irregular hairline',hair_faces,hair_position,hair,'head')
sub=hair_ob.modifiers.new('Scalp refinement','SUBSURF');sub.levels=2
bpy.context.view_layer.objects.active=hair_ob;bpy.ops.object.modifier_apply(modifier=sub.name)
# Fine strands follow the actual scalp surface; they do not enlarge it into a cap.
for i,f in enumerate(hair_faces):
    if i%3:continue
    a=hair_position(source_v[f[0][0]]);b=hair_position(source_v[f[1][0]])
    mid=(a+b)*.5;mid.z+=.0006
    curve('Short hair strand',[tuple(a),tuple(mid),tuple(b)],.00045,hair_strand,'head')
# Eyebrows and upper lid accents are actual small curved strands, not painted white eyes.
for side in [-1,1]:
    curve('Eyebrow',[(side*.017,-.096,1.586),(side*.034,-.101,1.590),(side*.052,-.092,1.583)],.0017,hair,'head')
    curve('Upper eyelid',[(side*.022,-.102,1.569),(side*.033,-.108,1.574),(side*.043,-.101,1.568)],.0008,skin,'head')
curve('Closed lips',[(-.020,-.104,1.511),(0,-.111,1.508),(.020,-.104,1.511)],.0011,lip,'head')
# Clothing seams, cuff trims, boot openings and soles.
for side,suffix in [(-1,'L'),(1,'R')]:
    x=side*.105
    def legweight(z,s=suffix):
        t=max(0,min(1,(z-.43)/.10)); return {'thigh.'+s:t,'shin.'+s:1-t}
    ringmesh('Linen trousers '+suffix,[(.20,x,.012,.049,.051),(.27,x,0,.058,.061),(.44,x,-.006,.066,.069),(.51,x,0,.072,.075),(.69,x,0,.076,.080)],linen,legweight,folds=.025)
    ringmesh('Leather boot '+suffix,[(.035,x,-.025,.068,.111),(.065,x,-.038,.073,.133),(.104,x,-.012,.068,.098),(.16,x,.005,.061,.061),(.25,x,.005,.062,.063)],leather,lambda z,s=suffix:{'shin.'+s:max(0,min(1,(z-.11)/.09)),'foot.'+s:1-max(0,min(1,(z-.11)/.09))},folds=.012)
    curve('Boot seam '+suffix,[(x,-.105,.06),(x,-.092,.10),(x,-.059,.17),(x,-.060,.24)],.0017,linen,'foot.'+suffix)
    # Arms descend with a slight natural outward angle. Skinning blends at the elbow.
    def armweight(z,s=suffix):
        t=max(0,min(1,(z-1.00)/.09)); return {'upper_arm.'+s:t,'forearm.'+s:1-t}
    ringmesh('Sleeve '+suffix,[(.84,side*.247,-.008,.045,.047),(.90,side*.252,-.006,.052,.052),(1.02,side*.256,0,.063,.060),(1.16,side*.239,0,.071,.070),(1.27,side*.203,0,.080,.073),(1.30,side*.18,0,.06,.06),(1.32,side*.17,0,.02,.02)],cloth,armweight,folds=.025)
    ringmesh('Cuff '+suffix,[(.837,side*.247,-.008,.047,.049),(.863,side*.249,-.008,.047,.049)],linen,fixed('forearm.'+suffix))
    # Anatomical palms, thumbs and fingers; align the source forearm to the existing wrist.
    def joint_centre(name):
        ids={v for f in source_groups[name] for v,uv in f}
        return sum((adapted(source_v[v]) for v in ids),Vector())/len(ids)
    src_side='l' if side>0 else 'r'
    wrist=joint_centre('joint-'+src_side+'-hand');elbow=joint_centre('joint-'+src_side+'-elbow')
    rotation=(wrist-elbow).rotation_difference(Vector((-side*.009,-.008,-.20)))
    hand_faces=[f for f in source_groups['body'] if all(source_v[v].x*side>3.90 for v,uv in f)]
    anatomy('Anatomical hand '+suffix,hand_faces,lambda p:rotation@(adapted(p)-wrist)+Vector((side*.247,-.008,.84)),skin,'hand.'+suffix)
    curve('Tunic side seam '+suffix,[(side*.208,0,.63),(side*.19,0,.76),(side*.16,0,.90)],.0017,linen,'pelvis')
# Small tied belt pouch; restrained detailing.
ball('Leather pouch',(.17,-.035,.87),(.045,.035,.065),leather,'pelvis')
curve('Pouch drawstring',[(.14,-.06,.91),(.17,-.076,.92),(.20,-.06,.91)],.002,linen,'pelvis')
curve('Neck slit',[(0,-.069,1.345),(0,-.103,1.29),(0,-.112,1.27)],.0025,linen,'chest')
# Fuse torso/sleeves into a single garment surface with real thickness at the hem.
cloth_parts=[ob for ob in objects if ob.name=='Wool tunic' or ob.name.startswith('Sleeve ')]
cloth_samples=[]
for ob in cloth_parts:
    groups={g.index:g.name for g in ob.vertex_groups}
    for v in ob.data.vertices:cloth_samples.append((v.co.copy(),{groups[g.group]:g.weight for g in v.groups}))
kd=KDTree(len(cloth_samples))
for i,(co,_) in enumerate(cloth_samples):kd.insert(co,i)
kd.balance()
bpy.ops.object.select_all(action='DESELECT')
for ob in cloth_parts:ob.select_set(True)
bpy.context.view_layer.objects.active=cloth_parts[0];bpy.ops.object.join();garment=cloth_parts[0]
objects=[ob for ob in objects if ob not in cloth_parts]+[garment]
remesh=garment.modifiers.new('Continuous cloth topology','REMESH');remesh.mode='VOXEL';remesh.voxel_size=.0075;remesh.use_smooth_shade=True
bpy.ops.object.modifier_apply(modifier=remesh.name)
smooth=garment.modifiers.new('Soften joined shoulder seams','SMOOTH');smooth.factor=.5;smooth.iterations=3
bpy.ops.object.modifier_apply(modifier=smooth.name)
dec=garment.modifiers.new('Runtime cloth density','DECIMATE');dec.ratio=.45
bpy.ops.object.modifier_apply(modifier=dec.name)
garment.vertex_groups.clear()
for v in garment.data.vertices:
    weights={};neighbours=kd.find_n(v.co,6);total=sum(1/max(d,.003)**2 for _,_,d in neighbours)
    for _,i,d in neighbours:
        for bone,w in cloth_samples[i][1].items():weights[bone]=weights.get(bone,0)+w/max(d,.003)**2/total
    for bone,w in weights.items():
        vg=garment.vertex_groups.get(bone) or garment.vertex_groups.new(name=bone);vg.add([v.index],w,'REPLACE')
layer=garment.data.uv_layers.new(name='Cloth metres')
for polygon in garment.data.polygons:
    polygon.use_smooth=True
    for li in polygon.loop_indices:
        p=garment.data.vertices[garment.data.loops[li].vertex_index].co
        layer.data[li].uv=(math.atan2(p.y,p.x)/math.tau,p.z*1.3)
# Batch the character by material, retaining UVs, skin colors and bone weights.
combined=[]
batches=[[ob for ob in objects if ob.data.materials[0]==mat] for mat in [skin,leather,linen,hair,hair_strand,eye,white,lip,metal,cloth]]
for parts in batches:
    if not parts:continue
    bpy.ops.object.select_all(action='DESELECT')
    for ob in parts:ob.select_set(True)
    bpy.context.view_layer.objects.active=parts[0]
    if len(parts)>1:bpy.ops.object.join()
    combined.append(parts[0])
objects=combined
# A simple deform rig, exported as glTF skin, with explicit elbow/knee/ankle joints.
bpy.ops.object.select_all(action='DESELECT')
armature=bpy.data.armatures.new('Coastal resident skeleton'); rig=bpy.data.objects.new('ResidentRig',armature); bpy.context.collection.objects.link(rig)
bpy.context.view_layer.objects.active=rig; rig.select_set(True); bpy.ops.object.mode_set(mode='EDIT')
def bone(name,head,tail,parent=None):
    b=armature.edit_bones.new(name); b.head=head; b.tail=tail
    if parent: b.parent=armature.edit_bones[parent]
bone('pelvis',(0,0,.85),(0,0,1.05)); bone('chest',(0,0,1.05),(0,0,1.34),'pelvis'); bone('head',(0,0,1.34),(0,0,1.68),'chest')
for side,s in [(-1,'L'),(1,'R')]:
    bone('upper_arm.'+s,(side*.205,0,1.27),(side*.256,0,1.04),'chest')
    bone('forearm.'+s,(side*.256,0,1.04),(side*.247,-.008,.84),'upper_arm.'+s)
    bone('hand.'+s,(side*.247,-.008,.84),(side*.247,-.012,.74),'forearm.'+s)
    bone('thigh.'+s,(side*.105,0,.85),(side*.105,-.006,.47),'pelvis')
    bone('shin.'+s,(side*.105,-.006,.47),(side*.105,.005,.14),'thigh.'+s)
    bone('foot.'+s,(side*.105,.005,.14),(side*.105,-.12,.07),'shin.'+s)
bpy.ops.object.mode_set(mode='OBJECT')
for ob in objects:
    mod=ob.modifiers.new('Weighted skeleton','ARMATURE'); mod.object=rig; ob.parent=rig
    # All explicit mesh winding is authored; recalculate outside consistently.
    bpy.context.view_layer.objects.active=ob; rig.select_set(False); ob.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT'); bpy.ops.mesh.select_all(action='SELECT'); bpy.ops.mesh.normals_make_consistent(inside=False); bpy.ops.object.mode_set(mode='OBJECT'); ob.select_set(False)
# Save editable source and a self-contained runtime export; runtime does not need Blender.
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'tools/art/coastal-resident.blend'))
bpy.ops.export_scene.gltf(filepath=str(ROOT/'assets/models/coastal-resident.glb'),export_format='GLB',export_animations=False,export_yup=True,export_skins=True,export_all_influences=False)
print('Exported rigged coastal resident study')
