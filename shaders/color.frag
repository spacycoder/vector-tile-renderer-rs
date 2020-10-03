#version 450 core

out vec4 FragColor;
in vec4 gl_FragCoord;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_color;
void main()
{   
    FragColor = u_color;
}
