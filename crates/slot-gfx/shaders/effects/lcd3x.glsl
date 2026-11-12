const vec3 offsets = vec3(3.141592654) * vec3(1.0 / 2.0, 1.0 / 2.0 - 2.0 / 3.0, 1.0 / 2.0 - 4.0 / 3.0);
void effect_lcd3x(inout vec3 c, vec2 uv, vec2 f) {
    vec2 angle = f * 6.283185307;
    float yfactor = (16.0 + sin(angle.y)) / 17.0;
    vec3 xfactors = (4.0 + sin(angle.x + offsets)) / 5.0;
    c *= yfactor * xfactors;
}
