#version 330

in vec2 pos;
in vec3 color;
out vec3 v_color;

void main () {
  v_color = color;
  gl_Position = vec4(pos, 0.0, 1.0);
}
