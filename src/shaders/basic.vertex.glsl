#version 330 core

in vec3 position;
in vec2 uv;
in vec3 normal;

out vec3 fragVert;
out vec3 fragNormal;
out vec2 fragUv;

layout(std140)
uniform Uniforms {
  uint pickingId;

  mat4 modelMatrix;
  mat4 normalMatrix;            // might need layout(row_major)
  mat4 viewMatrix;
  mat4 projectionMatrix;

  vec3 lightPosition;
  vec3 cameraPosition;
};

void main() {
  fragNormal = normal;
  fragVert = position;
  fragUv = uv;

  mat4 modelViewProject = projectionMatrix * viewMatrix * modelMatrix;
  gl_Position = modelViewProject * vec4(position, 1.0);
}
