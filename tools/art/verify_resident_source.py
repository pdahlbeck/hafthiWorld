"""Check cloth continuity and hidden trouser clearance in the editable rest pose.
blender --background tools/art/coastal-resident.blend --python-exit-code 1 --python tools/art/verify_resident_source.py
"""
import bpy
from mathutils.bvhtree import BVHTree

cloth = bpy.data.objects['Wool tunic'].data
adjacency = [set() for _ in cloth.vertices]
for edge in cloth.edges:
    a, b = edge.vertices
    adjacency[a].add(b)
    adjacency[b].add(a)
visited = set()
components = 0
for vertex in range(len(adjacency)):
    if vertex in visited:
        continue
    components += 1
    stack = [vertex]
    visited.add(vertex)
    while stack:
        for neighbour in adjacency[stack.pop()]:
            if neighbour not in visited:
                visited.add(neighbour)
                stack.append(neighbour)
assert components == 1, f'Tunic has {components} disconnected pieces'

surface = BVHTree.FromPolygons([v.co for v in cloth.vertices], [p.vertices[:] for p in cloth.polygons])
linen = bpy.data.objects['Linen neckline']
thighs = {g.index for g in linen.vertex_groups if g.name.startswith('thigh.')}
checked = 0
for vertex in linen.data.vertices:
    # Above the visible hem, trousers must stay inside the garment in rest pose.
    if vertex.co.z < .625 or not any(g.group in thighs and g.weight > 0 for g in vertex.groups):
        continue
    position, normal, _, distance = surface.find_nearest(vertex.co)
    assert position is not None
    signed = (vertex.co - position).dot(normal)
    assert signed < .002, f'Trousers poke through tunic at {tuple(vertex.co)}: {signed:.4f} m'
    checked += 1
assert checked > 50, 'No hidden trouser region checked'
print(f'Continuous tunic: {len(cloth.vertices)} vertices, 1 component; {checked} hidden trouser vertices inside cloth')
