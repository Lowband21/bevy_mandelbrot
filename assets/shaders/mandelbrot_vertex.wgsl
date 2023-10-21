// Existing bindings for zoom and offset
@group(1) @binding(2) var<uniform> zoom: f32;
@group(1) @binding(3) var<uniform> offset: vec2<f32>;

@group(1) @binding(7)
var<uniform> chunk_offset: vec2<f32>;
@group(1) @binding(6)
var<uniform> chunk_zoom: f32;

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

    // Apply only chunk_zoom and chunk_offset to vertex position.
    let zoomed_position: vec3<f32> = vertex.position * vec3<f32>(chunk_zoom, chunk_zoom, 1.0); // Using chunk_zoom
    let chunk_offset_position: vec3<f32> = zoomed_position + vec3<f32>(chunk_offset.x, chunk_offset.y, 0.0); // Using chunk_offset

    // Set clip_position and world_position based on chunk_zoom and chunk_offset only.
    out.clip_position = vec4<f32>(chunk_offset_position, 1.0);
    out.world_position = vec4<f32>(chunk_offset_position, 1.0);

    // Assuming normals are facing positively along the z-axis (No change here).
    out.normals = vec3<f32>(0.0, 0.0, 1.0);

    // Modify the UVs to include only chunk_offset and chunk_zoom
    // Assuming the mesh has normalized coordinates, adjust them for this chunk.
    let chunk_zoomed_uv: vec2<f32> = vertex.position.xy * vec2<f32>(chunk_zoom, chunk_zoom); // Using chunk_zoom
    out.uv = chunk_zoomed_uv + chunk_offset; // Using chunk_offset

    return out;
}
