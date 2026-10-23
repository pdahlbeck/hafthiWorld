"""Fetch the pinned MIT authoring inputs. Never used by the game or cargo builds.
python3 tools/art/fetch_rocketbox.py /path/to/cache
Then set HAFTHI_ROCKETBOX_SOURCE to that directory for the Blender builder.
"""
import hashlib,json,sys,urllib.request
from pathlib import Path
manifest=json.loads(Path(__file__).with_name('rocketbox-sources.json').read_text())
cache=Path(sys.argv[1])
for item in manifest['files']:
    target=cache/item['local']
    if target.exists() and hashlib.sha256(target.read_bytes()).hexdigest()==item['sha256']:
        continue
    url=f"https://raw.githubusercontent.com/{manifest['repository']}/{manifest['commit']}/{item['upstream']}"
    with urllib.request.urlopen(url) as response:data=response.read()
    assert hashlib.sha256(data).hexdigest()==item['sha256'], f"Source checksum mismatch: {item['upstream']}"
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
    print(item['local'])
