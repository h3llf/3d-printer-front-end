struct LineSegment {
	p0 : vec3<f32>,
	p1 : vec3<f32>,
};

struct Vertex {
	x : f32,
	y : f32,
	z : f32
};

struct Params {
	segment_count : u32
};

@group(0) @binding(0)
var<storage, read> line_segments : array<LineSegment>;

@group(0) @binding(1)
var<storage, read_write> vertices : array<Vertex>;

@group(0) @binding(2)
var<storage, read_write> indices : array<u32>;

@group(0) @binding(3)
var<uniform> params : Params;

const up_dir = vec3<f32>(0.0, 1.0, 0.0);

//const workgroup_count : u32 = 10;
const workgroup_size : u32 = 64;

const LINE_WIDTH : f32 = 0.02;

@compute
@workgroup_size(64)
fn main(
	@builtin(global_invocation_id) id : vec3<u32>,
	@builtin(num_workgroups) num_wg : vec3<u32>)
{
	let total_threads = num_wg.x * workgroup_size;

	for (var i = id.x; i < params.segment_count; i += total_threads) {
		let line : LineSegment = line_segments[i];
		let line_dir = normalize(line.p0 - line.p1);
		var right = cross(line_dir, up_dir);

		if (length(right) < 0.0001) {
			right = vec3<f32>(1.0, 0.0, 0.0);
		}

		right = normalize(right) * LINE_WIDTH;

		let vertex_base = i * 4;
		vertices[vertex_base + 0u].x = line.p0.x + right.x;
		vertices[vertex_base + 0u].y = line.p0.y + right.y;
		vertices[vertex_base + 0u].z = line.p0.z + right.z;

		vertices[vertex_base + 1u].x = line.p0.x - right.x;
		vertices[vertex_base + 1u].y = line.p0.y - right.y;
		vertices[vertex_base + 1u].z = line.p0.z - right.z;

		vertices[vertex_base + 2u].x = line.p1.x + right.x;
		vertices[vertex_base + 2u].y = line.p1.y + right.y;
		vertices[vertex_base + 2u].z = line.p1.z + right.z;

		vertices[vertex_base + 3u].x = line.p1.x - right.x;
		vertices[vertex_base + 3u].y = line.p1.y - right.y;
		vertices[vertex_base + 3u].z = line.p1.z - right.z;


		let index_base = i * 6;
		indices[index_base + 0u] = vertex_base + 0u;
		indices[index_base + 1u] = vertex_base + 1u;
		indices[index_base + 2u] = vertex_base + 2u;
		indices[index_base + 3u] = vertex_base + 2u;
		indices[index_base + 4u] = vertex_base + 1u;
		indices[index_base + 5u] = vertex_base + 3u;
	}
}
