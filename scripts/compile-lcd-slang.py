"""Compile the unmodified Libretro LCD Slang sources to the renderer's GLES2 dialect.
Usage: python scripts/compile-lcd-slang.py GLSLANG SPIRV_CROSS
Generated files are checked in; the handheld needs neither compiler.
"""
from pathlib import Path
import hashlib,re,subprocess,sys,tempfile
root=Path(__file__).resolve().parents[1]
for name in ['lcd1x','lcd3x']:
 src=root/'crates/omega-gfx/shaders'/f'{name}.upstream.slang'
 source=src.read_text(encoding='utf-8')
 fragment=source.split('#pragma stage vertex')[0]+source.split('#pragma stage fragment')[1]
 with tempfile.TemporaryDirectory() as temp:
  temp=Path(temp);frag=temp/'shader.frag';spv=temp/'shader.spv';out=temp/'generated.frag'
  frag.write_text(fragment,encoding='utf-8')
  subprocess.run([sys.argv[1],'-V','-Os','-S','frag',str(frag),'-o',str(spv)],check=True)
  subprocess.run([sys.argv[2],str(spv),'--es','--version','100','--output',str(out)],check=True)
  # Bindings only: the renderer supplies its GL/ES version and fragment output declaration.
  text=out.read_text().replace('#version 100\n','').replace('gl_FragData[0]','FRAG_COLOR')
  header=f'// Generated from {src.name}; source SHA256: {hashlib.sha256(src.read_bytes().replace(bytes([13,10]),bytes([10]))).hexdigest()}\n// glslang -V -Os -> SPIRV-Cross --es --version 100. Do not edit shader arithmetic.\n'
  for base in ['crates/omega-gfx/shaders','licenses/handheld-shaders']:
   (root/base/f'{name}.generated.glsl').write_text(header+text,encoding='utf-8',newline='\n')
 print(name,'compiled from Slang')
