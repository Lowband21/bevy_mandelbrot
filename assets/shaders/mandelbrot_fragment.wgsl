// Bindings for the texture and sampler
@group(1) @binding(4)
var colormap_texture: texture_2d<f32>;

@group(1) @binding(5)
var colormap_sampler: sampler;

struct MandelbrotMaterial {
    color_scale: f32,
    max_iterations: u32,
    zoom: f32,
    offset: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> mandelbrotMaterial: MandelbrotMaterial;

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>
) -> @location(0) vec4<f32> {
    var c: vec2<f32> = uv * 4.0 - 2.0;
    var z: vec2<f32> = vec2<f32>(0.0, 0.0);
    var iteration: f32 = 0.0;
    let max_iterations: f32 = 1000.0;

    // Check for early exit
    let q: f32 = (c.x - 0.25) * (c.x - 0.25) + c.y * c.y;
    if (q * (q + (c.x - 0.25)) < 0.25 * c.y * c.y || (c.x + 1.0) * (c.x + 1.0) + c.y * c.y < 0.0625) {
        iteration = max_iterations;
    } else {
        while (iteration < max_iterations) {
            let x: f32 = (z.x * z.x - z.y * z.y) + c.x;
            let y: f32 = (2.0 * z.x * z.y) + c.y;
            if (abs(x) > 2.0 || abs(y) > 2.0) {
                // Modify iteration based on the distance from the origin
                iteration += (1.0 - length(z) / 2.0) * 20.0; // 20.0 is an arbitrary factor to control gradient strength
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
    let color = adjusted_color * (1.0 - mandelbrotMaterial.color_scale) + mandelbrotMaterial.color_scale;

    // Sample from the colormap texture
    let colormap_color: vec4<f32> = textureSample(colormap_texture, colormap_sampler, vec2<f32>(color, 0.5));
    return colormap_color;
}
