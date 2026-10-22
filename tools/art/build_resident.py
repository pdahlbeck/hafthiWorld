"""Original coastal resident study, authored in Blender metres. Blender 4.3+.
Run: blender --background --python tools/art/build_resident.py
Project-authored garments/rig; the anatomical head uses the documented CC0 MakeHuman excerpt.
"""
import bpy, math
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
skin=material('Warm matte skin',(.49,.30,.21),.86)
leather=material('Worn brown leather',(.105,.065,.038),.92)
linen=material('Natural linen',(.56,.50,.38))
hair=material('Chestnut hair',(.075,.045,.026))
eye=material('Small dark iris',(.035,.042,.030),.7)
white=material('Warm eye white',(.43,.40,.32),.7)
lip=material('Muted lips',(.32,.16,.12))
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
            w.append(weight(z))
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
    vg=ob.vertex_groups.new(name=bone); vg.add(list(range(len(ob.data.vertices))),1,'REPLACE'); objects.append(ob)

# Natural adult proportions; garment silhouette instead of a cylinder/capsule body.
ringmesh('Wool tunic',[(.61,0,0,.21,.125),(.65,0,0,.215,.13),(.76,0,0,.19,.12),(.87,0,0,.16,.11),(1.00,0,0,.155,.10),(1.13,0,0,.195,.115),(1.25,0,0,.205,.105),(1.31,0,0,.18,.09),(1.35,0,0,.085,.065)],cloth,lambda z:{'pelvis':1} if z<.92 else {'chest':1},folds=.045)
ringmesh('Belt',[(.943,0,0,.167,.112),(.977,0,0,.167,.112)],leather,fixed('pelvis'),folds=.005)
ringmesh('Linen neckline',[(1.334,0,0,.092,.071),(1.355,0,0,.080,.062)],linen,fixed('chest'))
ringmesh('Neck',[(1.32,0,0,.060,.061),(1.42,0,.004,.058,.060)],skin,fixed('head'))
# CC0 anatomical head/neck, with authored wool costume and a project-specific rig.
head_v=[]; head_uv=[]; head_faces=[]; face_uv=[]
for line in (ROOT/'tools/art/reference/makehuman-head.obj').read_text().splitlines():
    if line.startswith('v '):
        x,y,z=map(float,line.split()[1:4]); head_v.append((x*.105,-(z-.35)*.105,1.40+(y-5.7)*.105))
    elif line.startswith('vt '): head_uv.append(tuple(map(float,line.split()[1:3])))
    elif line.startswith('f '):
        face=[tuple(int(x)-1 for x in token.split('/')[:2]) for token in line.split()[1:]]
        head_faces.append(tuple(x[0] for x in face)); face_uv.append(tuple(x[1] for x in face))
head=finish('Adapted CC0 anatomical head',head_v,head_faces,skin,[{'head':1} for _ in head_v])
for polygon,indices in zip(head.data.polygons,face_uv):
    for loop,index in zip(polygon.loop_indices,indices):head.data.uv_layers.active.data[loop].uv=head_uv[index]
# Small eyeballs sit behind the base mesh's sculpted eyelids.
for side in [-1,1]:
    x=side*.30775*.105; z=1.40+(7.28415-5.7)*.105; y=-(1.24535-.35)*.105
    ball('Eye',(x,y,z),(.0105,.0105,.0105),white,'head')
    ball('Iris',(x,y-.0102,z),(.004,.0015,.004),eye,'head')
# Close-cropped cap with receding front/sideburns, not a spherical helmet.
verts=[]; faces=[]; weights=[]
for i in range(13):
    for j in range(49):
        a=j/48*math.tau; end=1.27+.38*max(0,math.sin(a))
        t=.015+i/12*end
        r=1+.012*math.sin(a*21+i*.4)
        verts.append((.104*math.sin(t)*math.cos(a)*r,-.025+.135*math.sin(t)*math.sin(a)*r,1.553+.146*math.cos(t)))
        weights.append({'head':1})
for i in range(12):
    for j in range(48):
        q=i*49+j; faces.append((q,q+49,q+50,q+1))
finish('Textured hair cap',verts,faces,hair,weights)
# Clothing seams, cuff trims, boot openings and soles.
for side,suffix in [(-1,'L'),(1,'R')]:
    x=side*.105
    def legweight(z,s=suffix):
        t=max(0,min(1,(z-.43)/.10)); return {'thigh.'+s:t,'shin.'+s:1-t}
    ringmesh('Linen trousers '+suffix,[(.13,x,.012,.058,.060),(.27,x,0,.063,.063),(.44,x,-.006,.073,.073),(.51,x,0,.078,.078),(.69,x,0,.086,.087),(.86,x,0,.090,.090)],linen,legweight,folds=.025)
    ringmesh('Leather boot '+suffix,[(.035,x,-.025,.068,.111),(.065,x,-.038,.073,.133),(.104,x,-.012,.068,.098),(.16,x,.005,.061,.061),(.25,x,.005,.062,.063)],leather,fixed('foot.'+suffix),folds=.012)
    curve('Boot seam '+suffix,[(x,-.105,.06),(x,-.092,.10),(x,-.059,.17),(x,-.060,.24)],.0017,linen,'foot.'+suffix)
    # Arms descend with a slight natural outward angle. Skinning blends at the elbow.
    def armweight(z,s=suffix):
        t=max(0,min(1,(z-1.00)/.09)); return {'upper_arm.'+s:t,'forearm.'+s:1-t}
    ringmesh('Sleeve '+suffix,[(.84,side*.247,-.008,.045,.047),(.90,side*.252,-.006,.052,.052),(1.02,side*.256,0,.063,.060),(1.16,side*.239,0,.071,.070),(1.27,side*.203,0,.080,.073),(1.30,side*.18,0,.06,.06),(1.32,side*.17,0,.02,.02)],cloth,armweight,folds=.025)
    ringmesh('Cuff '+suffix,[(.837,side*.247,-.008,.047,.049),(.863,side*.249,-.008,.047,.049)],linen,fixed('forearm.'+suffix))
    ball('Hand '+suffix,(side*.247,-.014,.790),(.038,.026,.060),skin,'hand.'+suffix)
    ball('Thumb '+suffix,(side*.216,-.027,.799),(.014,.018,.034),skin,'hand.'+suffix)
    curve('Tunic side seam '+suffix,[(side*.208,0,.63),(side*.19,0,.76),(side*.16,0,.90)],.0017,linen,'pelvis')
# Small tied belt pouch; restrained detailing.
ball('Leather pouch',(.17,-.035,.87),(.045,.035,.065),leather,'pelvis')
curve('Pouch drawstring',[(.14,-.06,.91),(.17,-.076,.92),(.20,-.06,.91)],.002,linen,'pelvis')
curve('Neck slit',[(0,-.069,1.345),(0,-.103,1.29),(0,-.112,1.27)],.0025,linen,'chest')
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
