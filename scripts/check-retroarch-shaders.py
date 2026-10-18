#!/usr/bin/env python3
"""Check the real RetroArch Slang stages (not the frontend's GLSL adapters)."""
from pathlib import Path
import hashlib, subprocess, sys, tempfile
root=Path(__file__).resolve().parents[1]
assets=root/'card/System/retroarch-shaders'
for line in (assets/'upstream.sha256').read_text().splitlines():
    digest, name=line.split('  ',1)
    assert hashlib.sha256((assets/name).read_bytes().replace(b'\r\n',b'\n')).hexdigest()==digest,name
count=0
for source in sorted(assets.rglob('*.slang')):
    common, body=source.read_text(encoding='utf-8').split('#pragma stage vertex',1)
    vertex, fragment=body.split('#pragma stage fragment',1)
    with tempfile.TemporaryDirectory(prefix='omega-slang-') as temp:
        temp=Path(temp)
        for stage, code in [('vert',vertex),('frag',fragment)]:
            src=temp/f'stage.{stage}'; out=temp/f'{stage}.spv'
            src.write_text(common+code,encoding='utf-8')
            p=subprocess.run([sys.argv[1],'-V','-Os','-S',stage,str(src),'-o',str(out)],capture_output=True,text=True)
            if p.returncode:
                raise SystemExit(f'{source.relative_to(root)} {stage}:\n{p.stdout}\n{p.stderr}')
            count+=1
print(f'Validated {count} real Slang stages and pinned upstream source hashes')
