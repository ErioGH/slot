const vec3 luma_coeff = vec3(0.2126, 0.7152, 0.0722);
void effect_simpletex(inout vec3 c, vec2 uv, vec2 f) {
    c = pow(c, vec3(1.05));
    vec2 d = abs(f - 0.5);
    float x2 = max(d.x, d.y);
    x2 = x2 * x2;
    float x4 = x2 * x2;
    float w = 48.0 * (x4 - (8.0 / 3.0) * x4 * x2);
    w = w * (w + (1.0 - w) * 0.65) * 0.65;
    float luma = dot(c, luma_coeff);
    w = w * luma;
    c = mix(c, vec3(1.0), w);
    vec3 bg = vec3(texture2D(u_paper, (floor(gl_FragCoord.xy - u_origin) + 0.5) / u_paper_size).r) * c;
    c = mix(c, bg, dot(c, luma_coeff));
}
