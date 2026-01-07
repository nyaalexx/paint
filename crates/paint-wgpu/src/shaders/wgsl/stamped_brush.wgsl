struct Immediates {
    transform: mat2x2<f32>,
    translation: vec2<f32>,
}

var<immediate> imm: Immediates;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) radius: f32,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) rel_pos: vec2<f32>,
    @location(1) radius: f32,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    const vertices = array<vec2<f32>, 6>(
        vec2(-1.0,  1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),

        vec2( 1.0,  1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0, -1.0)
    );

    var output: VertexOutput;
    
    let padded_radius = in.radius + 1.0;
    let vertex = vertices[in.vertex_index];
    let pos = imm.transform * (vertex * padded_radius + in.pos) + imm.translation;

    output.pos = vec4(pos, 0.0, 1.0);
    output.rel_pos = vertex * padded_radius ;
    output.radius = in.radius;

    return output;
}

@fragment
fn fragment(v: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(v.rel_pos) - 0.5 * v.radius;
    let alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
    return vec4(1.0, 1.0, 1.0, alpha);
}
