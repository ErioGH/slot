
precision mediump float;
uniform sampler2D u_tex;
uniform vec3 u_gain;
uniform float u_saturation;
uniform float u_contrast;
varying vec2 v_uv;
void main() {
    vec3 c = texture2D(u_tex, v_uv).rgb;
    c = mix(vec3(dot(c, vec3(0.2126, 0.7152, 0.0722))), c, 1.0 + u_saturation);
    c = (c - 0.5) * (1.0 + u_contrast) + 0.5;
    FRAG_COLOR = vec4(clamp(c, 0.0, 1.0) * u_gain, 1.0);
}
