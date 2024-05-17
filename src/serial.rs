#![allow(dead_code)]
use rppal::uart::{Parity, Uart};
use std::{process::Command, time::Duration};

pub struct Serial {
    uart: Uart,
}

impl Serial {
    pub fn new(baud_rate: u32) -> Self {
        let output = Command::new("sh")
            .arg("-c")
            .arg("ls /dev/ttyACM*")
            .output()
            .unwrap();
        let path = String::from_utf8(output.stdout).unwrap();
        let mut uart = Uart::with_path(&path.trim(), baud_rate, Parity::None, 8, 1).unwrap();
        uart.set_read_mode(0, Duration::default()).unwrap();
        Self { uart }
    }

    pub fn write(&mut self, data: Vec<u8>) {
        self.uart.write(&data).unwrap();
    }

    pub fn read(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.uart.read(&mut buffer).unwrap();
        buffer[0]
    }

    pub fn readln(&mut self) -> String {
        let mut buffer = [0u8; 1];
        let mut data = String::new();
        loop {
            self.uart.read(&mut buffer).unwrap();
            let byte = buffer[0];
            if byte == b'\n' || byte == b'\r' {
                break;
            }
            data.push(byte as char);
        }
        data
    }
}
