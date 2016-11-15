#version 330 core

flat in vec3 fragPosition;
out vec4 color;

void main() {
  color = vec4(fragPosition, 1.0);
}
