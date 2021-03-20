#version 330 core

out vec4 FragColor;

in vec2 Coord;

uniform sampler2D Texture;

void main()
{
    FragColor = texture(Texture, Coord);
}
