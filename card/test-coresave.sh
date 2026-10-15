#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/coresave.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

setup() {
	rm -rf "$TMP/card" "$TMP/mountpoint"
	mkdir -p "$TMP/card"
	cat > "$TMP/mountpoint" <<-SH
		#!/bin/sh
		exit ${1:-0}
	SH
	chmod +x "$TMP/mountpoint"
}

run() {
	printf '%s' "$2" | AGS_CARD="$TMP/card" AGS_MOUNTPOINT="$TMP/mountpoint" \
		AGS_CORE_EXE="$1" AGS_CORE_PROG="${6:-}" sh "$SCRIPT" "${3:-4321}" "${4:-11}" "${5:-omega-emu}" 0
}

setup
run /usr/bin/busybox "not a frontend core"
grep -q 'signal=11 thread=omega-emu exe=/usr/bin/busybox' "$TMP/card/crash/cores.log" \
	|| { echo "no log line for a non-frontend crash: [$(cat "$TMP/card/crash/cores.log")]" >&2; exit 1; }
[ ! -f "$TMP/card/crash/omega-zero.core" ] \
	|| { echo "kept a core for a process that is not the frontend" >&2; exit 1; }

setup
run /mnt/sdcard/System/omega-zero "first core"
[ "$(cat "$TMP/card/crash/omega-zero.core")" = "first core" ] \
	|| { echo "the frontend's core was not saved" >&2; exit 1; }

run /mnt/sdcard/System/omega-zero "second core"
[ "$(cat "$TMP/card/crash/omega-zero.core")" = "second core" ] \
	|| { echo "a second crash did not replace the core" >&2; exit 1; }
[ ! -f "$TMP/card/crash/omega-zero.core.tmp" ] \
	|| { echo "left a half-written core behind" >&2; exit 1; }
[ "$(grep -c . "$TMP/card/crash/cores.log")" = "2" ] \
	|| { echo "cores.log did not record both crashes" >&2; exit 1; }

setup
echo "fatal runtime error: Rust cannot catch foreign exceptions, aborting" > "$TMP/card/omega-zero.log"
run /usr/lib/aarch64-linux-gnu/ld-linux-aarch64.so.1 "core through the loader" 4321 6 omega-emu \
	/mnt/sdcard/System/omega-zero
[ ! -f "$TMP/card/crash/omega-zero.core" ] \
	|| { echo "wrote a whole core of omega-zero run through the loader to the card" >&2; exit 1; }
grep -q 'cannot catch foreign exceptions' "$TMP/card/crash/omega-zero.log" \
	|| { echo "the log of the run that went down was not kept" >&2; exit 1; }
grep -q 'signal=6 thread=omega-emu exe=/usr/lib/aarch64-linux-gnu/ld-linux-aarch64.so.1 prog=/mnt/sdcard/System/omega-zero' \
	"$TMP/card/crash/cores.log" \
	|| { echo "cores.log does not name the program: [$(cat "$TMP/card/crash/cores.log")]" >&2; exit 1; }

setup
echo "omega-zero is fine" > "$TMP/card/omega-zero.log"
run /usr/lib/aarch64-linux-gnu/ld-linux-aarch64.so.1 "another program" 4321 11 worker /usr/bin/other
[ ! -f "$TMP/card/crash/omega-zero.log" ] \
	|| { echo "kept omega-zero's log for another program run through the loader" >&2; exit 1; }

setup
cat > "$TMP/mountpoint" <<-SH
	#!/bin/sh
	exit 1
SH
chmod +x "$TMP/mountpoint"
run /mnt/sdcard/System/omega-zero "core with nowhere to go" \
	|| { echo "a missing card made the helper fail" >&2; exit 1; }
[ ! -d "$TMP/card/crash" ] \
	|| { echo "wrote to a card that is not mounted" >&2; exit 1; }

echo "coresave: all passed"
