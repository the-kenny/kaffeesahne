#version 330 core

in vec3 fragNormal;
in vec3 fragVert;
in vec2 fragUv;

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

uniform bool hasDiffuseTexture;
uniform sampler2D diffuseTexture;

uniform Material {
  vec4 ambient;
  vec4 diffuse;
  vec4 specular;
  float shininess;
};

const float lightIntensity = 1.0;
const float ambientIntensity = 0.1;

out vec4 color;

vec3 ambientLighting();
vec3 diffuseLighting(in vec3 N, in vec3 L);
vec3 specularLighting(in vec3 N, in vec3 L, in vec3 V);

void main() {
  // All in WorldSpace
  vec4 worldPosition   = modelMatrix * vec4(fragVert, 1.0);
  vec3 normal          = normalize(mat3(normalMatrix)*fragNormal);
  vec3 lightDirection  = normalize(lightPosition - worldPosition.xyz);
  vec3 cameraDirection = normalize(cameraPosition - worldPosition.xyz);

  color.xyz = specularLighting(normal, lightDirection, cameraDirection)
    + ambientLighting()
    + diffuseLighting(normal, lightDirection);
  color.a = 1.0;                // TODO
}
vec3 specularLighting(in vec3 N, in vec3 L, in vec3 V) {
  vec3 H = normalize(L + V);
  float factor = max(pow(dot(N, H), shininess), 0.0);
  return specular.xyz*lightIntensity*factor;
}

vec3 diffuseLighting(in vec3 N, in vec3 L) {
  vec4 diffuse = int(hasDiffuseTexture) * texture(diffuseTexture, fragUv)
    + int(!hasDiffuseTexture) * diffuse;

  float factor = max(dot(N, L), 0.0);
  return diffuse.xyz*lightIntensity*factor;
}

vec3 ambientLighting() {
  return ambient.xyz*ambientIntensity;
}
