#version 430 core

uniform layout(location = 0) mat4x4 transPos;

layout (location=0) in vec3 position;

layout (location=1) in vec4 vColor;

layout (location=2) in vec3 vNormal;

layout (location=1) out vec4 vertexColor;

layout (location=2) out vec3 vertexNormal;


void main()
{
    vec4 pos = vec4(position, 1.0f);
    gl_Position = transPos * pos ;
    vertexColor = vColor;
    vertexNormal = vNormal;
}