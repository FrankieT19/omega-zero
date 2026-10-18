/* Build-time ARM64 smoke test: load a tiny original GB cartridge and cycle native palettes. */
#include <libretro.h>
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static const char *palette="GB - Pocket";
static bool dirty=true, declared=false;
static unsigned frames=0;
static uint32_t pixel=0;
static enum retro_pixel_format format=RETRO_PIXEL_FORMAT_0RGB1555;
static bool environment(unsigned command,void *data) {
    switch(command) {
    case RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION: *(unsigned*)data=0; return true;
    case RETRO_ENVIRONMENT_SET_VARIABLES: {
        const struct retro_variable *v=data;
        for (;v->key;v++) if (!strcmp(v->key,"gambatte_gb_internal_palette")) declared=true;
        return true;
    }
    case RETRO_ENVIRONMENT_GET_VARIABLE: {
        struct retro_variable *v=data;
        if (!strcmp(v->key,"gambatte_gb_internal_palette")) v->value=palette;
        else if (!strcmp(v->key,"gambatte_gb_colorization")) v->value="internal";
        else if (!strcmp(v->key,"gambatte_gbc_color_correction")) v->value="disabled";
        else if (!strcmp(v->key,"gambatte_mix_frames")) v->value="disabled";
        else { v->value=NULL; return false; }
        return true;
    }
    case RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: *(bool*)data=dirty; dirty=false; return true;
    case RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: format=*(enum retro_pixel_format*)data; return true;
    case RETRO_ENVIRONMENT_GET_CAN_DUPE: *(bool*)data=true; return true;
    case RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY:
    case RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: *(const char**)data="."; return true;
    default: return false;
    }
}
static void video(const void *data,unsigned w,unsigned h,size_t pitch) {
    if (!data) return;
    if (w!=160 || h!=144) { fprintf(stderr,"Gambatte frame %ux%u\n",w,h); exit(1); }
    const uint8_t *at=(const uint8_t*)data+72*pitch+80*(format==RETRO_PIXEL_FORMAT_XRGB8888?4:2);
    pixel=0; memcpy(&pixel,at,format==RETRO_PIXEL_FORMAT_XRGB8888?4:2); frames++;
}
static void audio(int16_t l,int16_t r) { (void)l;(void)r; }
static size_t batch(const int16_t *data,size_t count) { (void)data;return count; }
static void poll(void) {}
static int16_t input(unsigned p,unsigned d,unsigned i,unsigned id) { (void)p;(void)d;(void)i;(void)id;return 0; }
#define LOAD(name) __typeof__(&name) fn_##name=(__typeof__(&name))dlsym(lib,#name); if (!fn_##name) { fprintf(stderr,"missing %s\n",#name);return 1; }
int main(int argc,char **argv) {
    if(argc!=2) return 2;
    void *lib=dlopen(argv[1],RTLD_NOW|RTLD_LOCAL); if(!lib) { fprintf(stderr,"%s\n",dlerror());return 1; }
    LOAD(retro_set_environment);LOAD(retro_set_video_refresh);LOAD(retro_set_audio_sample);LOAD(retro_set_audio_sample_batch);
    LOAD(retro_set_input_poll);LOAD(retro_set_input_state);LOAD(retro_init);LOAD(retro_load_game);LOAD(retro_run);LOAD(retro_unload_game);LOAD(retro_deinit);
    fn_retro_set_environment(environment);fn_retro_set_video_refresh(video);fn_retro_set_audio_sample(audio);fn_retro_set_audio_sample_batch(batch);
    fn_retro_set_input_poll(poll);fn_retro_set_input_state(input);fn_retro_init();
    uint8_t rom[32768]={0}; rom[0x100]=0xc3;rom[0x101]=0x50;rom[0x102]=1;
    memcpy(rom+0x134,"PALETTE TEST",12);
    const uint8_t program[]={0xf3,0x3e,0x55,0xe0,0x47,0x3e,0x91,0xe0,0x40,0x18,0xfe}; memcpy(rom+0x150,program,sizeof(program));
    uint8_t check=0;for(unsigned i=0x134;i<=0x14c;i++)check=check-rom[i]-1;rom[0x14d]=check;
    struct retro_game_info game={"palette-test.gb",rom,sizeof(rom),NULL};
    if(!declared || !fn_retro_load_game(&game)) { fprintf(stderr,"Gambatte initialization/load failed\n");return 1; }
    for(unsigned i=0;i<180;i++)fn_retro_run();
    uint32_t pocket=pixel;
    const char *choices[]={"GB - DMG","GB - Light","GBC - Blue","GBC - Brown","GBC - Dark Blue","GBC - Dark Brown","GBC - Dark Green","GBC - Grayscale","GBC - Green","GBC - Inverted","GBC - Orange","GBC - Pastel Mix","GBC - Red","GBC - Yellow"};
    bool changed=false;
    for(unsigned i=0;i<sizeof(choices)/sizeof(*choices);i++) { palette=choices[i];dirty=true;fn_retro_run();fn_retro_run();changed|=pixel!=pocket; }
    fn_retro_unload_game();fn_retro_deinit();dlclose(lib);
    if(!frames || !changed) { fprintf(stderr,"No frames or palette switching had no effect\n");return 1; }
    printf("Gambatte native GB palette smoke test passed (%u frames)\n",frames);return 0;
}
