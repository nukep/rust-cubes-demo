#version 130

in vec3 position;
out vec3 fragment_coord;
out vec3 npos;
uniform mat4 projection_view;
uniform mat4 model;

uniform vec3 cube_pos;
uniform float cube_size;

void main(void) {
    vec4 v = vec4(position, 1.0);
    vec4 p = projection_view * model * v;
    // position ranges from (-0.5,-0,5,-0.5) to (+0.5,+0.5,+0.5)
    // 1. normalize to 0..1
    npos = position + 0.5;
    // range from cube_pos to cube_pos+cube_size
    fragment_coord = mix(cube_pos, cube_pos+cube_size, npos);
    gl_Position = p;
}
