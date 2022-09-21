#version 430 core

uniform layout(location = 0) mat4x4 transPos;

in layout (location=0) vec3 position;

in layout (location=1) vec4 vColor;

layout (location=1) out vec4 vertexColor;

void main()
{
    mat4x4 identityMatrix =  transPos * mat4(1) ;
    vec4 pos = vec4(position, 1.0f);
    gl_Position = identityMatrix * pos;
    vertexColor = vColor;
}