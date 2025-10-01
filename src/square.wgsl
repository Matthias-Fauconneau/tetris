struct Vertex {
	@location(0) position: vec2<f32>,
	@location(1) color: vec3<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@vertex fn vertex(vertex: Vertex) -> VertexOutput {
	let position = vec4(vertex.position, 0., 1.);
	return VertexOutput(position, vertex.color);
}

@fragment fn fragment(vertex: VertexOutput) -> @location(0) vec4<f32> {
	return vec4(vertex.color, 1.);
}
