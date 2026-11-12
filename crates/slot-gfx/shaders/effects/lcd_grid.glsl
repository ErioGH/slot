const vec3 lcd_centres = vec3(1.0 / 6.0, 0.5, 5.0 / 6.0);
const float lcd_stripe = 0.9;
const float lcd_gap_y = 0.35;
vec3 lcd_ramp(vec3 x) {
    vec3 u = fract(x + 0.5) - 0.5;
    return u - 2.0 * u * abs(u);
}
void effect_lcd_grid(inout vec3 c, vec2 uv, vec2 f) {
    vec2 hp = 0.5 * u_uv.zw * u_src / (u_rect.zw * u_fbo);
    vec2 p = uv * u_src;
    vec2 b = floor(p + hp);
    vec2 l0 = min(p - hp, b) - b + 1.0;
    vec2 r0 = max(p - hp, b) - b;
    vec2 r1 = p + hp - b;
    mediump vec3 gl0 = lcd_ramp(l0.x - lcd_centres);
    mediump vec3 gr0 = lcd_ramp(r0.x - lcd_centres);
    mediump vec3 gr1 = lcd_ramp(r1.x - lcd_centres);
    mediump vec3 gy = lcd_ramp(vec3(l0.y, r0.y, r1.y) - 0.5);
    mediump vec3 wl = vec3(1.0 - l0.x) + lcd_stripe * (lcd_ramp(-lcd_centres) - gl0);
    mediump vec3 wr = vec3(r1.x - r0.x) + lcd_stripe * (gr1 - gr0);
    mediump float wt = (1.0 - l0.y) - lcd_gap_y * gy.x;
    mediump float wb = (r1.y - r0.y) + lcd_gap_y * (gy.z - gy.y);
    vec2 lo = u_pic.xy * u_src + 0.5;
    vec2 hi = u_pic.zw * u_src - 0.5;
    vec2 a0 = clamp(b - 0.5, lo, hi) / u_src;
    vec2 a1 = clamp(b + 0.5, lo, hi) / u_src;
    mediump vec3 c00 = texture2D(u_game, a0).rgb;
    mediump vec3 c10 = texture2D(u_game, vec2(a1.x, a0.y)).rgb;
    mediump vec3 c01 = texture2D(u_game, vec2(a0.x, a1.y)).rgb;
    mediump vec3 c11 = texture2D(u_game, a1).rgb;
    mediump vec3 lit = (c00 * c00 * wl + c10 * c10 * wr) * wt + (c01 * c01 * wl + c11 * c11 * wr) * wb;
    c = sqrt(clamp(lit / (4.0 * hp.x * hp.y), 0.0, 1.0));
}
