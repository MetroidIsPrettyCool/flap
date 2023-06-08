#version 330

uniform float window_aspect_ratio;

in vec3 position;
in vec3 color;
out vec3 v_color;

void main () {
  v_color = color;
  gl_Position = vec4(position, 1.0);
  if (window_aspect_ratio > 1) {
    gl_Position.x /= window_aspect_ratio;
  }
  else {
    gl_Position.y *= window_aspect_ratio;
  }
}
