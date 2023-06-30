#version 330

uniform float window_aspect_ratio;

in vec3 position;
in vec2 texture_coordinates;
out vec2 v_texture_coordinates;
out vec3 v_position; // untransformed vertex position

void main () {
  v_position = position;
  v_texture_coordinates = texture_coordinates;
  gl_Position = vec4(position, 1.0);
  if (window_aspect_ratio > 1) {
    gl_Position.x /= window_aspect_ratio;
  }
  else {
    gl_Position.y *= window_aspect_ratio;
  }
}
