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
	@location(2) forward_1 : vec3<f32>,
	@location(3) miter_scale : f32,
	@location(4) forward_2 : vec3<f32>,
};

struct VSOut {
	@builtin(position) position : vec4<f32>,
	@location(0) dist : f32,
	@location(1) normal : vec3<f32>,
    	@location(2) view_dir : vec3<f32>,
};

// TODO: Look at expanding quad lines in screen space to avoid singularities with miters

@vertex
fn vs_main(input: VSIn) -> VSOut {
	var out : VSOut;

	let width : f32 = 0.03; // TODO: Pass this in a uniform

	let view_dir : vec3<f32> = normalize(camera.position - input.position);
	var side : vec3<f32> = normalize(cross(input.forward_1, view_dir));
	var side2 : vec3<f32> = normalize(cross(input.forward_2, view_dir));
	var tangent : vec3<f32> = normalize(input.forward_1 + input.forward_2);

	// Flip miter direction when joint facing away from camera
	if (dot(side, side2) < 0.0) {
		side2 = -side2;
	}

	// Miter vector
	var miter_vec = side;

	// dist == 0 or 1, determines if this vertex should be expanded right or left
	var offset : f32 = input.dist * 2.0 - 1.0;

	let denom = dot(input.forward_1, side2);
	var miter_length = 0.0;
	if (abs(denom) > 0.0001) {
		miter_length = ((width * 0.5 * (1.0 - dot(side, side2))) / denom) * offset;

		let miter_limit = 2.0 * width;
		miter_length = clamp(miter_length, -miter_limit, miter_limit);
	}

	var world_pos = input.position + offset * miter_vec * width * 0.5 + input.forward_1 * miter_length;
	out.position = camera.view_proj * vec4<f32>(world_pos, 1.0);

	out.dist = input.dist;
	out.normal = side * offset;
	out.view_dir = view_dir;

	return out;
}
