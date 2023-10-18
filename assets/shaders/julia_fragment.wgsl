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
var<uniform> c: vec2<f32>;

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>
) -> @location(0) vec4<f32> {
    var z: vec2<f32> = uv * 4.0 - 2.0; // z starts as the current pixel
    var iteration: f32 = 0.0;

    while (iteration < max_iterations) {
        let x_squared: f32 = z.x * z.x;
        let y_squared: f32 = z.y * z.y;
        let two_xy: f32 = 2.0 * z.x * z.y;
        
        if (x_squared + y_squared > 4.0) {
            break;
        }

        z.x = x_squared - y_squared + c.x;
        z.y = two_xy + c.y;
        
        iteration = iteration + 1.0;
    }

    let basic_color = iteration / max_iterations;
    let color = pow(basic_color, 0.1) * (1.0 - color_scale) + color_scale;

    let colormap_color: vec4<f32> = textureSample(colormap_texture, colormap_sampler, vec2<f32>(color, 0.5));
    return colormap_color;
}
