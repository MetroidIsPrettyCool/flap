#version 330

uniform sampler2D texture_atlas;

in vec2 v_texture_coordinates;

out vec4 color;

void main () {
  color = texture(texture_atlas, v_texture_coordinates);
  if (color.a < 0.5) discard;
}
