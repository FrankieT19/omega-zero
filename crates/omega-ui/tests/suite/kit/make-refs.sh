#!/bin/bash
# Screenshots kit.html's components into ref/, one PNG each at one pixel to an art pixel, for
# kit.rs to hold the Rust faces to. Run it again after changing a component in assets/ui; the
# faces must then change to match, pixel for pixel.
#
#   crates/omega-ui/tests/suite/kit/make-refs.sh
#
# macOS smooths DOM text (CoreText font smoothing), which rings each glyph with a faint halo the
# canvas words never get; kit.html turns it off, so the hints and titles are drawn on whole pixels
# as the faces are.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
chrome="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
mkdir -p "$here/ref"

# name, width and height in art pixels, then the query kit.html draws it from
shoot() {
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
    --default-background-color=00000000 --virtual-time-budget=3000 \
    --window-size="$2,$3" --screenshot="$here/ref/$1.png" "file://$here/kit.html?$4" 2>/dev/null
}

shoot modal 200 120 'c=modal&w=200&h=120'
shoot modal-panels 200 121 'c=modal&w=200&h=121&weights=1,1'
shoot modal-panels-4-3 200 120 'c=modal&w=200&h=120&weights=4,3'
shoot modal-panels-1-1 200 120 'c=modal&w=200&h=120&weights=1,1'
shoot panel-button-setting-active 160 60 'c=panel&w=160&h=60&label=BUTTON%20SETTING&active'
shoot panel-button-setting 160 60 'c=panel&w=160&h=60&label=BUTTON%20SETTING'
shoot panel-configs-active 160 60 'c=panel&w=160&h=60&label=CONFIGS&active'
shoot panel-configs 160 60 'c=panel&w=160&h=60&label=CONFIGS'
shoot action-left-quickmenu 131 27 'c=action-left&label=QUICKMENU&hint=SELECT'
shoot action-right-settings 124 27 'c=action-right&label=SETTINGS&hint=START'
shoot action-left-return 110 27 'c=action-left&label=RETURN&hint=B'
shoot action-right-confirm 117 27 'c=action-right&label=CONFIRM&hint=A'
shoot action-left-empty 107 27 'c=action-left'
shoot action-right-empty 107 27 'c=action-right'
