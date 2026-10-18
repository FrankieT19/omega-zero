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
static retro_atomic_int_t omega_dim_bits = 1065353216;
static retro_atomic_int_t omega_blue_bits = 0;
float omega_display_get(unsigned which) {
   int bits=retro_atomic_load_relaxed_int(which?&omega_blue_bits:&omega_dim_bits);
   float value;memcpy(&value,&bits,sizeof(value));return value;
}
void omega_display_set(unsigned which,float value) {
   int bits;value=value<0?0:value>1?1:value;memcpy(&bits,&value,sizeof(bits));
   retro_atomic_store_relaxed_int(which?&omega_blue_bits:&omega_dim_bits,bits);
}
static void command_parse_sub_msg(command_t *handle, const char *tok)''')
replace('command.c', '   handle->error   = false;', '''   handle->error   = false;
   if (sscanf(tok, "OMEGA_INPUT %x %u", &omega_pad, &omega_flags) == 2)
      return;
   if (strncmp(tok,"OMEGA_MSG ",10)==0)
   {
      runloop_msg_queue_push(tok+10, strlen(tok+10), 1, 90, true, NULL,
            MESSAGE_QUEUE_ICON_DEFAULT, MESSAGE_QUEUE_CATEGORY_INFO);
      return;
   }
   if (strncmp(tok,"OMEGA_SCALE ",12)==0)
   {
      unsigned mode, aspect;
      if (sscanf(tok+12,"%u %u",&mode,&aspect)==2 && mode<3 && aspect<=ASPECT_RATIO_FULL)
      {
         settings_t *settings=config_get_ptr();
         configuration_set_bool(settings,settings->bools.video_scale_integer,mode==0);
         configuration_set_uint(settings,settings->uints.video_aspect_ratio_idx,aspect);
         command_event(CMD_EVENT_VIDEO_SET_ASPECT_RATIO,NULL);
         command_event(CMD_EVENT_VIDEO_APPLY_STATE_CHANGES,NULL);
      }
      return;
   }
   if (strcmp(tok,"OMEGA_SHADER")==0)
   {
      video_shader_ctx_t current;
      if (video_shader_driver_get_current_shader(&current) && current.data)
         printf("OMEGA_SHADER %u %s\\n",current.data->passes,current.data->path);
      else printf("OMEGA_SHADER 0\\n");
      fflush(stdout);return;
   }
   if (strcmp(tok, "OMEGA_SCREEN") == 0)
   {
      video_shader_ctx_t current; video_viewport_t vp;
      if (video_shader_driver_get_current_shader(&current) && current.data
            && video_driver_get_viewport_info(&vp))
      {
         unsigned n;
         for (n=0;n<current.data->num_parameters;n++)
         {
            struct video_shader_parameter *p = &current.data->parameters[n];
            if (strcmp(p->id,"OMEGA_ORIGIN_X")==0) video_shader_param_set(p,(float)VIDEO_POS_X(vp.pos));
            if (strcmp(p->id,"OMEGA_ORIGIN_Y")==0) video_shader_param_set(p,(float)(VIDEO_SCALE_H(vp.full_dims)-VIDEO_POS_Y(vp.pos)-VIDEO_SCALE_H(vp.dims)));
         }
      }
      return;
   }
   if (strncmp(tok, "OMEGA_PARAM ", 12) == 0)
   {
      char name[64]; float value; video_shader_ctx_t current;
      if (sscanf(tok + 12, "%63s %f", name, &value) != 2) return;
      if (strcmp(name,"OMEGA_DIM")==0) omega_display_set(0,value);
      if (strcmp(name,"OMEGA_BLUE")==0) omega_display_set(1,value);
      if (video_shader_driver_get_current_shader(&current) && current.data)
      {
         unsigned n;
         for (n = 0; n < current.data->num_parameters; n++)
         {
            struct video_shader_parameter *p = &current.data->parameters[n];
            if (strcmp(p->id, name) == 0)
               video_shader_param_set(p, value < p->minimum ? p->minimum :
                  value > p->maximum ? p->maximum : value);
         }
      }
      return;
   }
   if (strncmp(tok, "OMEGA_VOLUME ", 13) == 0)
   {
      float db = (float)atof(tok + 13);
      command_event_set_volume(config_get_ptr(), db, false, false);
      return;
   }''')
# Hold flags remain active between stdin polls. Standard commands remain unchanged.
replace('input/input_driver.c', 'static void *input_null_init', '''#include <sys/prctl.h>
#include <signal.h>
#include <unistd.h>
extern unsigned omega_pad;
extern unsigned omega_flags;
extern void omega_display_set(unsigned which,float value);
static void *input_null_init''')
replace('input/input_driver.c', 'static input_driver_t input_null = {', '''static void *omega_input_init(const char *joypad_driver) {
   const char *dim=getenv("OMEGA_DISPLAY_DIM");
   const char *blue=getenv("OMEGA_DISPLAY_BLUE");
   if (dim) omega_display_set(0,(float)atof(dim));
   if (blue) omega_display_set(1,(float)atof(blue));
   prctl(PR_SET_PDEATHSIG, SIGTERM);
   return getppid() == 1 ? NULL : (void*)-1;
}
static int16_t omega_input_state(void *data,
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
   omega_input_init, input_null_poll, omega_input_state, input_null_free,
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
replace('input/input_driver.c', 'static input_device_driver_t null_joypad = {', r"""
static void *omega_joy_init(void *data) { return (void*)-1; }
static bool omega_joy_query(unsigned port) { return port == 0; }
static void omega_joy_destroy(void) { }
static int32_t omega_joy_button(unsigned port, uint16_t id) { return port == 0 && id < 16 ? ((omega_pad >> id) & 1) : 0; }
static int16_t omega_joy_state(rarch_joypad_info_t *info, const uint16_t *keys, const uint32_t *axes, unsigned port) {
   unsigned n; uint16_t mask = 0;
   for (n = 0; n < 16; n++) if (omega_joy_button(port, keys[n])) mask |= 1u << n;
   return (int16_t)mask;
}
static void omega_joy_buttons(unsigned port, input_bits_t *state) {
   unsigned n; BIT256_CLEAR_ALL_PTR(state);
   for (n=0;n<16;n++) if (omega_joy_button(port,n)) BIT256_SET_PTR(state,n);
}
static int16_t omega_joy_axis(unsigned port, uint32_t axis) { return 0; }
static void omega_joy_poll(void) { }
static bool omega_joy_rumble(unsigned port, enum retro_rumble_effect effect, uint16_t strength) {
   static uint16_t motors[2] = {0,0};
   if (port || effect > RETRO_RUMBLE_WEAK) return false;
   motors[effect] = strength;
   printf("OMEGA_RUMBLE %u\n", motors[0] > motors[1] ? motors[0] : motors[1]); fflush(stdout);
   return true;
}
static const char *omega_joy_name(unsigned port) { return "omega-zero"; }
static input_device_driver_t omega_joypad = {
   omega_joy_init, omega_joy_query, omega_joy_destroy, omega_joy_button,
   omega_joy_state, omega_joy_buttons, omega_joy_axis, omega_joy_poll,
   omega_joy_rumble, NULL, NULL, NULL, omega_joy_name, "omega"
};
static input_device_driver_t null_joypad = {""")
replace('input/input_driver.c', 'input_device_driver_t *joypad_drivers[] = {', 'input_device_driver_t *joypad_drivers[] = {\n   &omega_joypad,')
# The context advertised GLSL alone even when it created an ES3 context for glcore.
replace('gfx/drivers_context/mali_fbdev_ctx.c', '   BIT32_SET(flags, GFX_CTX_FLAGS_SHADERS_GLSL);', '''   BIT32_SET(flags, GFX_CTX_FLAGS_SHADERS_GLSL);
   if (mali_flags & GFX_CTX_MALI_FBDEV_FLAG_GLES3)
      BIT32_SET(flags, GFX_CTX_FLAGS_SHADERS_SLANG);''')
print('Applied omega-zero private stdin/input bridge; Slang pipeline unchanged')

# The display multiply runs after game, menu and OSD rendering, not inside a preset.
replace('gfx/drivers/gl3.c', '   GLuint menu_texture;', '   GLuint menu_texture;\n   GLuint omega_shade_texture;')
replace('gfx/drivers/gl3.c', '   if (gl->menu_texture != 0)\n      glDeleteTextures(1, &gl->menu_texture);', '   if (gl->omega_shade_texture) glDeleteTextures(1,&gl->omega_shade_texture);\n   if (gl->menu_texture != 0)\n      glDeleteTextures(1, &gl->menu_texture);')
replace('gfx/drivers/gl3.c', '#if defined(HAVE_MENU)\nstatic void gl3_draw_menu_texture', Path(__file__).with_name('dimmer.h').read_text()+'\n#if defined(HAVE_MENU)\nstatic void gl3_draw_menu_texture')
replace('gfx/drivers/gl3.c', '   if (gl->ctx_driver->update_window_title)\n      gl->ctx_driver->update_window_title(gl->ctx_data);', '   gl3_draw_omega_shade(gl,width,height);\n   if (gl->ctx_driver->update_window_title)\n      gl->ctx_driver->update_window_title(gl->ctx_data);')
