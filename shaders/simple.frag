#version 430 core

uniform layout(location = 1) mat4x4 trans;

layout (location=1) in vec4 vertexColor;

layout (location=2) in vec3 vertexNormal;

out vec4 color;

void main()
{
    mat3x3 transNorm = mat3(trans);

    vec3 newNormal = normalize( transNorm * vertexNormal);

    vec3 lightDirection = normalize(vec3(0.8f, -0.5f, 0.6f));

    float finalLight = dot(newNormal, (-lightDirection));

    color =  vec4 (vertexColor.rgb * max(finalLight,0), vertexColor.a) ;

}