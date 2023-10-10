struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> transform: mat4x4<f32>; // a uniform matrix for transformations (this could be a ModelViewProjection matrix)

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = transform * vec4<f32>(input.position, 1.0);
    output.world_position = output.clip_position; // for a simple shader this might be the same, but often it could be different if you had lighting calculations.
    output.world_normal = vec3<f32>(0.0, 0.0, 1.0); // for a 2D sprite the normal might just point out of the screen
    output.uv = input.uv;

    return output;
}
