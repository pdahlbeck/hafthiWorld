"""Original analytic 1.2 m reed micro-surface, independent of the generated albedo.
No inferred/displaced photo geometry: this only supplies millimetric parallel grain.
Requires numpy and Pillow; not used at runtime.
"""
from pathlib import Path
import numpy as np
from PIL import Image
n = 1024
y, x = np.mgrid[:n, :n] / n
phase = x * 2*np.pi*208 + .7*np.sin(y*2*np.pi*3) + .25*np.sin(y*2*np.pi*13)
h = .00065*np.sin(phase) + .00018*np.sin(phase*2)
dx = (np.roll(h,-1,1)-np.roll(h,1,1)) * n / (2*1.2)
dy = (np.roll(h,-1,0)-np.roll(h,1,0)) * n / (2*1.2)
normal = np.stack([-dx,-dy,np.ones_like(h)],-1)
normal /= np.linalg.norm(normal,axis=-1,keepdims=True)
path = Path(__file__).resolve().parents[2] / 'assets/materials/thatch-normal.png'
Image.fromarray(np.uint8((normal*.5+.5)*255)).save(path)
print('Baked original 1024px reed normal')
