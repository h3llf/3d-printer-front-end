struct Point {
	p : vec3<f32>,
};

struct SegmentRange {
	start_point : u32,
	end_point : u32,
	start_index : u32
};

struct Vertex {
	x : f32,
	y : f32,
	z : f32,
	dist : f32,
	nx : f32,
	ny : f32,
	nz : f32,
};

struct Params {
	point_count : u32,
	range_count : u32,
	line_width : f32
};

@group(0) @binding(0)
var<storage, read> points : array<Point>;

@group(0) @binding(1)
var<storage, read> segment_ranges : array<SegmentRange>;

@group(0) @binding(2)
var<storage, read_write> vertices : array<Vertex>;

@group(0) @binding(3)
var<storage, read_write> indices : array<u32>;

@group(0) @binding(4)
var<uniform> params : Params;

const up_dir = vec3<f32>(0.0, 1.0, 0.0);

//const workgroup_count : u32 = 10;
const workgroup_size : u32 = 64;

@compute
@workgroup_size(64)
fn main(
	@builtin(global_invocation_id) id : vec3<u32>,
	@builtin(num_workgroups) num_wg : vec3<u32>)
{
	let total_threads = num_wg.x * workgroup_size;

	for (var i = id.x; i < params.range_count; i += total_threads) {
		let range : SegmentRange = segment_ranges[i];
		for (var j = range.start_point; j < range.end_point; j += 1) {
			let p0 : Point = points[j];
			let p1 : Point = points[j + 1];

			let line_dir = normalize(p0.p - p1.p);
			var right = cross(line_dir, up_dir);

			if (length(right) < 0.0001) {
				right = vec3<f32>(1.0, 0.0, 0.0);
			}

			let vertex_base = j * 4;

			// Normal vectors
			vertices[vertex_base + 0u].nx = right.x;
			vertices[vertex_base + 0u].ny = right.y;
			vertices[vertex_base + 0u].nz = right.z;

			vertices[vertex_base + 1u].nx = -right.x;
			vertices[vertex_base + 1u].ny = right.y;
			vertices[vertex_base + 1u].nz = -right.z;

			vertices[vertex_base + 2u].nx = right.x;
			vertices[vertex_base + 2u].ny = right.y;
			vertices[vertex_base + 2u].nz = right.z;

			vertices[vertex_base + 3u].nx = -right.x;
			vertices[vertex_base + 3u].ny = right.y;
			vertices[vertex_base + 3u].nz = -right.z;

			right = normalize(right) * params.line_width;

			// Position
			vertices[vertex_base + 0u].x = p0.p.x + right.x;
			vertices[vertex_base + 0u].y = p0.p.y + right.y;
			vertices[vertex_base + 0u].z = p0.p.z + right.z;
			vertices[vertex_base + 0u].dist = 1;

			vertices[vertex_base + 1u].x = p0.p.x - right.x;
			vertices[vertex_base + 1u].y = p0.p.y - right.y;
			vertices[vertex_base + 1u].z = p0.p.z - right.z;
			vertices[vertex_base + 1u].dist = 0;

			vertices[vertex_base + 2u].x = p1.p.x + right.x;
			vertices[vertex_base + 2u].y = p1.p.y + right.y;
			vertices[vertex_base + 2u].z = p1.p.z + right.z;
			vertices[vertex_base + 2u].dist = 1;

			vertices[vertex_base + 3u].x = p1.p.x - right.x;
			vertices[vertex_base + 3u].y = p1.p.y - right.y;
			vertices[vertex_base + 3u].z = p1.p.z - right.z;
			vertices[vertex_base + 3u].dist = 0;

			let index_base = range.start_index + (j - range.start_point) * 6;
			indices[index_base + 0u] = vertex_base + 0u;
			indices[index_base + 1u] = vertex_base + 1u;
			indices[index_base + 2u] = vertex_base + 2u;
			indices[index_base + 3u] = vertex_base + 2u;
			indices[index_base + 4u] = vertex_base + 1u;
			indices[index_base + 5u] = vertex_base + 3u;		
		}
	}
}
