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

// Slot port: shared sampling, texture dimensions and output geometry.
// Retro-v2 defaults. This pass reads the original image directly.
const float dgr_PIXEL_FILL = 0.75;
const float dgr_PIXEL_INPUT_EXPONENT = 2.4;
const float dgr_PIXEL_OUTPUT_EXPONENT = 1.0 / 2.2;
const float dgr_PIXEL_FACE_GAIN = 1.04;
const float dgr_PIXEL_EDGE_GAIN = 0.36;

// Hue is measured in turns; saturation and value use the [0, 1] range.
const float dgr_HUE_SECTORS = 6.0;
const float dgr_DEGREES_PER_TURN = 360.0;
const float dgr_CYAN_HUE = 3.0 / dgr_HUE_SECTORS;
const float dgr_BLUE_HUE = 4.0 / dgr_HUE_SECTORS;
const float dgr_MAGENTA_HUE = 5.0 / dgr_HUE_SECTORS;
const float dgr_NEUTRAL_EPSILON = 0.000001;
const float dgr_SHIFT_EPSILON = 0.0000001;

// Offsets are (hue in turns, saturation, value).
const float dgr_HSV_BLEND = 0.4;
const vec3 dgr_CYAN_BLUE_SHIFT = vec3(-20.96 / dgr_DEGREES_PER_TURN, 0.163, -0.1922);
const vec3 dgr_MAGENTA_SHIFT = vec3(-36.63 / dgr_DEGREES_PER_TURN, -0.761, -0.093);

// Effective gains from the original preset, with constant arithmetic folded.
const float dgr_GREEN_MAGENTA_MUTE = 0.135; // 0.45 * 0.3
const float dgr_SATURATION = 0.76;
const vec3 dgr_WARMTH_GAIN = vec3(1.024, 0.973, 0.91); // Original warmth mix at 0.3
const float dgr_CONTRAST = 1.22;
const float dgr_CONTRAST_PIVOT = 0.5;
const float dgr_GAMMA_EXPONENT = 1.08;
const float dgr_BRIGHTNESS = 0.98;

// Original luma approximation, evaluated in the sampled color space.
const vec3 dgr_LUMA_WEIGHTS = vec3(0.2126, 0.7152, 0.0722);
const float dgr_SHADOW_STRENGTH = 0.1;
const float dgr_SHADOW_FADE_START = 0.05;
const float dgr_SHADOW_FADE_END = 0.65;
const float dgr_VIGNETTE_STRENGTH = 0.3;
const float dgr_VIGNETTE_RADIUS_SCALE = 0.85;
const float dgr_VIGNETTE_FADE_START = 0.30;
const float dgr_VIGNETTE_FADE_END = 1.15;

float dgr_luminance(vec3 rgb) {
    return dot(rgb, dgr_LUMA_WEIGHTS);
}

float dgr_hueSector(vec3 rgb, float highest, float chroma) {
    if (chroma <= dgr_NEUTRAL_EPSILON) return 0.0;
    // Red, green and blue occupy sectors 0, 2 and 4 of the HSV hexagon.
    if (highest == rgb.r) return (rgb.g - rgb.b) / chroma;
    if (highest == rgb.g) return 2.0 + (rgb.b - rgb.r) / chroma;
    return 4.0 + (rgb.r - rgb.g) / chroma;
}

vec3 dgr_rgbToHsv(vec3 rgb) {
    float highest = max(rgb.r, max(rgb.g, rgb.b));
    float lowest = min(rgb.r, min(rgb.g, rgb.b));
    float chroma = highest - lowest;
    float hue = fract(dgr_hueSector(rgb, highest, chroma) / dgr_HUE_SECTORS + 1.0);
    float saturation = highest > dgr_NEUTRAL_EPSILON ? chroma / highest : 0.0;
    return vec3(hue, saturation, highest);
}

vec3 dgr_hsvToRgb(vec3 hsv) {
    // RGB phase offsets and triangular ramps around the six-sector hue wheel.
    const vec3 dgr_channelPhases = vec3(0.0, 2.0 / 3.0, 1.0 / 3.0);
    vec3 ramps = abs(fract(hsv.xxx + dgr_channelPhases) * dgr_HUE_SECTORS - 3.0);
    vec3 pureHue = clamp(ramps - 1.0, 0.0, 1.0);
    return hsv.z * mix(vec3(1.0), pureHue, hsv.y);
}

float dgr_hueWeight(float hue, float center) {
    float distance = abs(hue - center);
    float circularDistance = min(distance, 1.0 - distance);
    return max(1.0 - circularDistance * dgr_HUE_SECTORS, 0.0);
}

vec3 dgr_hueShift(float hue) {
    float cyanBlueWeight = dgr_hueWeight(hue, dgr_CYAN_HUE) + dgr_hueWeight(hue, dgr_BLUE_HUE);
    float magentaWeight = dgr_hueWeight(hue, dgr_MAGENTA_HUE);
    return dgr_HSV_BLEND * (cyanBlueWeight * dgr_CYAN_BLUE_SHIFT + magentaWeight * dgr_MAGENTA_SHIFT);
}

vec3 dgr_selectiveColor(vec3 rgb) {
    vec3 hsv = dgr_rgbToHsv(rgb);
    if (hsv.y < dgr_NEUTRAL_EPSILON) return rgb;

    // All corrections use the original hue, without serial remapping.
    vec3 shift = dgr_hueShift(hsv.x);
    if (dot(abs(shift), vec3(1.0)) < dgr_SHIFT_EPSILON) return rgb;

    vec3 adjusted = vec3(fract(hsv.x + shift.x + 1.0),
                         clamp(hsv.yz + shift.yz, 0.0, 1.0));
    return dgr_hsvToRgb(adjusted);
}

vec3 dgr_mutedColor(vec3 rgb) {
    // Both masks use the same input, so neither correction affects the other.
    float greenExcess = max(rgb.g - max(rgb.r, rgb.b), 0.0);
    float magentaExcess = max(min(rgb.r, rgb.b) - rgb.g, 0.0);
    return rgb - dgr_GREEN_MAGENTA_MUTE * vec3(magentaExcess, greenExcess, magentaExcess);
}

vec3 dgr_saturatedColor(vec3 rgb) {
    return mix(vec3(dgr_luminance(rgb)), rgb, dgr_SATURATION);
}

vec3 dgr_toneMappedColor(vec3 rgb) {
    vec3 warm = rgb * dgr_WARMTH_GAIN;
    vec3 contrasted = clamp((warm - dgr_CONTRAST_PIVOT) * dgr_CONTRAST + dgr_CONTRAST_PIVOT, 0.0, 1.0);
    return pow(contrasted, vec3(dgr_GAMMA_EXPONENT)) * dgr_BRIGHTNESS;
}

vec3 dgr_shadedColor(vec3 rgb) {
    float shadowMask = 1.0 - smoothstep(dgr_SHADOW_FADE_START, dgr_SHADOW_FADE_END, dgr_luminance(rgb));
    return rgb * (1.0 - shadowMask * dgr_SHADOW_STRENGTH);
}

float dgr_vignetteGain(vec2 uv) {
    // Preserve the original texture-coordinate vignette and its aspect behavior.
    vec2 centered = uv * 2.0 - 1.0;
    float radius = length(centered * dgr_VIGNETTE_RADIUS_SCALE);
    return 1.0 - dgr_VIGNETTE_STRENGTH * smoothstep(dgr_VIGNETTE_FADE_START, dgr_VIGNETTE_FADE_END, radius);
}

vec3 dgr_gradedColor(vec3 rgb, vec2 uv) {
    vec3 selective = dgr_selectiveColor(rgb);
    vec3 muted = dgr_mutedColor(selective);
    vec3 saturated = dgr_saturatedColor(muted);
    vec3 toned = dgr_toneMappedColor(saturated);
    vec3 shaded = dgr_shadedColor(toned);
    return shaded * dgr_vignetteGain(uv);
}

// Original retro-v2 response, applied after grading. The unequal gamma
// exponents are intentional: replacing them would change the supplied effect.
vec3 dgr_pixelatedColor(vec3 rgb, vec2 uv) {
    vec3 linearColor = pow(rgb, vec3(dgr_PIXEL_INPUT_EXPONENT));
    vec2 pixelPosition = fract(uv * u_src);
    vec2 pixelFootprint = (u_uv.zw * u_src) / (u_rect.zw * u_fbo);
    vec2 edgeCoverage = clamp(
        clamp(pixelPosition + 0.5 * pixelFootprint, 0.0, 1.0) - dgr_PIXEL_FILL,
        vec2(0.0), pixelFootprint
    ) / pixelFootprint;
    float edgeWeight = max(edgeCoverage.x, edgeCoverage.y);
    vec3 face = linearColor * (dgr_PIXEL_FACE_GAIN + pixelPosition.x * pixelPosition.y);
    vec3 edge = linearColor * dgr_PIXEL_EDGE_GAIN;
    vec3 shadedPixel = mix(face, edge, edgeWeight);
    return clamp(pow(shadedPixel, vec3(dgr_PIXEL_OUTPUT_EXPONENT)), 0.0, 1.0);
}

void effect_dark_gritty_retro(inout vec3 c, vec2 uv, vec2 f) {
    c = dgr_pixelatedColor(dgr_gradedColor(c, uv), uv);
}
