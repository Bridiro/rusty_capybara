pub mod vision {
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

    pub struct Vision<'a> {
        cam: videoio::VideoCapture,
        model: ModelUltralyticsV8,
        classes_labels: Vec<&'a str>,
        net_width: i32,
        net_height: i32,
    }

    impl<'a> Vision<'a> {
        pub fn new(
            camera_index: i32,
            model_path: &str,
            classes_labels: Vec<&'a str>,
            net_width: i32,
            net_height: i32,
            class_filters: Vec<usize>,
        ) -> Result<Self> {
            let cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
            if !videoio::VideoCapture::is_opened(&cam)? {
                panic!("Unable to open default camera!");
            }
            let mf = ModelFormat::ONNX;
            let model = ModelUltralyticsV8::new_from_file(
                model_path,
                None,
                (net_width, net_height),
                mf,
                DNN_BACKEND_OPENCV,
                DNN_TARGET_CPU,
                class_filters.clone(),
            )?;
            Ok(Self {
                cam,
                model,
                classes_labels,
                net_width,
                net_height,
            })
        }

        pub fn run(&mut self, conf_threshold: f32, nms_threshold: f32) -> Result<()> {
            let window = "video capture";
            highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
            loop {
                let mut frame = Mat::default();
                self.cam.read(&mut frame)?;

                let mut resized = Mat::default();
                imgproc::resize(
                    &frame,
                    &mut resized,
                    Size {
                        width: self.net_width,
                        height: self.net_height,
                    },
                    0.0,
                    0.0,
                    imgproc::INTER_AREA,
                )?;

                let (bboxes, class_ids, confidences) =
                    self.model
                        .forward(&resized, conf_threshold, nms_threshold)?;
                for (i, bbox) in bboxes.iter().enumerate() {
                    let class_label = self.classes_labels[class_ids[i]];
                    let confidence_text = format!("{:.2}", confidences[i]);
                    rectangle(
                        &mut resized,
                        *bbox,
                        Scalar::new(0.0, 255.0, 0.0, 0.0),
                        2,
                        LINE_4,
                        0,
                    )?;
                    put_text(
                        &mut resized,
                        &format!("{}: {}", class_label, confidence_text),
                        Point {
                            x: bbox.x,
                            y: bbox.y,
                        },
                        FONT_HERSHEY_SIMPLEX,
                        0.5,
                        Scalar::new(0.0, 255.0, 0.0, 0.0),
                        1,
                        LINE_4,
                        false,
                    )?;
                }
                highgui::imshow(window, &resized)?;
                if highgui::wait_key(10)? > 0 {
                    break;
                }
            }
            Ok(())
        }
    }
}
