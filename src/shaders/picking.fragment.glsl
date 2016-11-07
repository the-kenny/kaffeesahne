#version 330 core

uniform uint picking_id;
out uint color;

void main() {
  color = picking_id;
}
