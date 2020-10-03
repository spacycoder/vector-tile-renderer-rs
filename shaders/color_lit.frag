#version 450 core

out vec4 FragColor;

in vec4 gl_FragCoord;
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_position;

uniform float u_time;
uniform vec2 u_resolution;
uniform sampler2D texture1;
uniform vec3 u_viewPos;
uniform vec4 u_color;

void main()
{   
    float specularStrength = 0.5;
    float ambientStrength = 0.1;
    vec3 color = u_color.rgb;
    vec3 lightColor = vec3(1.0, 1.0, 1.0);

    vec3 ambient = ambientStrength * lightColor;

    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    float diff = max(0.0, dot(v_normal, -lightDirection));
    vec3 diffuse = diff * lightColor;

    vec3 viewDir = normalize(u_viewPos - v_position);
    vec3 reflectDir = reflect(-lightDirection, v_normal);  

    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
    vec3 specular = specularStrength * spec * lightColor;

    color = (diffuse + ambient + specular) * color;
    FragColor = vec4(color, 1.0);
}
