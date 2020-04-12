#![no_std]
#[allow(non_camel_case_types)]

use core::fmt::Debug;

use embedded_hal as hal;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE> {
    Comm(CommE),

    GenericError,
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
type AbsorptionType = LuminanceType;

impl<I2C, CommE> HRS3300<I2C>
    where
        I2C: hal::blocking::i2c::Write<Error = CommE>
        + hal::blocking::i2c::WriteRead<Error = CommE>,
        CommE: core::fmt::Debug
{
    pub const DEFAULT_DEVICE_ADDRESS: u8 = 0x44;

    fn new(i2c_port: I2C, address: u8, adc_resolution: AdcResolution) -> Self {
        Self {
            i2c_port,
            address,
            adc_resolution,
        }
    }

    pub fn default(i2c_port: I2C) -> Self {
        Self::new(i2c_port, Self::DEFAULT_DEVICE_ADDRESS, AdcResolution::Bits14 )
    }

    pub fn init(&mut self) -> Result<(), Error<CommE>> {
        self.set_adc_resolution(self.adc_resolution)?;
        Ok(())
    }

    pub fn set_adc_resolution(&mut self, resolution: AdcResolution) -> Result<(),  Error<CommE>> {
        self.adc_resolution = resolution;
        let res = self.read_register(Register::RES)?;
        let res = (res & 0xF0) | (resolution as u8);
        self.write_register(Register::RES, res)
    }

    pub fn enable_power(&mut self, enable: bool) -> Result<(),  Error<CommE>> {
        let base = self.read_register(Register::ENABLE)?;
        let new_val =
            if enable { base | (EnableRegField::HEN as u8) }
            else { base & !(EnableRegField::HEN as u8)};
        self.write_register(Register::ENABLE, new_val)
    }

    pub fn sample_one(&mut self) -> Result<Option<HeartRateType>,  Error<CommE>> {
        let raw_sample = self.read_raw_sample()?;
        // TODO filter raw samples, detect peaks, return valid heart rate if known
       Ok(None)
    }

    pub fn read_raw_sample(&mut self) -> Result<(AmbientLightType, AbsorptionType),  Error<CommE>>  {
        let block = self.read_sample_block()?;
        //TODO convert raw data into ALS and HRS measurements
        unimplemented!()
    }

    /// Read the multiple registers needed to form a complete sample
    fn read_sample_block(&mut self) -> Result<[u8;SAMPLE_BLOCK_LEN],  Error<CommE>>  {
        //
        let mut sample_buf = [0u8; SAMPLE_BLOCK_LEN];
        // read multiple registers starting at C1DATAM
        self.read_registers(Register::C1DATAM, sample_buf.as_mut())?;
        Ok(sample_buf)
    }

    fn read_register(&mut self, register: Register) -> Result<u8,  Error<CommE>> {
        let mut data = [0];
        self.read_registers(register, data.as_mut())?;
        Ok(data[0])
    }

    fn write_register(&mut self, register: Register, value: u8) -> Result<(),  Error<CommE>> {
        self.i2c_port
            .write(self.address, &[register as u8, value])
            .map_err(Error::Comm)
    }

    /// Read one or more registers at once, beginning at the start register
    fn read_registers(&mut self, start: Register, buf: &mut [u8]) -> Result<(),  Error<CommE>> {
        self.i2c_port
            .write_read(self.address, &[start as u8], buf)
            .map_err(Error::Comm)?;
        Ok(())
    }
}

const SAMPLE_BLOCK_LEN: usize = 7;


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum AdcResolution {
    Bits8 = 0,
    Bits9 = 1,
    Bits10 = 2,
    Bits11 = 3,
    Bits12 = 4,
    Bits13 = 5,
    Bits14 = 6,
    Bits15 = 7,
    Bits16 = 8,
    Bits17 = 9,
    Bits18 = 10,
}

/// Registers described in the data sheet for this device
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum Register {
    ID = 0x00, // device ID: s/b 0x21
    ENABLE = 0x01, // Enable HRS (rec:  0x68)
    C1DATAM = 0x08,
    C0DATAM = 0x09,
    C0DATAH = 0x0A, // CH0 data register bit 7~4
    PDRIVER = 0x0C, // HRS LED driver/PON/PDRIVE[0]  (rec: 0x68)
    C1DATAH = 0x0D, // CH1 data register bit 17~11
    C1DATAL = 0x0E, // CH1 data register bit 2~0
    RES =  0x16, // ALS and HRS (ADC) resolution (rec: 0x66)
    HGAIN  = 0x17, // HRS gain
}

/// Fields from ENABLE Register(0x01)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum EnableRegField {
    /// HRS enable
    HEN = 1 << 7,
    /// HRS wait time
    HWT = 0b111 << 4,
    /// LED drive current setup
    PDRIVE1 = 1 << 3,
}

/// Fields from HRS LED Driver Set Register (0x0C)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum PDriverRegField {
    /// Used for LED drive current setup
    PDRIVE0 = 1 << 6,
    /// Write 1 active OSC, write 0 disable OSC. Generate PD signal to analog(0 for work, 1 for Power down)
    PON = 1 << 5,
}

/// Fields from RESOLUTION Register (0x16)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum ResRegField {
    /// ALS ADC resolution. Generate TSEL to analog in ALS mode
    ALS_RES = 0b1111,
}

/// Fields from HGAIN Register(0x17)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum HgainRegField {
    /// HRS gain
    HGAIN = 0b111 << 2,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
