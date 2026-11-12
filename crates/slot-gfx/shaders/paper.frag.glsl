
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D u_tex;
uniform vec2 pv;
uniform float sc, lift, tear, fold, mode, bow;
varying vec2 P;
const vec2 SZ = vec2(226.0, 425.0);
const vec2 D = vec2(-0.62, -0.78);
const float CUTY = 407.0;
const float TH = 0.42;
const float SQ = 1.5;
const float R = 24.0;
const float PI = 3.14159265;
vec2 local(vec2 s) {
    return (s - pv) / sc + SZ;
}
float alphaAt(vec2 q) {
    if (q.x < 0.0 || q.y < 0.0 || q.x > SZ.x || q.y > SZ.y) return 0.0;
    if (q.x < tear) {
        float saw = abs(fract(q.x / 6.0) - 0.5) * 5.0;
        if (q.y > CUTY - saw) return 0.0;
    }
    return texture2D(u_tex, q / SZ).a;
}
vec4 front(vec2 q) {
    if (alphaAt(q) <= 0.0) return vec4(0.0);
    vec4 t = texture2D(u_tex, q / SZ);
    t.rgb /= t.a;
    return t;
}
bool inside(vec2 q) {
    return alphaAt(q) > 0.5;
}
vec4 sheet(vec2 Q) {
    float d = dot(Q, D) + fold * dot(D, D);
    vec2 B = Q - D * d;
    vec3 backc = vec3(0.885, 0.895, 0.88);
    if (mode > 0.5) {
        if (d < R) {
            float s = d < 0.0 ? PI * R - d * SQ : R * (PI - asin(d / R));
            vec2 q = B + D * s;
            if (inside(q)) {
                float nz = d < 0.0 ? 1.0 : sqrt(max(0.0, 1.0 - d * d / (R * R)));
                float l = mix(0.70, 0.97, nz);
                float ghost = 1.0 - 0.07 * (1.0 - dot(front(q).rgb, vec3(0.333)));
                return vec4(backc * l * ghost, 1.0);
            }
        }
        return vec4(0.0);
    }
    if (d >= 0.0 && d < R) {
        float s = R * asin(d / R);
        vec2 q = B + D * s;
        if (inside(q)) {
            float nz = sqrt(max(0.0, 1.0 - d * d / (R * R)));
            vec4 t = front(q);
            t.rgb *= mix(0.74, 1.0, nz);
            return t;
        }
    }
    if (d < 0.0) {
        vec4 t = front(Q);
        if (t.a > 0.0) {
            t.rgb *= 1.0 - 0.20 * exp(d / (R * 0.9));
            return t;
        }
    }
    return vec4(0.0);
}
float cover(vec2 Q) {
    float d = dot(Q, D) + fold * dot(D, D);
    if (d >= R) return 0.0;
    vec2 B = Q - D * d;
    if (mode > 0.5) {
        float s = d < 0.0 ? PI * R - d * SQ : R * (PI - asin(d / R));
        return inside(B + D * s) ? 1.0 : 0.0;
    }
    if (d >= 0.0) {
        float a = alphaAt(B + D * (R * asin(d / R)));
        return a > 0.5 ? a : 0.0;
    }
    return alphaAt(Q);
}
vec2 unbow(vec2 Q) {
    if (bow <= 0.0) return Q;
    float u = (Q.y - CUTY * 0.5) / (CUTY * 0.5) * (TH / sin(TH));
    float su = u * sin(TH);
    if (abs(su) >= 1.0) return vec2(-1.0);
    float f = asin(su) / TH;
    float w = 1.0 + 0.022 * sqrt(1.0 - su * su);
    return vec2(113.0 + (Q.x - 113.0) / w, CUTY * 0.5 + f * CUTY * 0.5);
}
void main() {
    vec2 Q = unbow(local(P));
    vec4 s = sheet(Q);
    if (bow > 0.0 && s.a > 0.0) {
        float n = clamp((Q.y - CUTY * 0.5) / (CUTY * 0.5) * sin(TH), -1.0, 1.0);
        s.rgb *= 1.0 - (0.13 * max(n, 0.0) + 0.03 * max(-n, 0.0)) + 0.02 * (1.0 - abs(n));
        s.rgb *= 1.0 - 0.10 * pow(abs(n) / sin(TH), 5.0);
    }
    float aa = s.a;
    float sh = 0.0;
    if (aa < 1.0) {
        for (int i = 0; i < 8; i++) {
            float a = float(i) * 0.785;
            vec2 o = vec2(cos(a), sin(a)) * lift * 0.55 + vec2(lift * 0.3, lift * 0.42);
            sh += cover(unbow(local(P - o))) / 8.0;
        }
    }
    float al = aa + (1.0 - aa) * sh * 0.5;
    FRAG_COLOR = vec4(s.rgb * aa, al);
}
