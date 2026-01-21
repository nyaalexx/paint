struct VertexInput {
    @location(0) pos_ndc: vec2<f32>,
    @location(1) corner_dist: f32,
}

struct VertexOutput {
    @builtin(position) pos_ndc: vec4<f32>,
    @location(0) corner_dist: f32,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.pos_ndc = vec4(in.pos_ndc, 0.0, 1.0);
    output.corner_dist = in.corner_dist;
    return output;
}

@fragment
fn fragment(v: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = 1.0 - smoothstep(0.5, 2.0, v.corner_dist);
    return vec4(1.0, 1.0, 1.0, alpha);
}
