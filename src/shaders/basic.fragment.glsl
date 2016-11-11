#version 330 core

in vec3 fragNormal;
in vec3 fragVert;

out vec4 color;
uniform vec3 lightPosition;
uniform mat3 normalMatrix;
uniform mat4 modelMatrix;

const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec4 surfaceColor = vec4(1.0, 0.5, 0.0, 1.0);
const vec3 ambientColor = vec3(0.1, 0.1, 0.1);

void main() {
  vec3 normal         = normalize(normalMatrix * fragNormal);
  vec3 fragPosition   = vec3(modelMatrix * vec4(fragVert, 0.0));
  vec3 surfaceToLight = lightPosition - fragPosition;

  float brightness = dot(normal, surfaceToLight) / (length(surfaceToLight) * length(normal));
  brightness = clamp(brightness, 0, 1);

  color = vec4(brightness * lightColor * surfaceColor.rgb + ambientColor, surfaceColor.a);
}
