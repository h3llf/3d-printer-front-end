use std::io::{BufReader, prelude::*};
use std::fs::File;
use std::path::PathBuf;
use super::gfx::gcode_render::{Point, SegmentRange, GCodeRenderData};

#[derive(Debug)]
struct MoveCommand {
    x : f32,
    y : f32,
    z : f32,
    e : f32,
    f : f32
}

//#[derive(Debug, Clone)]
/*struct Point {
    position : [f32; 3],
}*/

#[derive(Default)]
pub struct GCodeParser {
    raw_file : Vec<u8>,
    pub render_data : GCodeRenderData,

    // State machine
    axis_absolute : bool,
    extruder_absolute : bool,
    current_pos : [f32; 3],
    extruder_pos : f32,
    end_line : bool,
    current_segment : SegmentRange,
    current_point : u32,
}

impl GCodeParser {
    pub fn default() -> Self {
        Self {
            raw_file : Vec::<u8>::new(),
            render_data : GCodeRenderData::default(),
            axis_absolute : false,
            extruder_absolute : false,
            current_pos : [0.0, 0.0, 0.0],
            extruder_pos : 0.0,
            end_line : false,
            current_segment : SegmentRange{start_vertex : 0, end_vertex : 0, start_index : 0},
            current_point : 0
        }
    }
    
    pub fn load_gcode(&mut self, dir : &PathBuf) {
        let mut file : std::io::Result<File> = File::open(dir);
        if file.is_err() {
            return;
        }

        self.render_data = GCodeRenderData::default();
        self.current_point = 0;

        let reader = BufReader::new(file.unwrap());
        let mut count : u32 = 0;
        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.split(';').next().unwrap();

            if line.is_empty() {
                continue;
            }

            let tokens : Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            match tokens[0] {
                "G1" => {
                    self.move_command(&tokens[1..]);
                } _=> {
                    continue;
                }
            }

            count += 1;
        }

        println!("Loaded {} vertices, {} indices, {} segments", 
            self.render_data.vertex_count, 
            self.render_data.index_count,
            self.render_data.segment_ranges.len());

        println!("Line count {}", count);
    }

    fn move_command(&mut self, tokens : &[&str]) {
        let mut move_cmd = MoveCommand {
            x : 0.0,
            y : 0.0,
            z : 0.0,
            e : 0.0,
            f : 0.0,
        };

        // TODO: Relative movements
        for token in tokens {
            let letter : char = token.chars().next().unwrap();
            let mut value : f32 = token[1..].parse().unwrap();
            value /= 10.0;

            match letter {
                'X' => {
                    move_cmd.x = value;
                    self.current_pos[0] = value;
                } 'Y' => {
                    move_cmd.y = value;
                    self.current_pos[2] = value;
                } 'Z' => {
                    move_cmd.z = value;
                    self.current_pos[1] = value;
                } 'E' => {
                    move_cmd.e = value;
                } 'F' => {
                    move_cmd.f = value;
                } _=>{}
            }
        }

        // TODO: Line range adjacency list
        // Vertices should remain the same but -6 indices per missing segment

        let point = Point{p : self.current_pos, _padding : 0.0};
        self.render_data.points.push(point.clone());
        self.render_data.vertex_count += 4;

        let end_line : bool;
        if move_cmd.e > 0.001 && self.extruder_pos - move_cmd.e < 0.001 &&
            move_cmd.x > 0.001 && move_cmd.y > 0.001
//        if move_cmd.e > 0.001 &&
//            move_cmd.x > 0.001 && move_cmd.y > 0.001
        {
            end_line = false;
            self.render_data.index_count += 6;
        } else {
            //TODO: Will break if called twice in a row

            // End segment
            end_line = true;
            self.current_segment.end_vertex = self.current_point;
            self.render_data.segment_ranges.push(self.current_segment);

            self.current_segment = SegmentRange::default();
            self.current_segment.start_vertex = self.current_point;
            self.current_segment.start_index = self.render_data.index_count;
        }

        self.end_line = end_line;
        self.current_point += 1;

        
        if move_cmd.e > 0.001 {
            self.extruder_pos = move_cmd.e;
        }
    }

}
