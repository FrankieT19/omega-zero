/* GPL-3.0-or-later. A final display colour multiply, outside the Slang pipeline.
 * This covers gameplay, arbitrary user presets, the RGUI and notifications alike. */
extern float omega_display_get(unsigned which);
static void gl3_draw_omega_shade(gl3_t *gl, unsigned width, unsigned height)
{
   unsigned n;
   float omega_dim=omega_display_get(0),omega_blue=omega_display_get(1);
   static const unsigned char white[4] = {255,255,255,255};
   float quad[32] = {
      0,0,0,0,1,1,1,1, 1,0,1,0,1,1,1,1,
      0,1,0,1,1,1,1,1, 1,1,1,1,1,1,1,1
   };
   if (omega_dim >= 0.99999f && omega_blue <= 0.00001f) return;
   for (n=0;n<4;n++) {
      quad[n*8+4] = omega_dim;
      quad[n*8+5] = omega_dim * (1.0f - 0.18f * omega_blue);
      quad[n*8+6] = omega_dim * (1.0f - 0.38f * omega_blue);
   }
   glBindFramebuffer(GL_FRAMEBUFFER,0);
   glViewport(0,0,width,height);
   glDisable(GL_SCISSOR_TEST);glDisable(GL_CULL_FACE);glDisable(GL_DEPTH_TEST);
   glActiveTexture(GL_TEXTURE1);
   if (!gl->omega_shade_texture) {
      glGenTextures(1,&gl->omega_shade_texture);
      glBindTexture(GL_TEXTURE_2D,gl->omega_shade_texture);
      glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_MIN_FILTER,GL_NEAREST);
      glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_MAG_FILTER,GL_NEAREST);
      glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_WRAP_S,GL_CLAMP_TO_EDGE);
      glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_WRAP_T,GL_CLAMP_TO_EDGE);
      glTexImage2D(GL_TEXTURE_2D,0,GL_RGBA,1,1,0,GL_RGBA,GL_UNSIGNED_BYTE,white);
   } else glBindTexture(GL_TEXTURE_2D,gl->omega_shade_texture);
   glEnable(GL_BLEND);glBlendEquation(GL_FUNC_ADD);
   glBlendFunc(GL_ZERO,GL_SRC_COLOR);
   gl3_draw_textured_quad(gl,gl->pipelines.alpha_blend,
         &gl->pipelines.alpha_blend_loc,gl->mvp_no_rot_yflip.data,quad);
   glDisable(GL_BLEND);gl3_blend_alpha(gl);
   glActiveTexture(GL_TEXTURE0);
}
