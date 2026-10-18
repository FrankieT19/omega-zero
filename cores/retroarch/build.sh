#!/bin/sh
# Reproducible RetroArch with the upstream Slang pipeline and an omega-zero input bridge.
set -eu
commit=ea79555bc8fcd4cb1a2fb8a48c1a6cdba819fb07
out=vendor/retroarch
stamp="commit=$commit bridge=1 gles=3 slang=upstream"
if [ -x "$out" ] && [ "$(cat "$out.meta" 2>/dev/null)" = "$stamp" ]; then exit 0; fi
apt-get install -y --no-install-recommends python3 pkg-config libegl1-mesa-dev libgles2-mesa-dev libasound2-dev zlib1g-dev
mkdir -p vendor target-device/retroarch
src=vendor/retroarch-src.tar.gz
[ -f "$src" ] || curl -fL --retry 3 -o "$src" "https://github.com/libretro/RetroArch/archive/$commit.tar.gz"
work=target-device/retroarch/source
mkdir -p "$work"
tar -xzf "$src" --strip-components=1 -C "$work"
python3 cores/retroarch/patch.py "$work"
cd "$work"
./configure --prefix=/usr --enable-opengles3 --enable-mali_fbdev \
  --enable-slang --enable-builtinssl --enable-builtinzlib \
  --enable-builtinglslang --enable-builtinspirv_cross --enable-command --enable-stdin_cmd \
  --enable-alsa --enable-rgui --disable-x11 --disable-wayland --disable-kms \
  --disable-sdl --disable-sdl2 --disable-vulkan --disable-opengl1 \
  --disable-udev --disable-pulse --disable-jack --disable-oss --disable-libudev \
  --disable-ffmpeg --disable-qt --disable-discord --disable-mpv --disable-libshake
make -j2
cd ../../..
cp "$work/retroarch" "$out"
strip "$out"
chmod +x "$out"
printf '%s\n' "$stamp" > "$out.meta"
"$out" --features > "$out.features"
grep -E 'Slang.*yes|slang.*yes' "$out.features"
file "$out" | grep -q aarch64
