#version 450 core

out vec4 FragColor;
in vec4 gl_FragCoord;
uniform float u_time;
uniform vec2 u_resolution;

varying in vec4 v_color;
varying in vec4 v_uv;
varying in vec4 v_objVertex;

void main()
{   
    vec4 color1 = vec4(0.0, 0.23, 0.38, 1.0);
    vec4 color2 = vec4(0.19, 0.69, 0.80, 1.0);
    float height = (v_objVertex.z + 0.5);
    vec4 color = mix(color1, color2, height); 
    FragColor = color;
}
