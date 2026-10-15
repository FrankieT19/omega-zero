#!/bin/sh
SD="${AGS_CARD:-/mnt/sdcard}"
MOUNTPOINT="${AGS_MOUNTPOINT:-mountpoint}"
DIR="$SD/crash"

if ! $MOUNTPOINT -q "$SD" 2>/dev/null; then
	cat > /dev/null
	exit 0
fi

exe="${AGS_CORE_EXE:-$(readlink "/proc/$1/exe" 2>/dev/null)}"
# launch_frontend.sh runs omega-zero through the dynamic loader, which is then the process's exe,
# omega-zero the first thing on its command line after it
prog="${AGS_CORE_PROG-$(tr '\0' '\n' < "/proc/$1/cmdline" 2>/dev/null | sed -n 2p)}"
mkdir -p "$DIR" 2>/dev/null
echo "$(date '+%F %T') pid=$1 signal=$2 thread=$3 exe=$exe${prog:+ prog=$prog}" >> "$DIR/cores.log"

case "$exe:$prog" in
*/omega-zero:* | */ld-linux*:*/omega-zero)
	# what it said before it went down, which the next start of it writes over
	cp "$SD/omega-zero.log" "$DIR/omega-zero.log" 2>/dev/null
	cp "/proc/$1/maps" "$DIR/omega-zero.maps" 2>/dev/null
	;;
esac

case "$exe" in
*/omega-zero)
	cat > "$DIR/omega-zero.core.tmp" && mv -f "$DIR/omega-zero.core.tmp" "$DIR/omega-zero.core"
	;;
*)
	cat > /dev/null
	;;
esac
sync
