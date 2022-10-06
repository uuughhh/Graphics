#version 430 core

layout (location=1) in vec4 vertexColor;

layout (location=2) in vec3 vertexNormal;

out vec4 color;

vec3 maxVec3 (vec3 vector1, vec3 vector2) {
    vec3 newVec = vec3 (max (vector1.x, vector2.x),max (vector1.y, vector2.y),max (vector1.z, vector2.z));
    return newVec;
}

void main()
{
    vec3 lightDirection = normalize(vec3(0.8f, -0.5f, 0.6f));

    vec3 finalLight = vertexNormal * (-lightDirection);

    vec3 zero = vec3 (0.0f,0.0f,0.0f);

    color = vec4 (vertexColor.rgb * maxVec3(finalLight,zero), vertexColor.a);
    // color = vec4 (vertexColor.rgb * vertexNormal,vertexColor.a);
    

}