struct Vertex {
	@location(0) position: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
}

@vertex fn vertex(vertex: Vertex) -> VertexOutput {
	let position = vec4(vertex.position, 0., 1.);
	return VertexOutput(position);
}

@fragment fn fragment(vertex: VertexOutput) -> @location(0) vec4<f32> {
	return vec4(1., 1., 1., 1.);
}
