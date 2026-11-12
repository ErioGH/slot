float dot_weight(vec2 f, vec2 o, float bloom) {
    return exp(-4.0 * length(f - (o + vec2(0.5))) * bloom);
}
void effect_dot(inout vec3 c, vec2 uv, vec2 f) {
    float bloom = mix(1.05, 0.95, dot(c, vec3(0.30, 0.59, 0.11)));
    float mid = dot_weight(f, vec2(0.0), bloom);
    float sum = 0.0;
    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            sum += dot_weight(f, vec2(float(i), float(j)), bloom);
        }
    }
    c = mix(c, mix(1.2 * c * mid, c * sum, 0.85), 0.85);
}
