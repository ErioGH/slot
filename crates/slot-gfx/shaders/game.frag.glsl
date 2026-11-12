
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D u_game;
uniform sampler2D u_paper;
uniform vec2 u_src;
uniform vec4 u_uv;
uniform float u_bright;
uniform float u_mode;
uniform float u_paper_size;
uniform vec2 u_origin;
uniform vec4 u_rect;
uniform vec2 u_fbo;
uniform vec4 u_pic;
varying vec2 v_uv;
void effect_lcd_grid(inout vec3 c, vec2 uv, vec2 f);
void effect_shimmerless(inout vec3 c, vec2 uv, vec2 f);
void effect_simpletex(inout vec3 c, vec2 uv, vec2 f);
void effect_dot(inout vec3 c, vec2 uv, vec2 f);
void effect_grid(inout vec3 c, vec2 uv, vec2 f);
void effect_lcd3x(inout vec3 c, vec2 uv, vec2 f);
void effect_dark_gritty_retro(inout vec3 c, vec2 uv, vec2 f);
void main() {
    vec2 uv = u_uv.xy + v_uv * u_uv.zw;
    vec2 f = fract(uv * u_src);
    vec3 c = texture2D(u_game, uv).rgb;
    bool inside = all(greaterThanEqual(uv, u_pic.xy)) && all(lessThan(uv, u_pic.zw));
    if (!inside) {
    } else if (u_mode > 6.5) {
        effect_dark_gritty_retro(c, uv, f);
    } else if (u_mode > 5.5) {
        effect_lcd_grid(c, uv, f);
    } else if (u_mode > 4.5) {
        effect_shimmerless(c, uv, f);
    } else if (u_mode > 3.5) {
        effect_simpletex(c, uv, f);
    } else if (u_mode > 2.5) {
        effect_dot(c, uv, f);
    } else if (u_mode > 1.5) {
        effect_grid(c, uv, f);
    } else if (u_mode > 0.5) {
        effect_lcd3x(c, uv, f);
    }
    FRAG_COLOR = vec4(c * u_bright, 1.0);
}
