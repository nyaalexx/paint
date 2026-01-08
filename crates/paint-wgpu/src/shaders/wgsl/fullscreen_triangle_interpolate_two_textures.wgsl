struct Immediates {
    alpha: f32,
}

var<immediate> imm: Immediates;

@group(0) @binding(0)
var u_sampler: sampler;

@group(0) @binding(1)
var u_texture_a: texture_2d<f32>;

@group(0) @binding(2)
var u_texture_b: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const positions = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2( 3.0, -1.0),
        vec2(-1.0, 3.0),
    );

    const uvs = array<vec2<f32>, 3>(
        vec2(0.0,  1.0),
        vec2(2.0,  1.0),
        vec2(0.0, -1.0),
    );

    var output: VertexOutput;
    
    output.pos = vec4(positions[in_vertex_index], 0.0, 1.0);
    output.uv = uvs[in_vertex_index];

    return output;
}

@fragment
fn fragment(v: VertexOutput) -> @location(0) vec4<f32> {
    let a = textureSample(u_texture_a, u_sampler, v.uv);
    let b = textureSample(u_texture_b, u_sampler, v.uv);
    return mix(a, b, imm.alpha);
}
