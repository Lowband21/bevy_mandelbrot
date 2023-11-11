// Bindings for the texture and sampler
@group(1) @binding(4)
var colormap_texture: texture_2d<f32>;

@group(1) @binding(5)
var colormap_sampler: sampler;

@group(1) @binding(0)
var<uniform> color_scale: f32;

@group(1) @binding(1)
var<uniform> max_iterations: f32;

@group(1) @binding(2)
var<uniform> zoom: f32;

@group(1) @binding(3)
var<uniform> offset: vec2<f32>;

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>
) -> @location(0) vec4<f32> {
    // Pre-calculate constants and perform calculations outside of the loop
    let scale = vec2<f32>(3.0, 2.0);
    let adjusted_uv = (uv * 2.0 - 1.0) * scale;
    var c: vec2<f32> = (adjusted_uv + offset) / zoom;

    var z: vec2<f32> = vec2<f32>(0.0, 0.0);
    var iteration: f32 = 0.0;
    var zLengthSquared: f32 = 0.0;

    while (iteration < max_iterations && zLengthSquared <= 4.0) {
        z = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y
        );
        zLengthSquared = dot(z, z);
        iteration = iteration + 1.0;
    }

    // Optimize color calculation
    let color = pow(f32(iteration) / f32(max_iterations), 0.3)
                * (1.0 - color_scale) + color_scale;

    // Sample color from the colormap
    return textureSample(colormap_texture, colormap_sampler, vec2<f32>(color, 0.5));
}
