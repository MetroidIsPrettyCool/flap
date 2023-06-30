#version 330

uniform sampler2D texture_atlas;

in vec2 v_texture_coordinates;
in vec3 v_position;

out vec4 color;

void main () {
  if (v_position.x > 1.0 || v_position.x < -1.0 || v_position.y > 1.0 || v_position.y < -1.0) discard;
  color = texture(texture_atlas, v_texture_coordinates);
  if (color.a < 0.5) discard;
}
