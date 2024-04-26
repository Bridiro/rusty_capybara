pub mod mpu6050;

use rppal::i2c::I2c;
use std::error::Error;

fn read_raw_data(i2c: &mut I2c, addr: u16) -> Result<i16, Box<dyn Error>> {
    let mut reg = [0u8, 2];
    i2c.block_read(addr as u8, &mut reg)?;
    Ok(((reg[0] as i16) << 8) | reg[1] as i16)
}
