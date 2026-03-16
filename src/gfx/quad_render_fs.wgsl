struct FSIn {
    @location(0) dist : f32,
    @location(1) normal : vec3<f32>
};

@fragment
fn fs_main(input : FSIn) -> @location(0) vec4<f32> {
	var colour : vec3<f32> = vec3<f32>(1.0, 0.1, 0.1);
	colour = colour * ((input.dist * 0.9) + 0.1);
	return vec4<f32>(colour, 1.0);
}
