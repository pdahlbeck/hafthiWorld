"""Adapt MIT Rocketbox skin/rig/motion to the project's costume and metre-scale glTF.
Source cache: HAFTHI_ROCKETBOX_SOURCE (see fetch_rocketbox.py). Blender 4.3.2.
"""
import bpy, math, os, json
from pathlib import Path
from mathutils import Vector, Matrix
ROOT=Path(__file__).resolve().parents[2]
CACHE=Path(os.environ.get('HAFTHI_ROCKETBOX_SOURCE',str(ROOT/'.cache/rocketbox')))
bpy.context.preferences.filepaths.save_version=0
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
bpy.ops.import_scene.fbx(filepath=str(CACHE/'adult05/Male_Adult_05.fbx'),use_anim=False)
source_rig=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE')
source_mesh=next(o for o in bpy.context.scene.objects if o.type=='MESH')
source_world=source_rig.matrix_world.copy()
source_bones={b.name:(source_world@b.matrix_local) for b in source_rig.data.bones}
source_parents={b.name:b.parent.name if b.parent else None for b in source_rig.data.bones}
source_lengths={b.name:b.length*.01 for b in source_rig.data.bones}
# Keep the actual source head/neck, hair cards and anatomical hands, not modern clothing.
def hand_weight(v):
    return sum(g.weight for g in v.groups if any(s in source_mesh.vertex_groups[g.group].name for s in [' Hand',' Finger']))
hand_planes=[]
for side in ['L','R']:
    name='Bip01 '+side+' Hand';wrist=source_bones[name].translation
    fingers=[source_bones[b.name].translation for b in source_rig.data.bones if b.parent and b.parent.name==name and 'Finger' in b.name]
    forward=(sum(fingers,Vector())/len(fingers)-wrist).normalized()
    hand_planes.append((wrist,forward))
def bare_hand(p):
    # Remove modern wristwatch/cuff geometry even when it has hand skin weights.
    return any(all((source_mesh.matrix_world@source_mesh.data.vertices[v].co-wrist).dot(forward)>.012
                   for v in p.vertices) for wrist,forward in hand_planes)
faces=[]
for p in source_mesh.data.polygons:
    if p.material_index in (1,2) or (all(hand_weight(source_mesh.data.vertices[v])>.55 for v in p.vertices) and bare_hand(p)):faces.append(p)

# Load the original editable costume, with its own 15-joint rest pose.
with bpy.data.libraries.load(str(ROOT/'tools/art/coastal-resident.blend'),link=False) as (available,loaded):
    loaded.objects=['ResidentRig','Wool tunic','Linen neckline','Belt']
donor={o.name:o for o in loaded.objects}
old_rig=donor['ResidentRig']
for o in donor.values():bpy.context.collection.objects.link(o)
# The donor uses opposite left/right naming; map by physical side, not spelling.
map_bone={'pelvis':'Bip01 Pelvis','chest':'Bip01 Spine1','head':'Bip01 Head'}
for old_side,new_side in [('L','R'),('R','L')]:
    for old,new in [('upper_arm','UpperArm'),('forearm','Forearm'),('hand','Hand'),('thigh','Thigh'),('shin','Calf'),('foot','Foot')]:map_bone[old+'.'+old_side]='Bip01 '+new_side+' '+new
next_joint={'upper_arm':' Forearm','forearm':' Hand','thigh':' Calf','shin':' Foot'}
# Retarget the garment's rest surface from the donor's lowered arms to Rocketbox's A-pose.
scale=1.812/1.70
old_transforms={}
for name,target in map_bone.items():
    bone=old_rig.data.bones[name]
    origin=old_rig.matrix_world@bone.head_local
    direction=old_rig.matrix_world.to_3x3()@(bone.tail_local-bone.head_local)
    target_origin=source_bones[target].translation
    stem=name.split('.')[0]
    if stem in next_joint:
        child=target.rsplit(' ',1)[0]+next_joint[stem]
        target_direction=source_bones[child].translation-target_origin
    elif stem=='foot':target_direction=Vector((0,-.13,-.02))
    else:target_direction=Vector((0,0,1))
    rotation=direction.normalized().rotation_difference(target_direction.normalized())
    old_transforms[name]=(origin,target_origin,rotation)

costume_data=[]
for ob in [donor['Wool tunic'],donor['Linen neckline'],donor['Belt']]:
    groups={g.index:g.name for g in ob.vertex_groups};positions=[];weights=[]
    for v in ob.data.vertices:
        p=ob.matrix_world@v.co;point=Vector();groups_out={}
        original={groups[g.group]:g.weight for g in v.groups if groups[g.group] in old_transforms and g.weight>0}
        if ob.name=='Wool tunic':
            # Voxel-fused donor sleeves inherited nearby wrist weights on the skirt.
            # Rebind by garment region, never by proximity to a lowered hand.
            z=p.z;side='R' if p.x>0 else 'L'
            radius=.16 if z<1.0 else .16+min(1,(z-1.0)/.25)*.045
            arm=max(0,min(1,(abs(p.x)-radius-.012)/.035)) if z>.83 else 0
            if z>1.27:arm*=max(0,min(1,(1.34-z)/.07))
            chest=max(0,min(1,(z-.86)/.22))
            thigh=max(0,min(1,(.83-z)/.22))*.28
            elbow=max(0,min(1,(z-1.00)/.09))
            original={'pelvis':(1-arm)*(1-chest)*(1-thigh),'chest':(1-arm)*chest,
                      'thigh.'+side:(1-arm)*(1-chest)*thigh,
                      'upper_arm.'+side:arm*elbow,'forearm.'+side:arm*(1-elbow)}
        if ob.name=='Linen neckline' and p.z>.51 and any(n.startswith('thigh.') and w>.5 for n,w in original.items()):
            # Concealed trouser tops must stay below the skirt even in a forward
            # stride. Compress upper rings rather than collapsing them to a plane.
            p.z=.51+(p.z-.51)*(.085/.18)
            assert p.z<=.6001,(ob.name,v.index,p.z)
        # A loose woven tunic should not cinch into an hourglass between ribs
        # and belt. Adjust the donor surface before retargeting; preserve sleeve
        # and cuff geometry, garment thickness and its carefully fitted hands.
        if ob.name in ('Wool tunic','Belt') and .80 < p.z < 1.12 and abs(p.x)<.19:
            fullness=max(0,1-abs(p.z-.96)/.16)
            p.x*=1+.08*fullness
            p.y*=1+.06*fullness
        total=sum(original.values())
        assert total>.99,(ob.name,v.index,total)
        for name,weight in original.items():
            if weight<=0:continue
            weight/=total
            origin,target,rotation=old_transforms[name]
            point+=(target+rotation@(p-origin)*scale)*weight
            mapped=map_bone[name];groups_out[mapped]=groups_out.get(mapped,0)+weight
        # The collar must follow the upper chest like the anatomical neck base,
        # rather than opening a gap as Spine2 bends relative to Spine1.
        upper=max(0,min(1,(p.z-1.12)/.18))
        chest_weight=groups_out.get('Bip01 Spine1',0)*upper
        if chest_weight:
            groups_out['Bip01 Spine1']-=chest_weight
            groups_out['Bip01 Spine2']=chest_weight
        positions.append(point);weights.append(groups_out)
    costume_data.append((ob,positions,weights))
# Normalize the complete dressed character, with its sole at zero and height 1.7 m.
head_positions=[source_mesh.matrix_world@source_mesh.data.vertices[v].co for p in faces if p.material_index==1 for v in p.vertices]
floor=min(p.z for ob,points,w in costume_data for p in points)
top=max(p.z for p in head_positions)
normal_scale=1.70/(top-floor)
normal=Matrix.Diagonal((normal_scale,normal_scale,normal_scale,1))@Matrix.Translation((0,0,-floor))
# Lower the complete skull, eyes and hair rigidly: no facial scaling. Keep
# shoulders/arms in their original rest positions and shorten only the neck.
HEAD_DROP=.04
head_bones={'Bip01 Head'}
for name in source_parents:
    parent=source_parents[name]
    while parent:
        if parent=='Bip01 Head':head_bones.add(name);break
        parent=source_parents[parent]
rest={}
for name,matrix in source_bones.items():
    transformed=normal@matrix
    clean=transformed.to_quaternion().to_matrix().to_4x4();clean.translation=transformed.translation
    if name in head_bones:clean.translation.z-=HEAD_DROP
    rest[name]=clean
# Output rig has identity object transforms and metre-scale bone translations.
armature=bpy.data.armatures.new('Coastal motion skeleton');rig=bpy.data.objects.new('CoastalRig',armature)
bpy.context.collection.objects.link(rig)
bpy.ops.object.select_all(action='DESELECT');rig.select_set(True);bpy.context.view_layer.objects.active=rig
bpy.ops.object.mode_set(mode='EDIT')
for name,matrix in rest.items():
    b=armature.edit_bones.new(name)
    # A new EditBone has zero length: assign length BEFORE matrix, otherwise
    # Blender cannot recover its Y axis and silently changes the rest rotation.
    b.length=max(.005,source_lengths[name]*normal_scale)
    clean=matrix.to_quaternion().to_matrix().to_4x4();clean.translation=matrix.translation;b.matrix=clean
for name,parent in source_parents.items():
    if parent:armature.edit_bones[name].parent=armature.edit_bones[parent]
bpy.ops.object.mode_set(mode='OBJECT')
for name,matrix in rest.items():
    actual=rig.data.bones[name].matrix_local
    assert abs(actual.to_quaternion().dot(matrix.to_quaternion()))>1-.000001,name
    assert (actual.translation-matrix.translation).length<.00001,name
output=[rig]

def image(name,normal=False):
    im=bpy.data.images.load(str(CACHE/'adult05/Textures'/name),check_existing=True)
    if normal:im.colorspace_settings.name='Non-Color'
    im.pack();return im

def skin_material(name,prefix,alpha=False):
    m=bpy.data.materials.new(name);m.use_nodes=True
    bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Roughness'].default_value=.78
    bs.inputs['Specular IOR Level'].default_value=.3
    tex=m.node_tree.nodes.new('ShaderNodeTexImage');tex.image=image(prefix+'_color.tga')
    m.node_tree.links.new(tex.outputs['Color'],bs.inputs['Base Color'])
    if alpha:
        m.node_tree.links.new(tex.outputs['Alpha'],bs.inputs['Alpha']);m.surface_render_method='DITHERED';m.use_backface_culling=False
    else:
        texn=m.node_tree.nodes.new('ShaderNodeTexImage');texn.image=image(prefix+'_normal.tga',True)
        n=m.node_tree.nodes.new('ShaderNodeNormalMap');n.inputs['Strength'].default_value=.65
        m.node_tree.links.new(texn.outputs['Color'],n.inputs['Color']);m.node_tree.links.new(n.outputs['Normal'],bs.inputs['Normal'])
    return m
body=skin_material('Resident skin hands','m009_body')
head=skin_material('Resident skin head','m009_head')
hair=skin_material('Resident hair','m009_opacity',True)

def bind(ob,weights):
    for vg in list(ob.vertex_groups):ob.vertex_groups.remove(vg)
    for i,groups in enumerate(weights):
        for bone,w in groups.items():
            if w>0.00001:
                vg=ob.vertex_groups.get(bone) or ob.vertex_groups.new(name=bone);vg.add([i],w,'REPLACE')
    for mod in list(ob.modifiers):ob.modifiers.remove(mod)
    mod=ob.modifiers.new('Coastal skin','ARMATURE');mod.object=rig;ob.parent=rig
    output.append(ob)
# Preserve source UV loops and material partitions exactly for the licensed skin.
ids=sorted({v for p in faces for v in p.vertices});lookup={v:i for i,v in enumerate(ids)}
head_ids={v for p in faces if p.material_index in (1,2) for v in p.vertices}
verts=[]
for v in ids:
    point=normal@(source_mesh.matrix_world@source_mesh.data.vertices[v].co)
    if v in head_ids:point.z-=HEAD_DROP
    verts.append(tuple(point))
mesh=bpy.data.meshes.new('Textured anatomy');mesh.from_pydata(verts,[],[tuple(lookup[v] for v in p.vertices) for p in faces]);mesh.update()
ob=bpy.data.objects.new('Textured anatomy',mesh);bpy.context.collection.objects.link(ob)
for m in [body,head,hair]:mesh.materials.append(m)
uv=mesh.uv_layers.new(name='Source skin UV')
for polygon,src in zip(mesh.polygons,faces):
    polygon.material_index=src.material_index;polygon.use_smooth=True
    for li,sli in zip(polygon.loop_indices,src.loop_indices):uv.data[li].uv=source_mesh.data.uv_layers.active.data[sli].uv
weights=[{source_mesh.vertex_groups[g.group].name:g.weight for g in source_mesh.data.vertices[v].groups if source_mesh.vertex_groups[g.group].name in rest} for v in ids]
# The source head was cut to fit a modern shirt: its rear neck rim ends about
# 5 cm above our collar. Extend that existing, UV-mapped anatomical surface below
# the tunic, retaining the head position/proportions and a continuous skin mesh.
# Only the open lower neck rim moves; eyes, jaw, head and hands stay untouched.
edge_use={}
for polygon in mesh.polygons:
    if polygon.material_index==1:
        for edge in polygon.edge_keys:edge_use[edge]=edge_use.get(edge,0)+1
neck_edges=[edge for edge,count in edge_use.items() if count==1 and all(mesh.vertices[i].co.z<1.48 for i in edge)]
neck_rim=sorted({i for edge in neck_edges for i in edge})
assert len(neck_edges)==len(neck_rim)==20,'Expected the original closed lower neck rim'
assert all(sum(i in edge for edge in neck_edges)==2 for i in neck_rim)
centre=sum((mesh.vertices[i].co for i in neck_rim),Vector())/len(neck_rim)
neck_floor=min(mesh.vertices[i].co.z for i in neck_rim)-.075
for i in neck_rim:
    v=mesh.vertices[i]
    v.co.x=centre.x+(v.co.x-centre.x)*.78
    v.co.y=centre.y+(v.co.y-centre.y)*.78
    v.co.z=neck_floor
    weights[i]={'Bip01 Spine2':1.0}
mesh.update()
bind(ob,weights)
for ob,positions,weights in costume_data:
    # Copy data so the donor source stays untouched and has no second armature.
    ob.data=ob.data.copy()
    for v,p in zip(ob.data.vertices,positions):v.co=normal@p
    ob.matrix_world=Matrix.Identity(4)
    for p in ob.data.polygons:p.use_smooth=True
    bind(ob,weights)
# A metre-scale prop socket cancels Blender's Z-up conversion for native Bevy tools.
hand_name='Bip01 L Hand'
finger_roots=[b for b in source_rig.data.bones if b.parent and b.parent.name==hand_name and 'Finger' in b.name]
palm=sum((source_bones[b.name].translation for b in finger_roots),Vector())/max(1,len(finger_roots))
palm=(palm+source_bones[hand_name].translation)*.5
socket=bpy.data.objects.new('AxeGrip',None);bpy.context.collection.objects.link(socket)
socket.parent=rig;socket.parent_type='BONE';socket.parent_bone=hand_name
socket.matrix_world=Matrix.Translation(normal@palm)@Matrix.Rotation(math.pi/2,4,'X')
output.append(socket)
# Remove all source objects; source imports below are transient and never exported.
for ob in list(bpy.context.scene.objects):
    if ob not in output:bpy.data.objects.remove(ob,do_unlink=True)

# Rocketbox FBXs share anatomical local axes (the limb direction is local +X),
# but their bind poses differ. Transfer absolute world rotations, not deltas from
# each clip's first/rest pose: subtracting that pose leaves idle arms stuck in A-pose.
# Target rest translations preserve this avatar's limb lengths.
clips=[]
for label,filename in [('Idle','m_idle_breathe_01.max.fbx'),('Walk','m_walk_neutral_01.max.fbx'),('Work','m_work_mid.max.fbx'),('Table','m_work_table.max.fbx')]:
    previous=set(bpy.data.objects)
    bpy.ops.import_scene.fbx(filepath=str(CACHE/'animations'/filename))
    imported=[o for o in bpy.data.objects if o not in previous]
    src=next(o for o in imported if o.type=='ARMATURE' and o.animation_data and o.animation_data.action)
    source_action=src.animation_data.action;start,end=map(int,source_action.frame_range)
    fps=bpy.context.scene.render.fps/bpy.context.scene.render.fps_base
    bpy.context.scene.frame_set(start);bpy.context.view_layer.update()
    first_position=src.matrix_world.translation.copy()
    action=bpy.data.actions.new(label);action.use_fake_user=True
    rig.animation_data_create();rig.animation_data.action=action
    samples=[];roots=[]
    for frame in range(start,end+1):
        bpy.context.scene.frame_set(frame);bpy.context.view_layer.update()
        roots.append(src.matrix_world.translation.copy());quats={}
        for name in rest:
            q=(src.matrix_world@src.pose.bones[name].matrix).to_quaternion()
            quats[name]=q
        samples.append(quats)
    # A short seam blend closes looping clips without a visible end-frame pop.
    seam=min(5,max(1,len(samples)//8))
    for index,sample in enumerate(samples):
        f=(index/fps)*30.0
        seam_t=max(0,(index-(len(samples)-1-seam))/seam)
        poses={}
        bob=(roots[index].z-first_position.z)*normal_scale*(1-seam_t)
        rig.location=Vector((0,0,bob))
        rig.keyframe_insert('location',frame=f,group='Vertical body motion')
        for name,matrix in rest.items():
            parent=source_parents[name]
            rel=rest[parent].inverted()@matrix if parent else matrix
            parent_pose=poses[parent] if parent else Matrix.Identity(4)
            goal=sample[name].slerp(samples[0][name],seam_t)
            rotation=rel.to_quaternion().inverted()@parent_pose.to_quaternion().inverted()@goal
            pb=rig.pose.bones[name];pb.rotation_mode='QUATERNION';pb.rotation_quaternion=rotation
            pb.location=Vector();pb.scale=Vector((1,1,1))
            pb.keyframe_insert('rotation_quaternion',frame=f,group=name)
            poses[name]=parent_pose@rel@Matrix.Translation(pb.location)@rotation.to_matrix().to_4x4()
    for curve in action.fcurves:
        for key in curve.keyframe_points:key.interpolation='LINEAR'
    duration=(end-start)/fps
    distance=(Vector((roots[-1].x,roots[-1].y))-Vector((roots[0].x,roots[0].y))).length*normal_scale
    clips.append({'name':label,'duration':duration,'source':filename,'source_fps':fps,'walk_speed':distance/duration if duration else 0})
    for ob in imported:bpy.data.objects.remove(ob,do_unlink=True)
# Rest source objects/actions are excluded; only the baked metre-scale rig is exported.
for ob in output:
    bpy.context.view_layer.objects.active=ob
    if ob.type=='MESH':
        bpy.ops.object.select_all(action='DESELECT');ob.select_set(True)
        bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT')
for action in list(bpy.data.actions):
    if action.name not in ('Idle','Walk','Work','Table'):bpy.data.actions.remove(action)
rig.animation_data.action=bpy.data.actions['Idle']
bpy.context.scene.render.fps=30;bpy.context.scene.frame_set(0)
bpy.ops.object.select_all(action='DESELECT')
for ob in output:ob.select_set(True)
bpy.context.view_layer.objects.active=rig
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'tools/art/textured-resident.blend'),compress=True)
bpy.ops.export_scene.gltf(filepath=str(ROOT/'assets/models/textured-resident.glb'),export_format='GLB',use_selection=True,export_animations=True,export_animation_mode='ACTIONS',export_frame_range=False,export_skins=True,export_yup=True,export_materials='EXPORT',export_bake_animation=True,export_copyright=(ROOT/'assets/licenses/Microsoft-Rocketbox-MIT.md').read_text())
manifest={'source_commit':'0943055db6ec570bcef9f2c8b41c9e5467c808f9','height':1.70-HEAD_DROP,'joints':len(rest),'clips':clips,'walk_source_speed':clips[1]['walk_speed'],'foot_height':{side:rest['Bip01 '+side+' Foot'].translation.z for side in ['L','R']}}
(ROOT/'assets/models/resident-motion.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('EXPORTED TEXTURED RESIDENT',manifest)
