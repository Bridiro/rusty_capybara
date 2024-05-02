#![allow(dead_code)]
pub mod mpu6050;
pub mod vl6180x;

use anyhow::Result;
use rppal::i2c::I2c;

fn read_raw_data(i2c: &mut I2c, addr: u16) -> Result<i16> {
    let mut reg = [0u8, 2];
    i2c.block_read(addr as u8, &mut reg)?;
    Ok(((reg[0] as i16) << 8) | reg[1] as i16)
}

fn write8(i2c: &mut I2c, addr: u16, data: u8) -> Result<()> {
    i2c.write(&[(addr >> 8) as u8 & 0xFF, addr as u8 & 0xFF, data])?;
    Ok(())
}

fn write16(i2c: &mut I2c, addr: u16, data: u16) -> Result<()> {
    i2c.write(&[
        (addr >> 8) as u8 & 0xFF,
        addr as u8 & 0xFF,
        (data >> 8) as u8 & 0xFF,
        data as u8 & 0xFF,
    ])?;
    Ok(())
}

fn read8(i2c: &mut I2c, addr: u16) -> Result<u8> {
    let mut reg = [0u8; 1];
    i2c.write(&[(addr >> 8) as u8 & 0xFF, addr as u8 & 0xFF])?;
    i2c.read(&mut reg)?;
    Ok(reg[0])
}

fn read16(i2c: &mut I2c, addr: u16) -> Result<i16> {
    let mut reg = [0u8, 2];
    i2c.write(&[(addr >> 8) as u8 & 0xFF, addr as u8 & 0xFF])?;
    i2c.read(&mut reg)?;
    Ok(((reg[0] as i16) << 8) | reg[1] as i16)
}
