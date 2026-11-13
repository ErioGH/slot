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

#pragma parameter RGB_TRANSFORM "RGB_TRANSFORM" 22 0 100 1
#pragma parameter BRIGHTNESS "BRIGHTNESS" 1.1 0.4 1.4 0.02
#pragma parameter SATURATION "SATURATION" 0.76 0 1.5 0.02
#pragma parameter CONTRAST "CONTRAST" 1.04 0.5 2 0.02
#pragma parameter GAMMA "GAMMA" 1.1 0.5 2 0.02
#pragma parameter WARMTH "WARMTH" 0.3 0 1 0.02
#pragma parameter GREEN_MUTE "GREEN_MUTE" 0.3 0 1 0.02
#pragma parameter RED_MUTE "RED_MUTE" 0 0 1 0.02
#pragma parameter BLUE_MUTE "BLUE_MUTE" 0 0 1 0.02
#pragma parameter YELLOW_MUTE "YELLOW_MUTE" 0 0 1 0.02
#pragma parameter TEAL_MUTE "TEAL_MUTE" 0 0 1 0.02
#pragma parameter MAGENTA_MUTE "MAGENTA_MUTE" 0.3 0 1 0.02
#pragma parameter SHADOWS "SHADOWS" 0.1 0 1 0.02
#pragma parameter VIGNETTE "VIGNETTE" 0.2 0 0.8 0.02
#pragma parameter BLACK_LIFT "BLACK_LIFT" 0 0 0.15 0.005
#pragma parameter PIXEL_STRENGTH "PIXEL_STRENGTH" 100 0 100 1
#pragma parameter PIXEL_FILL "PIXEL_FILL" 0.75 0 1 0.01
#pragma parameter PIXEL_INPUT_EXPONENT "PIXEL_INPUT_EXPONENT" 2.4 0.1 4 0.01
#pragma parameter PIXEL_OUTPUT_EXPONENT "PIXEL_OUTPUT_EXPONENT" 0.45454545454545453 0.1 2 0.000001
#pragma parameter PIXEL_FACE_GAIN "PIXEL_FACE_GAIN" 1.04 0 3 0.01
#pragma parameter PIXEL_EDGE_GAIN "PIXEL_EDGE_GAIN" 0.36 0 2 0.01
#pragma parameter RED_HUE "RED_HUE" 0 -180 180 0.01
#pragma parameter RED_SAT "RED_SAT" 0 -100 100 0.01
#pragma parameter RED_VAL "RED_VAL" 0 -100 100 0.01
#pragma parameter YELLOW_HUE "YELLOW_HUE" 0 -180 180 0.01
#pragma parameter YELLOW_SAT "YELLOW_SAT" 0 -100 100 0.01
#pragma parameter YELLOW_VAL "YELLOW_VAL" 0 -100 100 0.01
#pragma parameter GREEN_HUE "GREEN_HUE" 0 -180 180 0.01
#pragma parameter GREEN_SAT "GREEN_SAT" 0 -100 100 0.01
#pragma parameter GREEN_VAL "GREEN_VAL" 0 -100 100 0.01
#pragma parameter TEAL_HUE "TEAL_HUE" 0 -180 180 0.01
#pragma parameter TEAL_SAT "TEAL_SAT" 0 -100 100 0.01
#pragma parameter TEAL_VAL "TEAL_VAL" 0 -100 100 0.01
#pragma parameter BLUE_HUE "BLUE_HUE" 0 -180 180 0.01
#pragma parameter BLUE_SAT "BLUE_SAT" 0 -100 100 0.01
#pragma parameter BLUE_VAL "BLUE_VAL" 0 -100 100 0.01
#pragma parameter MAGENTA_HUE "MAGENTA_HUE" 0 -180 180 0.01
#pragma parameter MAGENTA_SAT "MAGENTA_SAT" 0 -100 100 0.01
#pragma parameter MAGENTA_VAL "MAGENTA_VAL" 0 -100 100 0.01
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
#define varying in
#define texture2D texture
out vec4 fragmentColor;
#define gl_FragColor fragmentColor
#endif
uniform vec2 TextureSize;
uniform vec2 InputSize;
uniform vec2 OutputSize;
uniform sampler2D Texture;
varying vec2 tex_coord;
uniform float RGB_TRANSFORM;
uniform float BRIGHTNESS;
uniform float SATURATION;
uniform float CONTRAST;
uniform float GAMMA;
uniform float WARMTH;
uniform float GREEN_MUTE;
uniform float RED_MUTE;
uniform float BLUE_MUTE;
uniform float YELLOW_MUTE;
uniform float TEAL_MUTE;
uniform float MAGENTA_MUTE;
uniform float SHADOWS;
uniform float VIGNETTE;
uniform float BLACK_LIFT;
uniform float PIXEL_STRENGTH;
uniform float PIXEL_FILL;
uniform float PIXEL_INPUT_EXPONENT;
uniform float PIXEL_OUTPUT_EXPONENT;
uniform float PIXEL_FACE_GAIN;
uniform float PIXEL_EDGE_GAIN;
uniform float RED_HUE;
uniform float RED_SAT;
uniform float RED_VAL;
uniform float YELLOW_HUE;
uniform float YELLOW_SAT;
uniform float YELLOW_VAL;
uniform float GREEN_HUE;
uniform float GREEN_SAT;
uniform float GREEN_VAL;
uniform float TEAL_HUE;
uniform float TEAL_SAT;
uniform float TEAL_VAL;
uniform float BLUE_HUE;
uniform float BLUE_SAT;
uniform float BLUE_VAL;
uniform float MAGENTA_HUE;
uniform float MAGENTA_SAT;
uniform float MAGENTA_VAL;
float luminance(vec3 c){return dot(c,vec3(0.2126,0.7152,0.0722));}
vec3 rgbToHsv(vec3 c){
float hi=max(c.r,max(c.g,c.b)),lo=min(c.r,min(c.g,c.b)),d=hi-lo;
float h=0.0;
if(d>0.000001){
if(hi==c.r)h=(c.g-c.b)/d;
else if(hi==c.g)h=2.0+(c.b-c.r)/d;
else h=4.0+(c.r-c.g)/d;
h=fract(h/6.0+1.0);
}
return vec3(h,hi>0.000001?d/hi:0.0,hi);
}
vec3 hsvToRgb(vec3 c){
vec3 p=abs(fract(c.xxx+vec3(0.0,2.0/3.0,1.0/3.0))*6.0-3.0);
return c.z*mix(vec3(1.0),clamp(p-1.0,0.0,1.0),c.y);
}
float hueWeight(float h,float center){
float d=abs(h-center);d=min(d,1.0-d);
return max(1.0-d*6.0,0.0);
}
vec3 selectiveHSV(vec3 rgb){
vec3 hsv=rgbToHsv(rgb);
// Leave neutral pixels unchanged. All weights come from the original hue.
if(hsv.y<0.000001)return rgb;
vec3 shift=vec3(0.0);
shift+=hueWeight(hsv.x,0.0000000000)*vec3(RED_HUE/360.0,RED_SAT/100.0,RED_VAL/100.0);
shift+=hueWeight(hsv.x,0.1666666667)*vec3(YELLOW_HUE/360.0,YELLOW_SAT/100.0,YELLOW_VAL/100.0);
shift+=hueWeight(hsv.x,0.3333333333)*vec3(GREEN_HUE/360.0,GREEN_SAT/100.0,GREEN_VAL/100.0);
shift+=hueWeight(hsv.x,0.5000000000)*vec3(TEAL_HUE/360.0,TEAL_SAT/100.0,TEAL_VAL/100.0);
shift+=hueWeight(hsv.x,0.6666666667)*vec3(BLUE_HUE/360.0,BLUE_SAT/100.0,BLUE_VAL/100.0);
shift+=hueWeight(hsv.x,0.8333333333)*vec3(MAGENTA_HUE/360.0,MAGENTA_SAT/100.0,MAGENTA_VAL/100.0);
// Same calibrated HSV offsets as the RGB example, scaled by one uniform.
// Add to manual offsets using the same original-hue weights (no serial remapping).
vec3 exampleShift=(hueWeight(hsv.x,0.5)+hueWeight(hsv.x,2.0/3.0))
    *vec3(-20.96/360.0,16.3/100.0,-19.22/100.0)
    +hueWeight(hsv.x,5.0/6.0)*vec3(-36.63/360.0,-76.1/100.0,-9.3/100.0);
shift+=(RGB_TRANSFORM/100.0)*exampleShift;
if(dot(abs(shift),vec3(1.0))<0.0000001)return rgb;
hsv.x=fract(hsv.x+shift.x+1.0);
hsv.yz=clamp(hsv.yz+shift.yz,0.0,1.0);
return hsvToRgb(hsv);
}
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

void main(){
vec4 source=texture2D(Texture,tex_coord); vec3 color=selectiveHSV(source.rgb);
// All six mute masks use the same RGB after HSV correction: slider order cannot affect the result.
vec3 rgb=color;
float redExcess=max(rgb.r-max(rgb.g,rgb.b),0.0);
float greenExcess=max(rgb.g-max(rgb.r,rgb.b),0.0);
float blueExcess=max(rgb.b-max(rgb.r,rgb.g),0.0);
float yellowExcess=max(min(rgb.r,rgb.g)-rgb.b,0.0);
float tealExcess=max(min(rgb.g,rgb.b)-rgb.r,0.0);
float magentaExcess=max(min(rgb.r,rgb.b)-rgb.g,0.0);
vec3 primary=vec3(redExcess*RED_MUTE,greenExcess*GREEN_MUTE,blueExcess*BLUE_MUTE);
vec3 secondary=vec3(1.0,1.0,0.0)*(yellowExcess*YELLOW_MUTE)
             +vec3(0.0,1.0,1.0)*(tealExcess*TEAL_MUTE)
             +vec3(1.0,0.0,1.0)*(magentaExcess*MAGENTA_MUTE);
color-=0.45*(primary+secondary);
float luma=luminance(color);color=mix(vec3(luma),color,SATURATION);
color*=mix(vec3(1.0),vec3(1.08,0.91,0.70),WARMTH);
color=clamp((color-0.5)*CONTRAST+0.5,0.0,1.0);
color=pow(color,vec3(GAMMA));color*=BRIGHTNESS;
float shadowMask=1.0-smoothstep(0.05,0.65,luminance(color));
color*=1.0-shadowMask*SHADOWS;
color=color*(1.0-BLACK_LIFT)+vec3(BLACK_LIFT);
vec2 centered=tex_coord*2.0-1.0;
float radius=length(centered*vec2(0.85,0.85));
color*=1.0-VIGNETTE*smoothstep(0.30,1.15,radius);
if(PIXEL_STRENGTH>0.0)color=mix(color,pixelatedColor(color,tex_coord),PIXEL_STRENGTH/100.0);
gl_FragColor=vec4(clamp(color,0.0,1.0),source.a);
}
#endif
