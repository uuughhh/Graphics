#version 430 core

layout (location=1) in vec3 vertexColor;

layout (location=2) in vec3 vertexNormal;

out vec4 color;

void main()
{
    vec3 finalLight = vertexNormal * (-lightDirection);

}