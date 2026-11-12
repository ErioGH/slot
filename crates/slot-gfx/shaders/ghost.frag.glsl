
precision mediump float;
uniform sampler2D u_tex;
uniform sampler2D u_prev;
uniform vec2 u_keep;
varying vec2 v_uv;
void main() {
    vec3 now = texture2D(u_tex, v_uv).rgb;
    vec3 was = texture2D(u_prev, v_uv).rgb;
    vec3 keep = mix(vec3(u_keep.y), vec3(u_keep.x), step(was, now));
    vec3 c = mix(now, was, keep);
    c = mix(c, now, step(abs(now - was), vec3(1.5 / 255.0)));
    FRAG_COLOR = vec4(c, 1.0);
}
