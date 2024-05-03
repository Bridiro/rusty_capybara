#![allow(dead_code)]
use od_opencv::{model_format::ModelFormat, model_ultralytics::ModelUltralyticsV8};
use opencv::{
    core::{Point, Rect, Scalar, Size},
    dnn::{DNN_BACKEND_OPENCV, DNN_TARGET_CPU}, // I will utilize my GPU to perform faster inference. Your way may vary
    highgui,
    imgproc::{self, put_text, rectangle, FONT_HERSHEY_SIMPLEX, LINE_4},
    prelude::*,
    videoio,
    Result,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub struct Vision {
    cam: Arc<Mutex<videoio::VideoCapture>>,
    model: Arc<Mutex<ModelUltralyticsV8>>,
    classes_labels: Vec<String>,
    net_width: i32,
    net_height: i32,
    detection_channel: Sender<Detection>,
    running: Arc<Mutex<bool>>,
}

pub struct Detection {
    pub class_label: String,
    pub confidence: f32,
    pub bbox: Rect,
}

impl Vision {
    pub fn new(
        camera_index: i32,
        model_path: &str,
        classes_labels: Vec<String>,
        net_width: i32,
        net_height: i32,
        class_filters: Vec<usize>,
        detection_channel: Sender<Detection>,
    ) -> Result<Self> {
        let cam = Arc::new(Mutex::new(videoio::VideoCapture::new(
            camera_index,
            videoio::CAP_ANY,
        )?));
        if !videoio::VideoCapture::is_opened(&cam.lock().unwrap())? {
            panic!("Unable to open default camera!");
        }
        let mf = ModelFormat::ONNX;
        let model = Arc::new(Mutex::new(ModelUltralyticsV8::new_from_file(
            model_path,
            None,
            (net_width, net_height),
            mf,
            DNN_BACKEND_OPENCV,
            DNN_TARGET_CPU,
            class_filters.clone(),
        )?));
        let running = Arc::new(Mutex::new(false));
        Ok(Self {
            cam,
            model,
            classes_labels,
            net_width,
            net_height,
            detection_channel,
            running,
        })
    }

    pub fn run(&mut self, conf_threshold: f32, nms_threshold: f32, graphical: bool) -> Result<()> {
        *self.running.lock().unwrap() = true;
        let running = self.running.clone();
        let detection_channel = self.detection_channel.clone();
        let classes_labels = self.classes_labels.clone();
        let net_width = self.net_width.clone();
        let net_height = self.net_height.clone();
        let cam = self.cam.clone();
        let model = self.model.clone();

        std::thread::spawn(move || -> Result<()> {
            let window = "video capture";
            if graphical {
                highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
            }
            while *running.lock().unwrap() {
                let mut frame = Mat::default();
                cam.lock().unwrap().read(&mut frame)?;

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

                let (bboxes, class_ids, confidences) =
                    model
                        .lock()
                        .unwrap()
                        .forward(&resized, conf_threshold, nms_threshold)?;

                for (i, bbox) in bboxes.iter().enumerate() {
                    let class_label = &classes_labels[class_ids[i]];
                    let confidence_text = format!("{:.2}", confidences[i]);
                    let detection = Detection {
                        class_label: class_label.to_string(),
                        confidence: confidences[i],
                        bbox: *bbox,
                    };
                    detection_channel
                        .send(detection)
                        .map_err(|err| opencv::Error::new(0, err.to_string()))?;
                    if graphical {
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
                }
                if graphical {
                    highgui::imshow(window, &resized)?;
                    if highgui::wait_key(10)? > 0 {
                        break;
                    }
                }
            }
            Ok(())
        });
        Ok(())
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}
