#version 330 core

in vec3 fragNormal;
in vec3 fragVert;

out vec4 color;
uniform vec3 lightPosition;
uniform mat3 normalMatrix;
uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 modelViewMatrix;

const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec4 surfaceColor = vec4(1.0, 0.5, 0.0, 1.0);
const vec3 ambientColor = vec3(0.1, 0.1, 0.1);

vec3 ambientLighting();
vec3 diffuseLighting(in vec3 N, in vec3 L);
vec3 specularLighting(in vec3 N, in vec3 L, in vec3 V);

void main() {
  // All in WorldSpace
  vec4 worldPosition   = modelMatrix * vec4(fragVert, 1.0);
  vec3 normal          = normalize(normalMatrix*fragNormal);
  vec3 lightDirection  = normalize(lightPosition - worldPosition.xyz);
  vec3 cameraPosition  = (inverse(viewMatrix) * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
  vec3 cameraDirection = normalize(cameraPosition - worldPosition.xyz);

  color.xyz = specularLighting(normal, lightDirection, cameraDirection)
    + ambientLighting()
    + diffuseLighting(normal, lightDirection);
  color.a = surfaceColor.a;
}

vec3 specularLighting(in vec3 N, in vec3 L, in vec3 V) {
   vec3 H = normalize(L + V);
   float specularTerm = max(pow(dot(N, H), 32.0), 0.0);
   return surfaceColor.xyz*lightColor*specularTerm;
}

vec3 diffuseLighting(in vec3 N, in vec3 L) {
  float diffuse = max(dot(N, L), 0.0);
  return surfaceColor.xyz*lightColor*diffuse;
}

vec3 ambientLighting() {
  return ambientColor;
}
