#version 450 core
out vec4 FragColor;

in vec3 v_uvs;

uniform samplerCube skybox;

void main()
{
    FragColor = texture(skybox, v_uvs);
}