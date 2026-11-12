
attribute vec2 a_pos;
uniform vec4 u_uv;
varying vec2 v_uv;
void main() {
    v_uv = u_uv.xy + a_pos * u_uv.zw;
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
