"""Usage: python scripts/check-handheld-shaders.py [path/to/glslangValidator]"""
from pathlib import Path
import re
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1] / "crates/omega-gfx"
vertex = re.search(r'const VERT: &str = r#"(.*?)"#;', (root / "src/presets.rs").read_text(), re.S)[1]
with tempfile.TemporaryDirectory() as folder:
    folder = Path(folder)
    vert = folder / "preset.vert"
    frag = folder / "preset.frag"
    vert.write_text("#version 100\n" + vertex)
    for source in sorted((root / "shaders").glob("*.frag")):
        selected=vertex
        vert.write_text("#version 100\n"+selected)
        print(source.name, flush=True)
        frag.write_text("#version 100\n#define FRAG_COLOR gl_FragColor\n" + (root / "shaders/palette.glsl").read_text() + source.read_text())
        subprocess.run([sys.argv[1] if len(sys.argv) > 1 else "glslangValidator", "-l", str(vert), str(frag)], check=True)
