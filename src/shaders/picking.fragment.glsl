#version 330 core

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

out uint color;

void main() {
  color = pickingId;
}
