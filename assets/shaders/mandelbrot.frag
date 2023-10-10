#version 450

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

layout(set = 2, binding = 0) uniform MandelbrotUniforms {
    vec2 offset;
    float zoom;
    float max_iterations;
};

void main() {
    vec2 c = (gl_FragCoord.xy / vec2(1920, 1080)) * zoom - offset;  // Adjust resolution as needed
    vec2 z = vec2(0.0, 0.0);
    float n = 0.0;
    for(int i = 0; i < int(max_iterations); i++) {
        z = vec2(z.x*z.x - z.y*z.y, 2.0*z.x*z.y) + c;
        if (dot(z, z) > 4.0) break;
        n++;
    }
    if (n == max_iterations) discard; 
    float brightness = smoothstep(0.0, 1.0, n/max_iterations);
    o_Target = vec4(vec3(brightness), 1.0);
}
