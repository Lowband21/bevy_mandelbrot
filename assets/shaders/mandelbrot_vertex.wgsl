// Existing bindings for zoom and offset
@group(1) @binding(2) var<uniform> zoom: f32;
@group(1) @binding(3) var<uniform> offset: vec2<f32>;

// Define the input and output structs
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    // Remove blend_color if you're not using it
};

// Vertex shader output structure aligned with the fragment shader inputs
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Translate vertex position so center becomes the origin
    let translated_position = vertex.position.xy - vec2<f32>(0.5, 0.5);

    // Apply zoom to vertex position
    let zoomed_position = translated_position * zoom;

    // Apply offset
    let offset_position = zoomed_position + offset;

    // Re-translate back and update z-coordinate
    let final_position = vec3<f32>(offset_position.x + 0.5, offset_position.y + 0.5, vertex.position.z);

    // Set clip_position and world_position
    out.clip_position = vec4<f32>(final_position, 1.0);
    out.world_position = vec4<f32>(final_position, 1.0); 

    // Other unchanged code
    out.normals = vec3<f32>(0.0, 0.0, 1.0);
    out.uv = vertex.position.xy;

    return out;
}
