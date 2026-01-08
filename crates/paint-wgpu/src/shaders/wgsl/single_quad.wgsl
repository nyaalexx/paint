struct Immediates {
    transform: mat2x2<f32>,
    translation: vec2<f32>,
}

var<immediate> imm: Immediates;

@group(0) @binding(0)
var u_sampler: sampler;

@group(0) @binding(1)
var u_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const vertices = array<vec2<f32>, 6>(
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0),

        vec2(1.0, 1.0),
        vec2(1.0, 0.0),
        vec2(0.0, 0.0)
    );

    var output: VertexOutput;
    
    let vertex = vertices[in_vertex_index];

    output.pos = vec4(imm.transform * vertex + imm.translation, 0.0, 1.0);
    output.uv = vertex;

    return output;
}

@fragment
fn fragment(v: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, v.uv);
}
