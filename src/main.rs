mod map;
mod vision;
use std::thread;

use crate::map::map::Maze;
use crate::vision::vision::Vision;

fn main() {
    let camera_index = 0;
    let model_path = "bestsmall.onnx";
    let classes_labels: Vec<&str> = vec!["GREEN", "H", "RED", "S", "U", "YELLOW"];
    let net_width = 480;
    let net_height = 384;
    let class_filters: Vec<usize> = vec![];
    if let Ok(mut vis) = Vision::new(
        camera_index,
        model_path,
        classes_labels,
        net_width,
        net_height,
        class_filters,
    ) {
        thread::spawn(move || {
            if let Ok(()) = vis.run(0.6, 0.7) {
                println!("Done!");
            } else {
                println!("Error running object detection!");
            }
        });
    } else {
        println!("Error creating object!");
    }

    Maze::test_mapping();
}
