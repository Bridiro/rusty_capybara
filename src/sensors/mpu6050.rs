/*!
This module contains the implementation of the MPU6050 sensor.

The MPU6050 is a 6-axis motion tracking device that combines a 3-axis gyroscope and a 3-axis accelerometer.
It provides measurements of roll, pitch, and yaw angles.

# Example

```rust
use rusty_capybara::sensors::mpu6050::MPU6050;

fn main() {
    // Create a new MPU6050 sensor instance on I2C bus 1
    let mut mpu = MPU6050::new(1).unwrap();

    // Start reading sensor data
    mpu.run().unwrap();

    // Get roll, pitch, and yaw angles
    let roll = mpu.get_roll();
    let pitch = mpu.get_pitch();
    let yaw = mpu.get_yaw();

    // Stop reading sensor data
    mpu.stop();
}
```

# References

- [MPU-6050 Datasheet](https://www.invensense.com/wp-content/uploads/2015/02/MPU-6000-Datasheet1.pdf)
- [MPU-6050 Register Map](https://www.invensense.com/wp-content/uploads/2015/02/MPU-6000-Register-Map1.pdf)
- [MPU-6050 Tutorial](https://howtomechatronics.com/tutorials/arduino/arduino-and-mpu6050-accelerometer-and-gyroscope-tutorial)
- [RPPAL Documentation](https://docs.rs/rppal)

# Note

This implementation uses the `rppal` crate for I2C communication and error handling.
Make sure to add `rppal` as a dependency in your `Cargo.toml` file.

```toml
[dependencies]
rppal = "0.17.1"
```

The MPU6050 sensor must be connected to the I2C bus of the Raspberry Pi.
The I2C bus must be enabled on the Raspberry Pi.
You can enable the I2C bus by following the instructions in the Raspberry Pi documentation.

Make sure to enable the I2C bus before running the program.

# Safety

This implementation uses multi-threading to continuously read sensor data.
It is important to properly handle synchronization and ensure thread safety when accessing the sensor data.
The `MPU6050` struct provides methods to get the roll, pitch, and yaw angles, which internally lock the data using a mutex.
It is recommended to use these methods to access the sensor data in a thread-safe manner.
*/
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

/**
The MPU6050 struct represents the MPU6050 sensor.
It stores values of the angles on all axis.
*/
pub struct MPU6050 {
    i2c: Arc<Mutex<I2c>>,
    roll: Arc<Mutex<f32>>,
    pitch: Arc<Mutex<f32>>,
    yaw: Arc<Mutex<f32>>,
    running: Arc<Mutex<bool>>,
}

impl MPU6050 {
    /**
    Creates a new MPU6050 sensor instance on the specified I2C bus.
    # Arguments
    * `bus` - The I2C bus number (e.g., 1 for `/dev/i2c-1`).
    # Returns
    A `Result` containing the `MPU6050` sensor instance if successful, or an error if the sensor could not be initialized.
    # Errors
    This method returns an error if the I2C bus could not be opened or if there was an error initializing the sensor.
    # Example
    ```rust
    use rusty_capybara::sensors::mpu6050::MPU6050;

    let mut mpu = MPU6050::new(1).unwrap();
    ```
    # Safety
    This method accesses the I2C bus and initializes the MPU6050 sensor.
    It is important to handle errors and ensure that the sensor is properly connected and configured.
    Make sure to enable the I2C bus on the Raspberry Pi before running this method.
    The I2C bus must be enabled in the Raspberry Pi configuration.
    This method uses the `rppal` crate for I2C communication and error handling.
    Make sure to add `rppal` as a dependency in your `Cargo.toml` file.
    ```toml
    [dependencies]
    rppal = "0.17.1"
    ```
    */
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

    /**
    Starts reading sensor data from the MPU6050 sensor.
    This method continuously reads sensor data and calculates the roll, pitch, and yaw angles.
    The roll, pitch, and yaw angles can be accessed using the [`get_roll`](#method.get_pitch), [`get_pitch`](#method.get_pitch), and [`get_yaw`](#method.get_yaw) methods.
    # Returns
    A `Result` indicating whether the sensor data reading was started successfully.
    # Errors
    This method returns an error if there was an error reading sensor data or calculating the angles.
    # Example
    ```rust
    use rusty_capybara::sensors::mpu6050::MPU6050;

    let mut mpu = MPU6050::new(1).unwrap();
    mpu.run().unwrap();
    ```
    # Safety
    This method uses multi-threading to continuously read sensor data.
    It is important to properly handle synchronization and ensure thread safety when accessing the sensor data.
    The `MPU6050` struct provides methods to get the [roll](#method.get_pitch), [pitch](#method.get_pitch), and [yaw](#method.get_yaw) angles, which internally lock the data using a mutex.
    It is recommended to use these methods to access the sensor data in a thread-safe manner.
    This method uses the `rppal` crate for I2C communication and error handling.
    Make sure to add `rppal` as a dependency in your `Cargo.toml` file.
    ```toml
    [dependencies]
    rppal = "0.17.1"
    ```
    The MPU6050 sensor must be connected to the I2C bus of the Raspberry Pi.
    The I2C bus must be enabled on the Raspberry Pi.
    You can enable the I2C bus by following the instructions in the Raspberry Pi documentation.
    Make sure to enable the I2C bus before running the program.
    */
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

    /**
    Gets the roll angle in degrees.
    The roll angle represents the rotation around the x-axis.
    # Returns
    The roll angle in degrees.
    # Example
    ```rust
    use rusty_capybara::sensors::mpu6050::MPU6050;

    let mut mpu = MPU6050::new(1).unwrap();
    mpu.run().unwrap();
    let roll = mpu.get_roll();
    ```
    # Safety
    This method accesses the roll angle data using a mutex.
    It is important to ensure thread safety when accessing the sensor data.
    Make sure to call this method after starting the sensor data reading using the [`run`](#method.run) method.
    The roll angle is continuously updated while the sensor data reading is running.
    It is recommended to call this method periodically to get the latest roll angle value.
    This method uses the `rppal` crate for I2C communication and error handling.
    Make sure to add `rppal` as a dependency in your `Cargo.toml` file.
    ```toml
    [dependencies]
    rppal = "0.17.1"
    ```
    The MPU6050 sensor must be connected to the I2C bus of the Raspberry Pi.
    The I2C bus must be enabled on the Raspberry Pi.
    You can enable the I2C bus by following the instructions in the Raspberry Pi documentation.
    Make sure to enable the I2C bus before running the program.
    */
    pub fn get_roll(&self) -> f32 {
        *self.roll.lock().unwrap()
    }

    /**
    Gets the pitch angle in degrees.
    The pitch angle represents the rotation around the y-axis.
    # Returns
    The pitch angle in degrees.
    # Example
    ```rust
    use rusty_capybara::sensors::mpu6050::MPU6050;

    let mut mpu = MPU6050::new(1).unwrap();
    mpu.run().unwrap();
    let pitch = mpu.get_pitch();
    ```
    # Safety
    This method accesses the pitch angle data using a mutex.
    It is important to ensure thread safety when accessing the sensor data.
    Make sure to call this method after starting the sensor data reading using the [`run`](#method.run) method.
    The pitch angle is continuously updated while the sensor data reading is running.
    It is recommended to call this method periodically to get the latest pitch angle value.
    This method uses the `rppal` crate for I2C communication and error handling.
    Make sure to add `rppal` as a dependency in your `Cargo.toml` file.
    ```toml
    [dependencies]
    rppal = "0.17.1"
    ```
    The MPU6050 sensor must be connected to the I2C bus of the Raspberry Pi.
    The I2C bus must be enabled on the Raspberry Pi.
    You can enable the I2C bus by following the instructions in the Raspberry Pi documentation.
    Make sure to enable the I2C bus before running the program.
    */
    pub fn get_pitch(&self) -> f32 {
        *self.pitch.lock().unwrap()
    }

    /**
    Gets the yaw angle in degrees.
    The pitch angle represents the rotation around the z-axis.
    # Returns
    The yaw angle in degrees.
    # Example
    ```rust
    use rusty_capybara::sensors::mpu6050::MPU6050;

    let mut mpu = MPU6050::new(1).unwrap();
    mpu.run().unwrap();
    let yaw = mpu.get_yaw();
    ```
    # Safety
    This method accesses the yaw angle data using a mutex.
    It is important to ensure thread safety when accessing the sensor data.
    Make sure to call this method after starting the sensor data reading using the [`run`](#method.run) method.
    The yaw angle is continuously updated while the sensor data reading is running.
    It is recommended to call this method periodically to get the latest yaw angle value.
    This method uses the `rppal` crate for I2C communication and error handling.
    Make sure to add `rppal` as a dependency in your `Cargo.toml` file.
    ```toml
    [dependencies]
    rppal = "0.17.1"
    ```
    The MPU6050 sensor must be connected to the I2C bus of the Raspberry Pi.
    The I2C bus must be enabled on the Raspberry Pi.
    You can enable the I2C bus by following the instructions in the Raspberry Pi documentation.
    Make sure to enable the I2C bus before running the program.
    */
    pub fn get_yaw(&self) -> f32 {
        *self.yaw.lock().unwrap()
    }

    /// Stops reading sensor data from the MPU6050 sensor.
    /// This method stops the thread that reads sensor data and calculates the angles.
    /// # Example
    /// ```rust
    /// use rusty_capybara::sensors::mpu6050::MPU6050;
    ///
    /// let mut mpu = MPU6050::new(1).unwrap();
    /// mpu.run().unwrap();
    /// mpu.stop();
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

    /**
    Calculate the error in the accelerometer and gyroscope readings.
    This method reads raw data from the accelerometer and gyroscope and calculates the average error.
    The error values are used to compensate for the sensor drift.
    # Arguments
    * `samples` - The number of samples to read for calculating the error.
    # Returns
    A tuple containing the average error values for the accelerometer and gyroscope readings.
    The error values are calculated as follows:
    - Accelerometer error: The average accelerometer readings in the x, y, and z axes.
    - Gyroscope error: The average gyroscope readings in the x, y, and z axes.
    # Errors
    This method returns an error if there was an error reading raw data from the sensor.
    # Example
    ```rust
    use rusty_capybara::sensors::mpu6050::MPU6050;

    let mut mpu = MPU6050::new(1).unwrap();
    let (acc_x_err, acc_y_err, acc_z_err, gyro_x_err, gyro_y_err, gyro_z_err) = mpu.calculate_error(500).unwrap();
    ```
    */
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
