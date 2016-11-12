#version 330 core

uniform uint pickingId;
out uint color;

void main() {
  color = pickingId;
}
