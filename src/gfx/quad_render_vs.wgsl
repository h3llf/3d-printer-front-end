//TODO: Add y range min-max -> discard for layer view

struct Camera {
	view_proj : mat4x4<f32>,
	position : vec3<f32>,
};

@group(0) @binding(0)
var<uniform> camera : Camera;

struct VSIn {
	@location(0) position : vec3<f32>,
	@location(1) dist : f32,
	@location(2) forward : vec3<f32>,
};

struct VSOut {
	@builtin(position) position : vec4<f32>,
	@location(0) dist : f32,
	@location(1) forward : vec3<f32>
};

@vertex
fn vs_main(input: VSIn) -> VSOut {
	var out : VSOut;

	let width : f32 = 0.04; // TODO: Add this to a uniform

	let forward = input.forward;
	let view_dir : vec3<f32> = normalize(camera.position - input.position);
	var side : vec3<f32> = normalize(cross(forward, view_dir));
//	if length(side) < 0.001 {
//		side = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
//	}

	let offset : f32 = input.dist * 2.0 - 1.0;
	var world_pos = input.position + side * offset * width * 0.5;
	out.position = camera.view_proj * vec4<f32>(world_pos, 1.0);

	out.dist = input.dist;
	out.forward = side;

	return out;
}
