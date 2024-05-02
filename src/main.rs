mod map;
mod sensors;
mod vision;
use std::thread;

use crate::map::Maze;
use crate::sensors::mpu6050::MPU6050;
use crate::sensors::vl6180x::VL6180X;
use crate::vision::Vision;

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

    let bus = 1;
    if let Ok(mut mpu) = MPU6050::new(bus) {
        if let Ok(()) = mpu.run() {
            println!("Done!");
            for _ in 0..200 {
                println!(
                    "Roll: {}  Pitch: {}  Yaw: {}",
                    mpu.get_roll(),
                    mpu.get_pitch(),
                    mpu.get_yaw()
                );
                thread::sleep(std::time::Duration::from_millis(300));
            }
            mpu.stop();
        } else {
            println!("Error running MPU6050!");
        }
    } else {
        println!("Error creating MPU6050!");
    }

    let bus = 1;
    if let Ok(mut tof) = VL6180X::new(bus, Some(0x30)) {
        if let Ok(()) = tof.begin() {
            println!("Done!");
            for _ in 0..200 {
                println!("Range: {}mm", tof.range().unwrap());
                thread::sleep(std::time::Duration::from_millis(300));
            }
        } else {
            println!("Error running VL6180X!");
        }
    } else {
        println!("Error creating VL6180X!");
    }
}
