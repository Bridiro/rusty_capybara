#![allow(dead_code)]
use super::{read8, write8};
use anyhow::Result;
use rppal::i2c::I2c;

const ADDR: u16 = 0x29;

const IDENTIFICATION_MODEL_ID: u16 = 0x000;
const SYSTEM_CHANGE_ADDRESS: u16 = 0x212;
const SYSTEM_HISTORY_CTRL: u16 = 0x012;
const SYSTEM_INTERRUPT_CONFIG: u16 = 0x014;
const SYSTEM_INTERRUPT_CLEAR: u16 = 0x015;
const SYSTEM_FRESH_OUT_OF_RESET: u16 = 0x016;

const SYSRANGE_START: u16 = 0x018;
const SYSRANGE_INTERMEASUREMENT_PERIOD: u16 = 0x01B;
const SYSRANGE_PART_TO_PART_RANGE_OFFSET: u16 = 0x024;

const RESULT_RANGE_STATUS: u16 = 0x04D;
const RESULT_INTERRUPT_STATUS_GPIO: u16 = 0x04F;
const RESULT_RANGE_VAL: u16 = 0x062;
const RESULT_RANGE_HISTORY_BUFFER_0: i16 = 0x052;

pub struct VL6180X {
    i2c: I2c,
    addr: u16,
}

impl VL6180X {
    pub fn new(bus: u8, addr: Option<u16>) -> Result<Self> {
        let i2c = I2c::with_bus(bus)?;
        let addr = addr.unwrap_or(ADDR);
        Ok(Self { i2c, addr })
    }

    pub fn begin(&mut self) -> Result<()> {
        self.i2c.set_slave_address(self.addr)?;
        if let Err(_) = read8(&mut self.i2c, IDENTIFICATION_MODEL_ID) {
            self.i2c.set_slave_address(ADDR)?;
            self.change_addr(self.addr)?;
            self.i2c.set_slave_address(self.addr)?;
        }
        if read8(&mut self.i2c, IDENTIFICATION_MODEL_ID)? != 0xB4 {
            return Err(anyhow::anyhow!(
                "Could not connect to VL6180X on address: {}",
                self.addr
            ));
        }

        self.load_settings()?;
        write8(&mut self.i2c, SYSTEM_FRESH_OUT_OF_RESET, 0x00)?;

        if self.continuous_mode_enabled()? {
            self.stop_range_continuous()?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        write8(&mut self.i2c, SYSTEM_HISTORY_CTRL, 0x01)?;

        Ok(())
    }

    pub fn range(&mut self) -> Result<u8> {
        if self.continuous_mode_enabled()? {
            self.read_range_continuous()
        } else {
            self.read_range_single()
        }
    }

    fn load_settings(&mut self) -> Result<()> {
        write8(&mut self.i2c, 0x0207, 0x01)?;
        write8(&mut self.i2c, 0x0208, 0x01)?;
        write8(&mut self.i2c, 0x0096, 0x00)?;
        write8(&mut self.i2c, 0x0097, 0xFD)?;
        write8(&mut self.i2c, 0x00E3, 0x00)?;
        write8(&mut self.i2c, 0x00E4, 0x04)?;
        write8(&mut self.i2c, 0x00E5, 0x02)?;
        write8(&mut self.i2c, 0x00E6, 0x01)?;
        write8(&mut self.i2c, 0x00E7, 0x03)?;
        write8(&mut self.i2c, 0x00F5, 0x02)?;
        write8(&mut self.i2c, 0x00D9, 0x05)?;
        write8(&mut self.i2c, 0x00DB, 0xCE)?;
        write8(&mut self.i2c, 0x00DC, 0x03)?;
        write8(&mut self.i2c, 0x00DD, 0xF8)?;
        write8(&mut self.i2c, 0x009F, 0x00)?;
        write8(&mut self.i2c, 0x00A3, 0x3C)?;
        write8(&mut self.i2c, 0x00B7, 0x00)?;
        write8(&mut self.i2c, 0x00BB, 0x3C)?;
        write8(&mut self.i2c, 0x00B2, 0x09)?;
        write8(&mut self.i2c, 0x00CA, 0x09)?;
        write8(&mut self.i2c, 0x0198, 0x01)?;
        write8(&mut self.i2c, 0x01B0, 0x17)?;
        write8(&mut self.i2c, 0x01AD, 0x00)?;
        write8(&mut self.i2c, 0x00FF, 0x05)?;
        write8(&mut self.i2c, 0x0100, 0x05)?;
        write8(&mut self.i2c, 0x0199, 0x05)?;
        write8(&mut self.i2c, 0x01A6, 0x1B)?;
        write8(&mut self.i2c, 0x01AC, 0x3E)?;
        write8(&mut self.i2c, 0x01A7, 0x1F)?;
        write8(&mut self.i2c, 0x0030, 0x00)?;

        write8(&mut self.i2c, 0x0011, 0x10)?;
        write8(&mut self.i2c, 0x010A, 0x30)?;
        write8(&mut self.i2c, 0x003F, 0x46)?;
        write8(&mut self.i2c, 0x0031, 0xFF)?;
        write8(&mut self.i2c, 0x0040, 0x63)?;
        write8(&mut self.i2c, 0x002E, 0x01)?;

        write8(&mut self.i2c, 0x001B, 0x09)?;
        write8(&mut self.i2c, 0x003E, 0x31)?;
        write8(&mut self.i2c, 0x0014, 0x24)?;

        Ok(())
    }

    pub fn change_addr(&mut self, addr: u16) -> Result<()> {
        write8(&mut self.i2c, SYSTEM_CHANGE_ADDRESS, addr as u8 & 0x7F)?;
        Ok(())
    }

    pub fn start_range_continuous(&mut self, period: i32) -> Result<()> {
        if let 10..=2550 = period {
            let period_reg = period / 10 - 1;
            write8(
                &mut self.i2c,
                SYSRANGE_INTERMEASUREMENT_PERIOD,
                period_reg as u8,
            )?;
            write8(&mut self.i2c, SYSRANGE_START, 0x03)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Period must be between 10 and 2550"))
        }
    }

    pub fn stop_range_continuous(&mut self) -> Result<()> {
        if self.continuous_mode_enabled()? {
            write8(&mut self.i2c, SYSRANGE_START, 0x01)?;
        }
        Ok(())
    }

    fn continuous_mode_enabled(&mut self) -> Result<bool> {
        Ok(read8(&mut self.i2c, SYSRANGE_START)? > 1 & 0x1)
    }

    pub fn offset(&mut self, offset: u8) -> Result<()> {
        write8(
            &mut self.i2c,
            SYSRANGE_PART_TO_PART_RANGE_OFFSET,
            offset.to_le_bytes()[0],
        )?;
        Ok(())
    }

    fn read_range_single(&mut self) -> Result<u8> {
        while read8(&mut self.i2c, RESULT_RANGE_STATUS)? & 0x01 == 0 {}
        write8(&mut self.i2c, SYSRANGE_START, 0x01)?;
        Ok(self.read_range_continuous()?)
    }

    fn read_range_continuous(&mut self) -> Result<u8> {
        while read8(&mut self.i2c, RESULT_INTERRUPT_STATUS_GPIO)? & 0x04 == 0 {}
        let range = read8(&mut self.i2c, RESULT_RANGE_VAL)?;
        write8(&mut self.i2c, SYSTEM_INTERRUPT_CLEAR, 0x07)?;
        Ok(range)
    }
}
