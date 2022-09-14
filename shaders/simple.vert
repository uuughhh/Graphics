#version 430 core

in vec3 position;

in vec3 vColor;

out vec3 vertexColor;

void main()
{
    gl_Position = vec4(position, 1.0f);
    vertexColor = vColor;
}