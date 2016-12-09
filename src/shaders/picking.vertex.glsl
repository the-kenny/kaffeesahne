#version 330 core

in vec3 position;

layout(std140)
uniform Uniforms {
  uint pickingId;

  mat4 modelMatrix;
  mat4 normalMatrix;
  mat4 viewMatrix;
  mat4 projectionMatrix;

  vec3 lightPosition;
  vec3 cameraPosition;
};

void main() {
  gl_Position = projectionMatrix * viewMatrix * modelMatrix * vec4(position, 1.0);
}
