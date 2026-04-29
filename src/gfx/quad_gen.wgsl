struct Point {
	p : vec3<f32>,
};

struct SegmentRange {
	start_point : u32,
	end_point : u32,
	start_index : u32
};

struct Vertex {
	pos : vec3<f32>,
	dist : f32,
	forward : vec3<f32>,
	miter_scale : f32,
	fwd2 : vec3<f32>
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
		if range.end_point <= range.start_point {
			continue;
		}

		for (var j = range.start_point; j < range.end_point - 1; j += 1) {
			let p1 : Point = points[j];
			let p2 : Point = points[j + 1];

			let ld1 = normalize(p2.p - p1.p);

			var p0 : Point;
			if (j == 0u) {
				p0.p = points[j].p + ld1;
			} else {
				p0 = points[j - 1];
			}

			var p3 : Point;
			if (j == range.end_point - 2) {
				p3.p = points[j + 1].p - ld1;
			} else {
				p3 = points[j + 2];
			}

			let ld0 = normalize(p1.p - p0.p);
			let ld2 = normalize(p3.p - p2.p);

			let vertex_base = j * 4;
			var tan0 = ld1;
			if (length(ld0 + ld1) > 1e-4) {
				tan0 = normalize(ld0 + ld1);
			}
			var tan1 = ld1;
			if (length(ld2 + ld1) > 1e-4) {
				tan1 = normalize(ld2 + ld1);
			}

			// Directions
			let forward = ld1;
			vertices[vertex_base + 0u].fwd2 = ld0;
//			vertices[vertex_base + 0u].fwd2 = tan0;
			vertices[vertex_base + 0u].forward = forward;
			vertices[vertex_base + 0u].miter_scale = 1.0;
			vertices[vertex_base + 1u].fwd2 = ld0;
//			vertices[vertex_base + 1u].fwd2 = tan0;
			vertices[vertex_base + 1u].forward = forward;
			vertices[vertex_base + 1u].miter_scale = 1.0;
			vertices[vertex_base + 2u].fwd2 = ld2;
//			vertices[vertex_base + 2u].fwd2 = tan1;
			vertices[vertex_base + 2u].forward = forward;
			vertices[vertex_base + 2u].miter_scale = -1.0;
			vertices[vertex_base + 3u].fwd2 = ld2;
//			vertices[vertex_base + 3u].fwd2 = tan1;
			vertices[vertex_base + 3u].forward = forward;
			vertices[vertex_base + 3u].miter_scale = -1.0;

			// Position
			vertices[vertex_base + 0u].pos = p1.p;
			vertices[vertex_base + 0u].dist = 1;
			vertices[vertex_base + 1u].pos = p1.p;
			vertices[vertex_base + 1u].dist = 0;
			vertices[vertex_base + 2u].pos = p2.p;
			vertices[vertex_base + 2u].dist = 1;
			vertices[vertex_base + 3u].pos = p2.p;
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
