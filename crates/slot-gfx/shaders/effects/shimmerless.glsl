void effect_shimmerless(inout vec3 c, vec2 uv, vec2 f) {
    vec2 hp = 0.5 * u_uv.zw * u_src / (u_rect.zw * u_fbo);
    vec2 p = uv * u_src;
    vec2 b = floor(p + hp);
    vec2 t = clamp((p + hp - b) / (2.0 * hp), 0.0, 1.0);
    vec2 lo = u_pic.xy * u_src + 0.5;
    vec2 hi = u_pic.zw * u_src - 0.5;
    vec2 a0 = clamp(b - 0.5, lo, hi) / u_src;
    vec2 a1 = clamp(b + 0.5, lo, hi) / u_src;
    vec3 top = mix(texture2D(u_game, a0).rgb, texture2D(u_game, vec2(a1.x, a0.y)).rgb, t.x);
    vec3 bottom = mix(texture2D(u_game, vec2(a0.x, a1.y)).rgb, texture2D(u_game, a1).rgb, t.x);
    c = mix(top, bottom, t.y);
}
