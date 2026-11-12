
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
uniform vec2 u_turn;
uniform vec4 u_src;
uniform float u_quad;
uniform vec4 u_c01;
uniform vec4 u_c23;
varying vec2 v_uv;
void main() {
    v_uv = u_src.xy + a_pos * u_src.zw;
    vec2 mid = u_rect.zw * 0.5;
    vec2 local = a_pos * u_rect.zw - mid;
    vec2 turned = vec2(u_turn.x * local.x - u_turn.y * local.y,
                       u_turn.y * local.x + u_turn.x * local.y);
    vec2 flat_p = u_rect.xy + a_pos * u_rect.zw + (turned - local);
    vec2 top = mix(u_c01.xy, u_c01.zw, a_pos.x);
    vec2 bottom = mix(u_c23.zw, u_c23.xy, a_pos.x);
    vec2 quad_p = mix(top, bottom, a_pos.y);
    vec2 p = mix(flat_p, quad_p, u_quad) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
