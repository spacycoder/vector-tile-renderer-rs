#version 450 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec2 uv;

out vec4 v_color;
out vec2 v_uv;
out vec4 v_objVertex;

uniform mat4 transform;
uniform mat4 modelTransform;
uniform mat4 viewTransform;
uniform mat4 projectionTransform;
uniform float u_time;

void main()
{
    v_color = color;
    v_uv = uv;
    vec4 pos = vec4(position.xy, sin(position.x + u_time) / 2.0, 1.0f);
    v_objVertex = pos;
    gl_Position =  transform * pos;
}