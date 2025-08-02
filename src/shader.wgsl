struct Uniforms {
    time: f32,
};

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) v_start: vec2<f32>,
    @location(2) color: vec3<f32>,  // Add color if you want per-vertex or per-instance colors
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,  // Pass color to fragment shader
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

var<private> g: vec2<f32> = vec2<f32>(0.0, -0.2);

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let t = uniforms.time;
    let decay = 0.01;

    let new_x = clamp(input.position.x + input.v_start.x * t - t * decay, -1.0, 1.0);
    let new_y = input.position.y + input.v_start.y * t - t * t;

    var output: VertexOutput;
    output.position = vec4<f32>(new_x, new_y, 0.0, 1.0);
    output.color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}

