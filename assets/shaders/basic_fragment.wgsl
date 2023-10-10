@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var sampler: sampler;

@fragment
fn fragment() -> [[location(0)]] vec4 {
    // Just sample the texture and return its color
    return vec4(1.0, 0.0, 0.0, 1.0); // pure red color
}
