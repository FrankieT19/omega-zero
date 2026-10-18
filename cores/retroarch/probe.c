/* GPL-3.0-or-later. A real libretro core for the RetroArch companion smoke test. */
#include <libretro.h>
#include <stdint.h>
#include <string.h>
static retro_video_refresh_t video;
static retro_input_poll_t poll;
static retro_input_state_t input;
static retro_environment_t env;
static struct retro_rumble_interface rumble;
static uint8_t ram[32];
static uint32_t pixels[240*160];
static uint32_t frames;
void retro_set_environment(retro_environment_t cb) { env=cb; }
void retro_set_video_refresh(retro_video_refresh_t cb) { video=cb; }
void retro_set_audio_sample(retro_audio_sample_t cb) { (void)cb; }
void retro_set_audio_sample_batch(retro_audio_sample_batch_t cb) { (void)cb; }
void retro_set_input_poll(retro_input_poll_t cb) { poll=cb; }
void retro_set_input_state(retro_input_state_t cb) { input=cb; }
unsigned retro_api_version(void) { return RETRO_API_VERSION; }
void retro_init(void) { enum retro_pixel_format fmt=RETRO_PIXEL_FORMAT_XRGB8888;env(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,&fmt);env(RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE,&rumble); }
void retro_deinit(void) { }
void retro_get_system_info(struct retro_system_info *i) { memset(i,0,sizeof(*i));i->library_name="Omega bridge probe";i->library_version="1";i->valid_extensions="bin"; }
void retro_get_system_av_info(struct retro_system_av_info *i) { memset(i,0,sizeof(*i));i->geometry.base_width=i->geometry.max_width=240;i->geometry.base_height=i->geometry.max_height=160;i->geometry.aspect_ratio=1.5;i->timing.fps=60;i->timing.sample_rate=32768; }
bool retro_load_game(const struct retro_game_info *i) { (void)i;return true; }
bool retro_load_game_special(unsigned t,const struct retro_game_info *i,size_t n) { (void)t;(void)i;(void)n;return false; }
void retro_unload_game(void) { }
void retro_reset(void) { frames=0; }
void retro_set_controller_port_device(unsigned p,unsigned d) { (void)p;(void)d; }
void retro_run(void) {
   poll();frames++;
   if (input(0,RETRO_DEVICE_JOYPAD,0,RETRO_DEVICE_ID_JOYPAD_A)) ram[0]=1;
   if (input(0,RETRO_DEVICE_JOYPAD,0,RETRO_DEVICE_ID_JOYPAD_UP)) ram[1]=1;
   if (rumble.set_rumble_state && frames==60) rumble.set_rumble_state(0,RETRO_RUMBLE_STRONG,12345);
   video(pixels,240,160,240*4);
}
size_t retro_serialize_size(void) { return sizeof(frames)+sizeof(ram); }
bool retro_serialize(void *data,size_t n) { if(n<retro_serialize_size()) return false;memcpy(data,&frames,sizeof(frames));memcpy((char*)data+sizeof(frames),ram,sizeof(ram));return true; }
bool retro_unserialize(const void *data,size_t n) { if(n<retro_serialize_size()) return false;memcpy(&frames,data,sizeof(frames));memcpy(ram,(const char*)data+sizeof(frames),sizeof(ram));return true; }
void *retro_get_memory_data(unsigned id) { return id==RETRO_MEMORY_SAVE_RAM?ram:0; }
size_t retro_get_memory_size(unsigned id) { return id==RETRO_MEMORY_SAVE_RAM?sizeof(ram):0; }
unsigned retro_get_region(void) { return RETRO_REGION_NTSC; }
void retro_cheat_reset(void) { }
void retro_cheat_set(unsigned i,bool e,const char*c) { (void)i;(void)e;(void)c; }
