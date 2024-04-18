use opencv::{
    core::{Point, Scalar, Size},
    dnn::{DNN_BACKEND_OPENCV, DNN_TARGET_CPU}, // I will utilize my GPU to perform faster inference. Your way may vary
    highgui,
    imgproc::{self, put_text, rectangle, FONT_HERSHEY_SIMPLEX, LINE_4},
    prelude::*,
    videoio,
    Result,
};

use od_opencv::{model_format::ModelFormat, model_ultralytics::ModelUltralyticsV8};

fn main() -> Result<()> {
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
    if !videoio::VideoCapture::is_opened(&cam)? {
        panic!("Unable to open default camera!");
    }
    let classes_labels: Vec<&str> = vec!["GREEN", "H", "RED", "S", "U", "YELLOW"];
    let mf = ModelFormat::ONNX;
    let net_width = 480;
    let net_height = 384;

    let class_filters: Vec<usize> = vec![];

    let mut model = ModelUltralyticsV8::new_from_file(
        "bestsmall.onnx",
        None,
        (net_width, net_height),
        mf,
        DNN_BACKEND_OPENCV,
        DNN_TARGET_CPU,
        class_filters,
    )
    .unwrap();
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        let mut resized = Mat::default();
        imgproc::resize(
            &frame,
            &mut resized,
            Size {
                width: net_width,
                height: net_height,
            },
            0.0,
            0.0,
            imgproc::INTER_AREA,
        )?;

        let (bboxes, class_ids, confidences) = model.forward(&resized, 0.6, 0.7)?;
        println!("{}", bboxes.len());
        for (i, bbox) in bboxes.iter().enumerate() {
            let class_label = classes_labels[class_ids[i]];
            let confidence_text = format!("{:.2}", confidences[i]);
            rectangle(
                &mut resized,
                *bbox,
                Scalar::from((0.0, 255.0, 0.0)),
                2,
                LINE_4,
                0,
            )
            .unwrap();

            put_text(
                &mut resized,
                &(class_label.to_string() + ": " + &confidence_text),
                Point::new(bbox.tl().x, bbox.tl().y - 10),
                FONT_HERSHEY_SIMPLEX,
                0.5,
                Scalar::from((0.0, 255.0, 0.0)),
                2,
                1,
                false,
            )
            .unwrap();

            println!("Class: {}", class_label);
            println!("\tBounding box: {:?}", bbox);
            println!("\tConfidences: {}", confidence_text);
        }

        if resized.size()?.width > 0 {
            highgui::imshow(window, &resized)?;
        }
        let key = highgui::wait_key(10)?;
        if key == 113 {
            break;
        }
    }
    Ok(())
}
