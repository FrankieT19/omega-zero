#!/usr/bin/env python3
"""Add a private stdin controller bridge, without altering RetroArch's shader pipeline."""
from pathlib import Path
import sys
root = Path(sys.argv[1])
def replace(file, old, new):
    p = root / file
    s = p.read_text()
    if s.count(old) != 1:
        raise SystemExit(f'{file}: expected one patch anchor: {old[:80]}')
    p.write_text(s.replace(old, new))
replace('command.c', 'static void command_parse_sub_msg(command_t *handle, const char *tok)', '''unsigned omega_pad = 0;
unsigned omega_flags = 0;
static void command_parse_sub_msg(command_t *handle, const char *tok)''')
replace('command.c', '   handle->error   = false;', '''   handle->error   = false;
   if (sscanf(tok, "OMEGA_INPUT %x %u", &omega_pad, &omega_flags) == 2)
      return;''')
# Hold flags remain active between stdin polls. Standard commands remain unchanged.
replace('input/input_driver.c', 'static void *input_null_init', '''extern unsigned omega_pad;
extern unsigned omega_flags;
static void *input_null_init''')
replace('input/input_driver.c', 'static input_driver_t input_null = {', '''static int16_t omega_input_state(void *data,
      const input_device_driver_t *joypad, rarch_joypad_info_t *joypad_info,
      const retro_keybind_set *binds,
      unsigned port, unsigned device, unsigned index, unsigned id)
{
   if (port || device != RETRO_DEVICE_JOYPAD) return 0;
   if (id == RETRO_DEVICE_ID_JOYPAD_MASK) return (int16_t)omega_pad;
   return id < 16 ? ((omega_pad >> id) & 1) : 0;
}
static uint64_t omega_capabilities(void *data) { return 1ULL << RETRO_DEVICE_JOYPAD; }
static input_driver_t input_omega = {
   input_null_init, input_null_poll, omega_input_state, input_null_free,
   input_null_set_sensor_state, input_null_get_sensor_input, omega_capabilities,
   "omega", input_null_grab_mouse, input_null_grab_stdin, input_null_keypress_vibrate
};
static input_driver_t input_null = {''')
replace('input/input_driver.c', 'input_driver_t *input_drivers[] = {', 'input_driver_t *input_drivers[] = {\n   &input_omega,')
# input_driver_keys_pressed runs after command polling; only the bridge owns these holds.
replace('input/input_driver.c', '         && input_st->command[j]->state[i])', '''         && (input_st->command[j]->state[i]
            || (i == RARCH_FAST_FORWARD_HOLD_KEY && (omega_flags & 1))
            || (i == RARCH_REWIND && (omega_flags & 2))))''')
replace('input/input_driver.c', '            if (input_st->command[j]->state[i])', '''            if (input_st->command[j]->state[i]
                || (i == RARCH_FAST_FORWARD_HOLD_KEY && (omega_flags & 1))
                || (i == RARCH_REWIND && (omega_flags & 2)))''')
# The context advertised GLSL alone even when it created an ES3 context for glcore.
replace('gfx/drivers_context/mali_fbdev_ctx.c', '   BIT32_SET(flags, GFX_CTX_FLAGS_SHADERS_GLSL);', '''   BIT32_SET(flags, GFX_CTX_FLAGS_SHADERS_GLSL);
   if (mali_flags & GFX_CTX_MALI_FBDEV_FLAG_GLES3)
      BIT32_SET(flags, GFX_CTX_FLAGS_SHADERS_SLANG);''')
print('Applied omega-zero private stdin/input bridge; Slang renderer unchanged')
