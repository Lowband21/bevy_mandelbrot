// Bindings for the texture and sampler
@group(1) @binding(4)
var colormap_texture: texture_2d<f32>;

@group(1) @binding(5)
var colormap_sampler: sampler;

@group(1) @binding(0)
var<uniform> color_scale: f32;

@group(1) @binding(1)
var<uniform> max_iterations: f32;

@group(1) @binding(7)
var<uniform> chunk_offset: vec2<f32>;

@group(1) @binding(8)
var<uniform> chunk_id: vec2<f32>;

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>
) -> @location(0) vec4<f32> {

    // Use the transformed uv directly
    var c: vec2<f32> = uv;
    var z: vec2<f32> = vec2<f32>(0.0, 0.0);
    var iteration: f32 = 0.0;


    // Check for early exit
    let q: f32 = (c.x - 0.25) * (c.x - 0.25) + c.y * c.y;
    if (q * (q + (c.x - 0.25)) < 0.25 * c.y * c.y || (c.x + 1.0) * (c.x + 1.0) + c.y * c.y < 0.0625) {
        iteration = max_iterations;
    } else {
        while (iteration < max_iterations) {
            let x: f32 = (z.x * z.x - z.y * z.y) + c.x;
            let y: f32 = (2.0 * z.x * z.y) + c.y;
            if (abs(x) > 2.0 || abs(y) > 2.0) {
                break;
            }
            z.x = x;
            z.y = y;
            iteration = iteration + 1.0;
        }
    }

    // Convert iteration count to color
    let basic_color: f32 = f32(iteration) / f32(max_iterations);
    let adjusted_color = pow(basic_color, 0.3);
    let color = adjusted_color * (1.0 - color_scale) + color_scale;

    // Sample from the colormap texture
    let colormap_color: vec4<f32> = textureSample(colormap_texture, colormap_sampler, vec2<f32>(color, 0.5));

    // Color modulation based on chunk_offset (no changes here)
    let modulated_color: vec4<f32> = vec4<f32>(
        fract(chunk_offset.x * 0.1),
        fract(chunk_offset.y * 0.1),
        0.0,
        1.0
    );

    // Unique color modulation based on chunk_id
    let unique_chunk_color: vec4<f32> = vec4<f32>(
        fract(chunk_id.x * 0.3), // Use chunk_id.x to modulate the Red component uniquely for this chunk
        fract(chunk_id.y * 0.3), // Use chunk_id.y to modulate the Green component uniquely for this chunk
        0.0,  // Blue component (unchanged, but you could use this for further distinction)
        1.0   // Alpha (unchanged)
    );

    // Combine colormap_color, modulated_color and unique_chunk_color to form the final color
    let combined_color = mix(mix(colormap_color, modulated_color, 0.5), unique_chunk_color, 0.9);

    return combined_color;  // Return the final, uniquely modulated color
}
