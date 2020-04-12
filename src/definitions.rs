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
pub enum Register {
    ID = 0x00,     // device ID: s/b 0x21
    ENABLE = 0x01, // Enable HRS (rec:  0x68)
    C1DATAM = 0x08,
    C0DATAM = 0x09,
    C0DATAH = 0x0A, // CH0 data register bit 7~4
    PDRIVER = 0x0C, // HRS LED driver/PON/PDRIVE[0]  (rec: 0x68)
    C1DATAH = 0x0D, // CH1 data register bit 17~11
    C1DATAL = 0x0E, // CH1 data register bit 2~0
    C0DATAL = 0x0F, // CH1 data register bit 17~16 and 3~0
    RES = 0x16,     // ALS and HRS (ADC) resolution (rec: 0x66)
    HGAIN = 0x17,   // HRS gain
}

/// Fields from ENABLE Register(0x01)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EnableRegField {
    /// HRS sensor enable
    HEN = 1 << 7,
    /// HRS wait time
    HWT = 0b111 << 4,
    /// Used for LED drive current setup
    PDRIVE1 = 1 << 3,
}

/// Fields from HRS LED Driver Set Register (0x0C)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PDriverRegField {
    /// Used for LED drive current configuration
    PDRIVE0 = 1 << 6,
    /// Write 1 to enable LED OSC, write 0 to disable LED OSC
    PON = 1 << 5,
}

/// Fields from RESOLUTION Register (0x16)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ResRegField {
    /// ALS ADC resolution. Generate TSEL to analog in ALS mode
    ALS_RES = 0b1111,
}

/// Fields from HGAIN Register(0x17)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum HgainRegField {
    /// HRS gain
    HGAIN = 0b111 << 2,
}
