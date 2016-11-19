#version 330 core

in vec3 position;
// out vec4 fragVert;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;

void main() {
  gl_Position = viewMatrix * modelMatrix * vec4(position, 1.0);
  // fragVert = gl_Position;
}  
