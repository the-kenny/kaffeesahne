#version 330 core

in vec3 position;
in vec3 normal;

out vec3 fragNormal;
out vec3 fragVert;

uniform mat4 projectionMatrix;
uniform mat4 modelViewMatrix;

void main() {
  fragNormal = normal;
  fragVert = position;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
