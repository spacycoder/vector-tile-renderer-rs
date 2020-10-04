#version 450 core
layout (location = 0) in vec3 position;

out vec3 v_uvs;

uniform mat4 projection;
uniform mat4 view_transform;

void main()
{
    v_uvs = position;
    vec4 pos = projection * view_transform * vec4(position, 1.0);
    gl_Position = pos.xyww;
}