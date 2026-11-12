vec2 grid_band(vec2 x) {
    return floor(x) / 3.0 + min(fract(x), vec2(1.0 / 3.0));
}
void effect_grid(inout vec3 c, vec2 uv, vec2 f) {
    vec2 hp = 0.5 * u_uv.zw * u_src / (u_rect.zw * u_fbo);
    vec2 edge = (grid_band(f + hp) - grid_band(f - hp)) / (2.0 * hp);
    c *= 1.0 - 0.75 * max(edge.x, edge.y);
}
