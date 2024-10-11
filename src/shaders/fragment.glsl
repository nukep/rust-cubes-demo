#version 100
precision highp float;

varying vec3 fragment_coord;
varying vec3 npos;

uniform bool show_outlines;
uniform bool hovered;
uniform float cube_size;

// Source: http://lolengine.net/blog/2013/07/27/rgb-to-hsv-in-glsl
vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

bool is_edge(float x) {
    float m = 0.005/cube_size;
    return (x < m) || (x > (1.0-m));
}

void main(void) {
    // Hue: 0.0 to 1.0
    // Saturation: 0.0 to 1.0
    // Lightness: 0.5 to 1.0
    vec3 hsv = mix(vec3(0.0, 0.0, 0.25), vec3(1.0, 1.0, 1.0), fragment_coord);
    vec3 rgb = hsv2rgb(hsv);

    if (show_outlines) {
        bool e_x = is_edge(npos.x);
        bool e_y = is_edge(npos.y);
        bool e_z = is_edge(npos.z);

        if ((e_x && e_y) || (e_x && e_z) || (e_y && e_z)) {
            rgb = rgb * 0.5;
        }
    }

    if (hovered) {
        // tint red if hovered
        rgb = mix(vec3(0.9, 0.25, 0.25), vec3(1.0), rgb);
    }
    gl_FragColor = vec4(rgb, 1.0);
}
