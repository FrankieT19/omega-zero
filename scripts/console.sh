#!/bin/sh
# Talks to a console on the home network over SSH, for task deploy:wifi and task log:wifi.
#
#   console.sh deploy BINARY [FILE...]   copy BINARY to System/omega-zero, and every FILE into
#                                        System/, then reboot into it and wait for it to start
#   console.sh log                       print omega-zero's log
#
# CONSOLE (default rg35xxsp.local, HOST:PORT for another port), SD (default /mnt/sdcard) and
# CONSOLE_PASSWORD (default root, BaseOS's own unless baseos.conf sets ssh_password) say where.
set -eu

console="${CONSOLE:-rg35xxsp.local}"
port=22
case "$console" in
*:*)
	port="${console##*:}"
	console="${console%:*}"
	;;
esac
host="root@$console"
sd="${SD:-/mnt/sdcard}"
password="${CONSOLE_PASSWORD:-root}"

command -v sshpass > /dev/null || {
	echo "console.sh: needs sshpass (brew install sshpass)" >&2
	exit 1
}

ssh_once() {
	sshpass -p "$password" ssh -p "$port" -o ConnectTimeout=5 -o StrictHostKeyChecking=no \
		-o UserKnownHostsFile=/dev/null -o LogLevel=ERROR \
		-o PreferredAuthentications=password -o PubkeyAuthentication=no "$host" "$@"
}

# on [-i FILE] COMMAND runs COMMAND on the console, with FILE as its input. Dropbear now and
# then turns the first login away, so a failed connection (ssh's 255, sshpass's 5) is tried
# again, reading FILE from the start; the command's own failure is not.
on() {
	input=/dev/null
	if [ "$1" = "-i" ]; then
		input="$2"
		shift 2
	fi
	tries=0
	while :; do
		status=0
		ssh_once "$@" < "$input" || status=$?
		case "$status" in
		255 | 5) ;;
		*) return "$status" ;;
		esac
		tries=$((tries + 1))
		if [ "$tries" -ge 3 ]; then
			echo "console.sh: could not reach $host" >&2
			return "$status"
		fi
		sleep 1
	done
}

# Copies a file to the console under a temporary name, checks it arrived whole and renames it
# into place, so a shell running the old copy of a script keeps reading that one.
put() {
	on -i "$1" "cat > '$2.new'"
	want="$(shasum -a 256 "$1" | cut -d' ' -f1)"
	got="$(on "sha256sum '$2.new'" | cut -d' ' -f1)"
	if [ "$want" != "$got" ]; then
		echo "console.sh: $2 arrived damaged ($got, sent $want)" >&2
		exit 1
	fi
	on "mv -f '$2.new' '$2'"
}

lines() {
	ssh_once "cat '$1' 2> /dev/null | wc -l" < /dev/null 2> /dev/null | tr -d ' ' || true
}

case "${1:-}" in
deploy)
	binary="$2"
	shift 2
	status=0
	on "test -d '$sd/System'" || status=$?
	case "$status" in
	0) ;;
	1)
		echo "console.sh: no $sd/System on $host, set SD" >&2
		exit 1
		;;
	*) exit 1 ;;
	esac
	log="$sd/omega-zero-session.log"
	before="$(lines "$log")"
	put "$binary" "$sd/System/omega-zero"
	on "chmod +x '$sd/System/omega-zero'"
	for f in "$@"; do
		put "$f" "$sd/System/$(basename "$f")"
	done
	on sync
	echo "copied, rebooting $host"
	on "(sleep 1; reboot) > /dev/null 2>&1 &"
	# The launcher writes a line to the session log as it starts omega-zero.
	waited=0
	while [ "$waited" -lt 120 ]; do
		sleep 3
		waited=$((waited + 3))
		now="$(lines "$log")"
		if [ "${now:-0}" -gt "${before:-0}" ]; then
			ssh_once "tail -n 1 '$log'" < /dev/null
			exit 0
		fi
	done
	echo "console.sh: $host did not start omega-zero within 120 s" >&2
	exit 1
	;;
log)
	on "cat '$sd/omega-zero.log'"
	;;
*)
	echo "usage: console.sh deploy BINARY [FILE...] | console.sh log" >&2
	exit 2
	;;
esac
