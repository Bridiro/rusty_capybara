use super::read_raw_data;
use rppal::i2c::I2c;
use std::error::Error;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

const ADDR: u16 = 0x68;
const PWR_MGMT_1: u16 = 0x6B;
const SMPLRT_DIV: u16 = 0x19;
const CONFIG: u16 = 0x1A;
const GYRO_CONFIG: u16 = 0x1B;
const INT_ENABLE: u16 = 0x38;
const ACCEL_XOUT_H: u16 = 0x3B;
const ACCEL_YOUT_H: u16 = 0x3D;
const ACCEL_ZOUT_H: u16 = 0x3F;
const GYRO_XOUT_H: u16 = 0x43;
const GYRO_YOUT_H: u16 = 0x45;
const GYRO_ZOUT_H: u16 = 0x47;

pub struct MPU6050 {
    i2c: Arc<Mutex<I2c>>,
    roll: Arc<Mutex<f32>>,
    pitch: Arc<Mutex<f32>>,
    yaw: Arc<Mutex<f32>>,
    running: Arc<Mutex<bool>>,
}

impl MPU6050 {
    pub fn new(bus: u8) -> Result<MPU6050, Box<dyn Error>> {
        let i2c = Arc::new(Mutex::new(I2c::with_bus(bus)?));
        let mut mpu = MPU6050 {
            i2c,
            roll: Arc::new(Mutex::new(0.0)),
            pitch: Arc::new(Mutex::new(0.0)),
            yaw: Arc::new(Mutex::new(0.0)),
            running: Arc::new(Mutex::new(false)),
        };
        mpu.init()?;
        Ok(mpu)
    }

    pub fn run(&mut self) -> Result<(), rppal::i2c::Error> {
        let i2c = self.i2c.clone();
        let roll = self.roll.clone();
        let pitch = self.pitch.clone();
        let yaw = self.yaw.clone();
        let running = self.running.clone();

        let (acc_x_err, acc_y_err, _acc_z_err, gyro_x_err, gyro_y_err, gyro_z_err) =
            self.calculate_error(500).expect("Error calculating error");

        std::thread::spawn(move || {
            let mut previous_time = std::time::Instant::now();
            let mut gyro_angle_x = 0.0;
            let mut gyro_angle_y = 0.0;
            *roll.lock().unwrap() = 0.0;
            *pitch.lock().unwrap() = 0.0;
            *yaw.lock().unwrap() = 0.0;
            *running.lock().unwrap() = true;
            let mut last_yaw_rate = 0.0;

            while *running.lock().unwrap() {
                let acc_x = read_raw_data(&mut i2c.lock().unwrap(), ACCEL_XOUT_H)
                    .expect("Failed to read raw data") as f32
                    / 16384.0;
                let acc_y = read_raw_data(&mut i2c.lock().unwrap(), ACCEL_YOUT_H)
                    .expect("Failed to read raw data") as f32
                    / 16384.0;
                let acc_z = read_raw_data(&mut i2c.lock().unwrap(), ACCEL_ZOUT_H)
                    .expect("Failed to read raw data") as f32
                    / 16384.0;

                let acc_angle_x = (acc_y / (acc_x.powi(2) + acc_z.powi(2)).sqrt()).atan() * 180.0
                    / PI
                    - acc_x_err;
                let acc_angle_y =
                    (-(acc_x / (acc_y.powi(2) + acc_z.powi(2)).sqrt()).atan() * 180.0 / PI)
                        - acc_y_err;

                let gyro_x = read_raw_data(&mut i2c.lock().unwrap(), GYRO_XOUT_H)
                    .expect("Failed to read raw data") as f32
                    / 131.0;
                let gyro_y = read_raw_data(&mut i2c.lock().unwrap(), GYRO_YOUT_H)
                    .expect("Failed to read raw data") as f32
                    / 131.0;
                let gyro_z = read_raw_data(&mut i2c.lock().unwrap(), GYRO_ZOUT_H)
                    .expect("Failed to read raw data") as f32
                    / 131.0;

                let elapsed_time = previous_time.elapsed().as_secs_f32();
                previous_time = std::time::Instant::now();

                gyro_angle_x += (gyro_x - gyro_x_err) * elapsed_time;
                gyro_angle_y += (gyro_y - gyro_y_err) * elapsed_time;

                *yaw.lock().unwrap() += (gyro_z - gyro_z_err + last_yaw_rate) * 0.5 * elapsed_time;
                last_yaw_rate = gyro_z - gyro_z_err;

                *yaw.lock().unwrap() %= 360.0;

                *roll.lock().unwrap() = 0.98 * gyro_angle_x + 0.02 * acc_angle_x;
                *pitch.lock().unwrap() = 0.98 * gyro_angle_y + 0.02 * acc_angle_y;

                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        });

        Ok(())
    }

    pub fn get_roll(&self) -> f32 {
        *self.roll.lock().unwrap()
    }

    pub fn get_pitch(&self) -> f32 {
        *self.pitch.lock().unwrap()
    }

    pub fn get_yaw(&self) -> f32 {
        *self.yaw.lock().unwrap()
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.i2c.lock().unwrap().set_slave_address(ADDR)?;

        self.i2c
            .lock()
            .unwrap()
            .smbus_write_byte(PWR_MGMT_1 as u8, 0x00)?;
        self.i2c
            .lock()
            .unwrap()
            .smbus_write_byte(SMPLRT_DIV as u8, 0x07)?;
        self.i2c
            .lock()
            .unwrap()
            .smbus_write_byte(CONFIG as u8, 0x06)?;
        self.i2c
            .lock()
            .unwrap()
            .smbus_write_byte(GYRO_CONFIG as u8, 0x00)?;
        self.i2c
            .lock()
            .unwrap()
            .smbus_write_byte(INT_ENABLE as u8, 0x01)?;

        Ok(())
    }

    fn calculate_error(
        &mut self,
        samples: i32,
    ) -> Result<(f32, f32, f32, f32, f32, f32), Box<dyn Error>> {
        let mut acc_x = 0.0;
        let mut acc_y = 0.0;
        let mut acc_z = 0.0;
        let mut gyro_x = 0.0;
        let mut gyro_y = 0.0;
        let mut gyro_z = 0.0;

        for _ in 0..samples {
            acc_x += read_raw_data(&mut self.i2c.lock().unwrap(), ACCEL_XOUT_H)? as f32 / 16384.0;
            acc_y += read_raw_data(&mut self.i2c.lock().unwrap(), ACCEL_YOUT_H)? as f32 / 16384.0;
            acc_z += read_raw_data(&mut self.i2c.lock().unwrap(), ACCEL_ZOUT_H)? as f32 / 16384.0;
            gyro_x += read_raw_data(&mut self.i2c.lock().unwrap(), GYRO_XOUT_H)? as f32 / 131.0;
            gyro_y += read_raw_data(&mut self.i2c.lock().unwrap(), GYRO_YOUT_H)? as f32 / 131.0;
            gyro_z += read_raw_data(&mut self.i2c.lock().unwrap(), GYRO_ZOUT_H)? as f32 / 131.0;
        }

        acc_x /= samples as f32;
        acc_y /= samples as f32;
        acc_z /= samples as f32;
        gyro_x /= samples as f32;
        gyro_y /= samples as f32;
        gyro_z /= samples as f32;

        Ok((acc_x, acc_y, acc_z, gyro_x, gyro_y, gyro_z))
    }
}
