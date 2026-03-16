struct Camera {
	view_proj : mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera : Camera;

struct VSIn {
	@location(0) position : vec3<f32>,
	@location(1) dist : f32,
	@location(2) normal : vec3<f32>,
};

struct VSOut {
	@builtin(position) position : vec4<f32>,
	@location(0) dist : f32,
	@location(1) normal : vec3<f32>
};

@vertex
fn vs_main(input: VSIn) -> VSOut {
	var out : VSOut;

	let world_pos = vec4<f32>(input.position, 1.0);
	out.position = camera.view_proj * world_pos;

	out.dist = input.dist;
	out.normal = input.normal;

	return out;
}
