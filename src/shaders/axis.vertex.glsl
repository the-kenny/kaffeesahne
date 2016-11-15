#version 330 core

in vec3 position;
flat out vec3 fragPosition;

uniform mat4 projectionMatrix;
uniform mat4 viewMatrix;

void main() {
  fragPosition = position;
  gl_Position = projectionMatrix * viewMatrix * vec4(position, 1.0);
}
