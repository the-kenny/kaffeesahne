#version 330 core

in vec3 position;
in vec3 normal;

out vec3 fragNormal;
out vec3 fragVert;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;

void main() {
  fragNormal = normal;
  fragVert = position;

  mat4 modelViewProject = projectionMatrix * viewMatrix * modelMatrix;
  gl_Position = modelViewProject * vec4(position, 1.0);
}
