// Bindings for the texture and sampler
@group(1) @binding(4)
var colormap_texture: texture_2d<f32>;

@group(1) @binding(5)
var colormap_sampler: sampler;

// Group the uniforms into a single block
struct MandelbrotMaterial {
    max_iterations: u32,
    color_scale: f32,
    offset: vec2<f32>,
    zoom: f32,
};
@group(2) @binding(0)
var<uniform> mandelbrotMaterial: MandelbrotMaterial;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let c = (input.world_position.xy / vec2<f32>(1920.0, 1080.0)) * mandelbrotMaterial.zoom - mandelbrotMaterial.offset;
    var z = vec2<f32>(0.0, 0.0);
    var n: f32 = 0.0;
    
    for(var i: u32 = 0u; i < mandelbrotMaterial.max_iterations; i = i + 1u) {
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        if (dot(z, z) > 4.0) { break; };
        n = n + 1.0;
    }
    
    let escape_value = n / f32(mandelbrotMaterial.max_iterations);

    // Sample a color from the colormap_texture using the escape value
    let color = textureSample(colormap_texture, colormap_sampler, vec2<f32>(escape_value, 0.5));

    if (n == f32(mandelbrotMaterial.max_iterations)) {
        let color = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black for points inside the Mandelbrot set
    }

    
    return color;
    //return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red color
}
