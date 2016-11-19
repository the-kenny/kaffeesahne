#version 330 core

in vec3 fragNormal;
in vec3 fragVert;

uniform vec3 lightPosition;
uniform vec3 cameraPosition;
uniform mat3 normalMatrix;
uniform mat4 modelMatrix;

uniform samplerCube depthMap;
uniform float farPlane;

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
float shadowStrength();

void main() {
  // All in WorldSpace
  vec4 worldPosition   = modelMatrix * vec4(fragVert, 1.0);
  vec3 normal          = normalize(normalMatrix*fragNormal);
  vec3 lightDirection  = normalize(lightPosition - worldPosition.xyz);
  vec3 cameraDirection = normalize(cameraPosition - worldPosition.xyz);
  float shadow = shadowStrength();                      
    
  // color.xyz = (specularLighting(normal, lightDirection, cameraDirection)
  //   + ambientLighting()
  //   + diffuseLighting(normal, lightDirection))
  //   * (1.0 - shadow);
  // color.a = 1.0;                // TODO

  color = vec4(texture(depthMap, fragVert).r);
}

vec3 specularLighting(in vec3 N, in vec3 L, in vec3 V) {
   vec3 H = normalize(L + V);
   float factor = max(pow(dot(N, H), shininess), 0.0);
   return specular.xyz*lightIntensity*factor;
}

vec3 diffuseLighting(in vec3 N, in vec3 L) {
  float factor = max(dot(N, L), 0.0);
  return diffuse.xyz*lightIntensity*factor;
}

vec3 ambientLighting() {
  return ambient.xyz*ambientIntensity;
}

float shadowStrength() {
  // Get vector between fragment position and light position
  vec3 fragToLight = fragVert - lightPosition;
  // Use the light to fragment vector to sample from the depth map    
  float closestDepth = texture(depthMap, fragToLight).r;
  // It is currently in linear range between [0,1]. Re-transform back to original value
  closestDepth *= farPlane;
  // Now get current linear depth as the length between the fragment and light position
  float currentDepth = length(fragToLight);
  // Now test for shadows
  float bias = 0.05; 
  float shadow = (currentDepth - bias > closestDepth) ? 1.0 : 0.0;

  return shadow;
}  
