/*
   Hyllian - Retro Shader - 2013

   A re-implementation from the original made by Hyllian and DOLLS!
   
   This program is free software; you can redistribute it and/or
   modify it under the terms of the GNU General Public License
   as published by the Free Software Foundation; either version 2
   of the License, or (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program; if not, write to the Free Software
   Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.
*/

// Fixed dark-gritty preset. Constants are compiled in, not exposed as parameters.
#if defined(VERTEX)

#if __VERSION__ >= 130
#define ATTRIBUTE in
#define VARYING out
#else
#define ATTRIBUTE attribute
#define VARYING varying
#endif

ATTRIBUTE vec2 VertexCoord;
ATTRIBUTE vec2 TexCoord;
uniform mat4 MVPMatrix;
VARYING vec2 tex_coord;

void main() {
    gl_Position = MVPMatrix * vec4(VertexCoord, 0.0, 1.0);
    tex_coord = TexCoord;
}

#elif defined(FRAGMENT)

#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif

#if __VERSION__ >= 130
#define VARYING in
#define SAMPLE_TEXTURE texture
out vec4 fragmentColor;
#else
#define VARYING varying
#define SAMPLE_TEXTURE texture2D
#define fragmentColor gl_FragColor
#endif

uniform sampler2D Texture;
uniform vec2 TextureSize;
uniform vec2 InputSize;
uniform vec2 OutputSize;
VARYING vec2 tex_coord;

// Retro-v2 defaults. This pass reads the original image directly.
const float PIXEL_FILL = 0.75;
const float PIXEL_INPUT_EXPONENT = 2.4;
const float PIXEL_OUTPUT_EXPONENT = 1.0 / 2.2;
const float PIXEL_FACE_GAIN = 1.04;
const float PIXEL_EDGE_GAIN = 0.36;

// Hue is measured in turns; saturation and value use the [0, 1] range.
const float HUE_SECTORS = 6.0;
const float DEGREES_PER_TURN = 360.0;
const float CYAN_HUE = 3.0 / HUE_SECTORS;
const float BLUE_HUE = 4.0 / HUE_SECTORS;
const float MAGENTA_HUE = 5.0 / HUE_SECTORS;
const float NEUTRAL_EPSILON = 0.000001;
const float SHIFT_EPSILON = 0.0000001;

// Offsets are (hue in turns, saturation, value).
const float HSV_BLEND = 0.4;
const vec3 CYAN_BLUE_SHIFT = vec3(-20.96 / DEGREES_PER_TURN, 0.163, -0.1922);
const vec3 MAGENTA_SHIFT = vec3(-36.63 / DEGREES_PER_TURN, -0.761, -0.093);

// Effective gains from the original preset, with constant arithmetic folded.
const float GREEN_MAGENTA_MUTE = 0.135; // 0.45 * 0.3
const float SATURATION = 0.76;
const vec3 WARMTH_GAIN = vec3(1.024, 0.973, 0.91); // Original warmth mix at 0.3
const float CONTRAST = 1.22;
const float CONTRAST_PIVOT = 0.5;
const float GAMMA_EXPONENT = 1.08;
const float BRIGHTNESS = 0.98;

// Original luma approximation, evaluated in the sampled color space.
const vec3 LUMA_WEIGHTS = vec3(0.2126, 0.7152, 0.0722);
const float SHADOW_STRENGTH = 0.1;
const float SHADOW_FADE_START = 0.05;
const float SHADOW_FADE_END = 0.65;
const float VIGNETTE_STRENGTH = 0.3;
const float VIGNETTE_RADIUS_SCALE = 0.85;
const float VIGNETTE_FADE_START = 0.30;
const float VIGNETTE_FADE_END = 1.15;

float luminance(vec3 rgb) {
    return dot(rgb, LUMA_WEIGHTS);
}

float hueSector(vec3 rgb, float highest, float chroma) {
    if (chroma <= NEUTRAL_EPSILON) return 0.0;
    // Red, green and blue occupy sectors 0, 2 and 4 of the HSV hexagon.
    if (highest == rgb.r) return (rgb.g - rgb.b) / chroma;
    if (highest == rgb.g) return 2.0 + (rgb.b - rgb.r) / chroma;
    return 4.0 + (rgb.r - rgb.g) / chroma;
}

vec3 rgbToHsv(vec3 rgb) {
    float highest = max(rgb.r, max(rgb.g, rgb.b));
    float lowest = min(rgb.r, min(rgb.g, rgb.b));
    float chroma = highest - lowest;
    float hue = fract(hueSector(rgb, highest, chroma) / HUE_SECTORS + 1.0);
    float saturation = highest > NEUTRAL_EPSILON ? chroma / highest : 0.0;
    return vec3(hue, saturation, highest);
}

vec3 hsvToRgb(vec3 hsv) {
    // RGB phase offsets and triangular ramps around the six-sector hue wheel.
    const vec3 channelPhases = vec3(0.0, 2.0 / 3.0, 1.0 / 3.0);
    vec3 ramps = abs(fract(hsv.xxx + channelPhases) * HUE_SECTORS - 3.0);
    vec3 pureHue = clamp(ramps - 1.0, 0.0, 1.0);
    return hsv.z * mix(vec3(1.0), pureHue, hsv.y);
}

float hueWeight(float hue, float center) {
    float distance = abs(hue - center);
    float circularDistance = min(distance, 1.0 - distance);
    return max(1.0 - circularDistance * HUE_SECTORS, 0.0);
}

vec3 hueShift(float hue) {
    float cyanBlueWeight = hueWeight(hue, CYAN_HUE) + hueWeight(hue, BLUE_HUE);
    float magentaWeight = hueWeight(hue, MAGENTA_HUE);
    return HSV_BLEND * (cyanBlueWeight * CYAN_BLUE_SHIFT + magentaWeight * MAGENTA_SHIFT);
}

vec3 selectiveColor(vec3 rgb) {
    vec3 hsv = rgbToHsv(rgb);
    if (hsv.y < NEUTRAL_EPSILON) return rgb;

    // All corrections use the original hue, without serial remapping.
    vec3 shift = hueShift(hsv.x);
    if (dot(abs(shift), vec3(1.0)) < SHIFT_EPSILON) return rgb;

    vec3 adjusted = vec3(fract(hsv.x + shift.x + 1.0),
                         clamp(hsv.yz + shift.yz, 0.0, 1.0));
    return hsvToRgb(adjusted);
}

vec3 mutedColor(vec3 rgb) {
    // Both masks use the same input, so neither correction affects the other.
    float greenExcess = max(rgb.g - max(rgb.r, rgb.b), 0.0);
    float magentaExcess = max(min(rgb.r, rgb.b) - rgb.g, 0.0);
    return rgb - GREEN_MAGENTA_MUTE * vec3(magentaExcess, greenExcess, magentaExcess);
}

vec3 saturatedColor(vec3 rgb) {
    return mix(vec3(luminance(rgb)), rgb, SATURATION);
}

vec3 toneMappedColor(vec3 rgb) {
    vec3 warm = rgb * WARMTH_GAIN;
    vec3 contrasted = clamp((warm - CONTRAST_PIVOT) * CONTRAST + CONTRAST_PIVOT, 0.0, 1.0);
    return pow(contrasted, vec3(GAMMA_EXPONENT)) * BRIGHTNESS;
}

vec3 shadedColor(vec3 rgb) {
    float shadowMask = 1.0 - smoothstep(SHADOW_FADE_START, SHADOW_FADE_END, luminance(rgb));
    return rgb * (1.0 - shadowMask * SHADOW_STRENGTH);
}

float vignetteGain(vec2 uv) {
    // Preserve the original texture-coordinate vignette and its aspect behavior.
    vec2 centered = uv * 2.0 - 1.0;
    float radius = length(centered * VIGNETTE_RADIUS_SCALE);
    return 1.0 - VIGNETTE_STRENGTH * smoothstep(VIGNETTE_FADE_START, VIGNETTE_FADE_END, radius);
}

vec3 gradedColor(vec3 rgb, vec2 uv) {
    vec3 selective = selectiveColor(rgb);
    vec3 muted = mutedColor(selective);
    vec3 saturated = saturatedColor(muted);
    vec3 toned = toneMappedColor(saturated);
    vec3 shaded = shadedColor(toned);
    return shaded * vignetteGain(uv);
}

// Original retro-v2 response, applied after grading. The unequal gamma
// exponents are intentional: replacing them would change the supplied effect.
vec3 pixelatedColor(vec3 rgb, vec2 uv) {
    vec3 linearColor = pow(rgb, vec3(PIXEL_INPUT_EXPONENT));
    vec2 pixelPosition = fract(uv * TextureSize);
    vec2 pixelFootprint = InputSize / OutputSize;
    vec2 edgeCoverage = clamp(
        clamp(pixelPosition + 0.5 * pixelFootprint, 0.0, 1.0) - PIXEL_FILL,
        vec2(0.0), pixelFootprint
    ) / pixelFootprint;
    float edgeWeight = max(edgeCoverage.x, edgeCoverage.y);
    vec3 face = linearColor * (PIXEL_FACE_GAIN + pixelPosition.x * pixelPosition.y);
    vec3 edge = linearColor * PIXEL_EDGE_GAIN;
    vec3 shadedPixel = mix(face, edge, edgeWeight);
    return clamp(pow(shadedPixel, vec3(PIXEL_OUTPUT_EXPONENT)), 0.0, 1.0);
}

void main() {
    vec4 source = SAMPLE_TEXTURE(Texture, tex_coord);
    vec3 graded = gradedColor(source.rgb, tex_coord);
    fragmentColor = vec4(pixelatedColor(graded, tex_coord), source.a);
}

#endif
