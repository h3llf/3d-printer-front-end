struct FSIn {
    @location(0) dist : f32,
    @location(1) normal : vec3<f32>,
    @location(2) view_dir : vec3<f32>,
};

@fragment
fn fs_main(input : FSIn) -> @location(0) vec4<f32> {
	let light_dir : vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
	let half_way_dir : vec3<f32> = normalize(light_dir + input.view_dir);

	var colour : vec3<f32> = vec3<f32>(0.5, 0.1, 1.0);

	let diff : f32 = max(dot(input.normal, light_dir), 0.0);
	let diffuse : vec3<f32> = colour * diff;

	return vec4<f32>(colour * 0.1 + diffuse, 1.0);
}
