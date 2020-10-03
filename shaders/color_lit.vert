#version 450 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec2 uv;
layout (location = 3) in vec3 normal;

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;
out vec3 v_position;

uniform mat4 transform;
uniform mat4 modelTransform;
uniform mat4 viewTransform;
uniform mat4 projectionTransform;

void main()
{
    v_color = color;
    v_uv = uv;
    v_position = (modelTransform * vec4(position, 1.0f)).xyz;
    v_normal = normalize(mat3(modelTransform) * normal);
    gl_Position =  transform * vec4(position, 1.0f);
}