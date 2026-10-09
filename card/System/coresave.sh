#!/bin/sh
SD="${AGS_CARD:-/mnt/sdcard}"
MOUNTPOINT="${AGS_MOUNTPOINT:-mountpoint}"
DIR="$SD/crash"

if ! $MOUNTPOINT -q "$SD" 2>/dev/null; then
	cat > /dev/null
	exit 0
fi

exe="${AGS_CORE_EXE:-$(readlink "/proc/$1/exe" 2>/dev/null)}"
mkdir -p "$DIR" 2>/dev/null
echo "$(date '+%F %T') pid=$1 signal=$2 thread=$3 exe=$exe" >> "$DIR/cores.log"

case "$exe" in
*/omega-zero)
	cp "/proc/$1/maps" "$DIR/omega-zero.maps" 2>/dev/null
	cat > "$DIR/omega-zero.core.tmp" && mv -f "$DIR/omega-zero.core.tmp" "$DIR/omega-zero.core"
	;;
*)
	cat > /dev/null
	;;
esac
sync
