#![no_std]

use core::fmt::Debug;

use crate::Error::DeviceId;
use embedded_hal as hal;

#[allow(unused)]
mod definitions;
use definitions::{AdcResolution, EnableRegField, PDriverRegField, Register};

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE> {
    Comm(CommE),

    /// The device ID read from the device is unrecognized
    DeviceId,
}

#[derive(Debug)]
pub struct HRS3300<I2C> {
    i2c_port: I2C,
    address: u8,
    /// The selected bits of resolution of the ADC
    adc_resolution: AdcResolution,
}

type HeartRateType = u16;

type LuminanceType = u32;
type AmbientLightType = LuminanceType;
type ReflectedLightType = LuminanceType;

impl<I2C, CommE> HRS3300<I2C>
where
    I2C: hal::blocking::i2c::Write<Error = CommE> + hal::blocking::i2c::WriteRead<Error = CommE>,
    CommE: core::fmt::Debug,
{
    pub const DEFAULT_DEVICE_ADDRESS: u8 = 0x44;
    /// recommended value of reserved resolution bits
    const RESERVED_RESOLUTION_BITS: u8 = 0x60;

    fn new(i2c_port: I2C, address: u8, adc_resolution: AdcResolution) -> Self {
        Self {
            i2c_port,
            address,
            adc_resolution,
        }
    }

    pub fn default(i2c_port: I2C) -> Self {
        Self::new(
            i2c_port,
            Self::DEFAULT_DEVICE_ADDRESS,
            AdcResolution::Bits14,
        )
    }

    pub fn init(&mut self) -> Result<(), Error<CommE>> {
        //first verify we can get a device ID
        let device_id = self.get_device_id()?;
        if device_id != DEFAULT_DEVICE_ID {
            return Err(DeviceId);
        }

        // There are only four writable registers we need to set:
        // PDRIVER
        // RES
        // HGAIN
        // Enable
        //TODO subdivide these raw register configs into finer-grained control
        // we currently set the configuration values according to datasheet recommendations
        let pdrive_reg_val = 0x68 ; //PDriverRegField::PDRIVE0 as u8 | PDriverRegField::PON as u8;
        self.write_register(Register::PDRIVER, pdrive_reg_val)?;
        self.write_register(Register::RES, 0x66)?;
        //TODO self.set_adc_resolution(self.adc_resolution)?;
        self.write_register(Register::HGAIN, 0x10)?;
        // enable and set conversion time to 12.5 ms
        let enable_reg_val = (EnableRegField::HEN as u8 | EnableRegField::PDRIVE1 as u8) | 0x60;
        self.write_register(Register::ENABLE, enable_reg_val)?;

        // self.enable(true)?;
        Ok(())
    }

    pub fn enable(&mut self, enable: bool) -> Result<(), Error<CommE>> {
        let enable_val = self.read_register(Register::ENABLE)?;
        let enable_val = if enable {
            enable_val | (EnableRegField::HEN as u8)
        } else {
            enable_val & !(EnableRegField::HEN as u8)
        };
        self.write_register(Register::ENABLE, enable_val)?;

        let pdrive_val = self.read_register(Register::PDRIVER)?;
        let pdrive_val = if enable {
            pdrive_val | (PDriverRegField::PON as u8)
        } else {
            pdrive_val & !(PDriverRegField::PON as u8)
        };
        self.write_register(Register::PDRIVER, pdrive_val)?;

        Ok(())
    }

    pub fn get_device_id(&mut self) -> Result<u8, Error<CommE>> {
        let device_id = self.read_register(Register::ID)?;
        Ok(device_id)
    }

    pub fn set_adc_resolution(&mut self, resolution: AdcResolution) -> Result<(), Error<CommE>> {
        self.adc_resolution = resolution;
        let res = Self::RESERVED_RESOLUTION_BITS & (resolution as u8);
        // let res = self.read_register(Register::RES)?;
        // let res = (res & 0xF0) | (resolution as u8);
        self.write_register(Register::RES, res)
    }

    pub fn enable_power(&mut self, enable: bool) -> Result<(), Error<CommE>> {
        let base = self.read_register(Register::ENABLE)?;
        let new_val = if enable {
            base | (EnableRegField::HEN as u8)
        } else {
            base & !(EnableRegField::HEN as u8)
        };
        self.write_register(Register::ENABLE, new_val)
    }

    /// Read a sample from the sensors,
    /// store it in a local time series sample buffer,
    /// and process it into a heart rate measurement if possible.
    /// Returns a heart rate measurement if one can be estimated
    /// from the time-series data available.
    ///
    pub fn sample_one(&mut self) -> Result<Option<HeartRateType>, Error<CommE>> {
        let _raw_sample = self.read_raw_sample()?;
        // TODO filter raw samples, detect peaks, return valid heart rate if known
        Ok(None)
    }

    /// Read a raw sample from the sensors
    /// Returns `(HRS, ALS)` where:
    /// - HRS has units of the reflected light type,
    /// - ALS has units of the ambient light
    /// These units are undocumented but we assume they're the same (luminance or equivalent)
    pub fn read_raw_sample(
        &mut self,
    ) -> Result<(ReflectedLightType, AmbientLightType), Error<CommE>> {
        let block = self.read_sample_block()?;
        // The order of returned data is:
        // 0: C1DATAM
        // 1: C0DATAM
        // 2: C0DATAH
        // 3: PDRIVER
        // 4: C1DATAH
        // 5: C1DATAL
        // 6: C0DATAL

        let mut c1: u32 = (block[0] as u32) << 3; // 7:0 -> C1DATA[10:3]
        let mut c0: u32 = (block[1] as u32) << 8; // 7:0 -> C0DATA[15:8]
        c0 |= ((block[2] & 0b1111) as u32) << 4; // 3:0 -> C0DATA[7:4]
        c1 |= ((block[4] & 0b111111) as u32) << 11; //  6:0 -> C1DATA[17:11]
        c1 |= (block[5] & 0b111) as u32; // 2:0 -> C1DATA[2:0]
        c0 |= ((block[6] & 0b110000) as u32) << 16; // 5:4 -> C0DATA[17:16]
        c0 |= (block[6] & 0b1111) as u32; // 3:0 -> C0DATA[3:0]
                                          // c0 is HRS reflectance / absorption
                                          // c1 is ambient light sensor (luminance)

        Ok((c0 as ReflectedLightType, c1 as AmbientLightType))
    }

    /// Read the multiple registers needed to form a complete sample
    /// The order returned is:
    /// C1DATAM = 0x08, 7:0 -> C1DATA[10:3]
    /// C0DATAM = 0x09, 7:0 -> C0DATA[15:8]
    /// C0DATAH = 0x0A, 3:0 -> C0DATA[7:4]
    /// PDRIVER = 0x0C,
    /// C1DATAH = 0x0D, 6:0 -> C1DATA[17:11]
    /// C1DATAL = 0x0E, 2:0 -> C1DATA[2:0]
    /// C0DATAL = 0x0F, 5:4 -> C0DATA[17:16], 3:0 -> C0DATA[3:0]
    fn read_sample_block(&mut self) -> Result<[u8; SAMPLE_BLOCK_LEN], Error<CommE>> {
        //
        let mut sample_buf = [0u8; SAMPLE_BLOCK_LEN];
        // read multiple registers starting at C1DATAM
        self.read_registers(Register::C1DATAM, sample_buf.as_mut())?;
        Ok(sample_buf)
    }

    fn read_register(&mut self, register: Register) -> Result<u8, Error<CommE>> {
        let mut data = [0];
        self.read_registers(register, data.as_mut())?;
        Ok(data[0])
    }

    fn write_register(&mut self, register: Register, value: u8) -> Result<(), Error<CommE>> {
        self.i2c_port
            .write(self.address, &[register as u8, value])
            .map_err(Error::Comm)
    }

    /// Read one or more registers at once, beginning at the start register
    fn read_registers(&mut self, start: Register, buf: &mut [u8]) -> Result<(), Error<CommE>> {
        self.i2c_port
            .write_read(self.address, &[start as u8], buf)
            .map_err(Error::Comm)?;
        Ok(())
    }
}

const SAMPLE_BLOCK_LEN: usize = 7;

const DEFAULT_DEVICE_ID: u8 = 0x21;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
