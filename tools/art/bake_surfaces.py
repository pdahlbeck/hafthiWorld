"""Original deterministic tileable PBR studies. Requires numpy + Pillow, not at runtime."""
from pathlib import Path
import numpy as np
from PIL import Image
OUT = Path(__file__).resolve().parents[2] / 'assets/materials'
OUT.mkdir(parents=True, exist_ok=True)
N = 512
rng = np.random.default_rng(901)
y, x = np.mgrid[:N, :N] / N

def noise(scale):
    # Periodic Fourier noise, so all maps tile without a border seam.
    a = np.zeros((N,N))
    for _ in range(16):
        k,l = rng.integers(-scale,scale+1,2)
        if k or l:
            a += np.sin(2*np.pi*(k*x+l*y)+rng.random()*6.28)
    return a / max(np.std(a),1e-6)

broad, mid, fine = noise(3), noise(18), noise(100)
for kind in ('ground','limestone','wool','wood'):
    if kind == 'ground':
        h = mid*.025 + fine*.004
        tone = .60 + broad*.025 + mid*.018 + fine*.006
        rgb = np.stack([tone*.96,tone,tone*.91],-1)
        rough = .92 + fine*.025
    elif kind == 'limestone':
        bands = np.sin(2*np.pi*(y*17 + np.sin(x*6.28)*.09))
        pits = np.maximum(fine-1.15,0)
        h = mid*.025 + bands*.018-pits*.07
        tone = .69 + broad*.045 + mid*.018 - pits*.07 + bands*.018
        rgb = np.stack([tone*1.03,tone*1.02,tone*.94],-1)
        rough = .91 + mid*.035
    elif kind == 'wool':
        weave = np.sin(x*2*np.pi*170)*np.cos(y*2*np.pi*170)
        h = weave*.008 + fine*.005
        tone = .54 + broad*.025 + weave*.025 + fine*.025
        rgb = np.stack([tone*.81,tone,tone*.87],-1)
        rough = .98 + fine*.01
    else:
        grain = np.sin(2*np.pi*(x*34+.15*np.sin(y*2*np.pi*3)+.03*mid))
        h = grain*.006+fine*.002
        tone = .42 + grain*.025+broad*.02+fine*.005
        rgb = np.stack([tone,tone*.79,tone*.57],-1)
        rough = .89 + grain*.03
    dx = (np.roll(h,-1,1)-np.roll(h,1,1))*1.0
    dy = (np.roll(h,-1,0)-np.roll(h,1,0))*1.0
    normal = np.stack([-dx,-dy,np.ones_like(h)],-1)
    normal /= np.linalg.norm(normal,axis=-1,keepdims=True)
    orm = np.stack([np.ones_like(h),np.clip(rough,0,1),np.zeros_like(h)],-1)
    for suffix,arr in [('color',rgb),('normal',normal*.5+.5),('orm',orm)]:
        Image.fromarray(np.uint8(np.clip(arr,0,1)*255)).save(OUT/f'{kind}-{suffix}.png')
print('Baked 12 original 512px surface maps')
