use arduino_hal::{prelude::*, I2c};

#[derive(Debug)]
pub struct Bmp280Calibration {
    pub dig_t1: u16,
    pub dig_t2: i16,
    pub dig_t3: i16,
    pub dig_p1: u16,
    pub dig_p2: i16,
    pub dig_p3: i16,
    pub dig_p4: i16,
    pub dig_p5: i16,
    pub dig_p6: i16,
    pub dig_p7: i16,
    pub dig_p8: i16,
    pub dig_p9: i16,
}

impl Bmp280Calibration {
    pub fn new(i2c: &mut I2c) -> Result<Self, arduino_hal::i2c::Error> {
        Ok(Bmp280Calibration {
            dig_t1: read16(i2c, 0x88)?,
            dig_t2: read_s16(i2c, 0x8A)?,
            dig_t3: read_s16(i2c, 0x8C)?,
            dig_p1: read16(i2c, 0x8E)?,
            dig_p2: read_s16(i2c, 0x90)?,
            dig_p3: read_s16(i2c, 0x92)?,
            dig_p4: read_s16(i2c, 0x94)?,
            dig_p5: read_s16(i2c, 0x96)?,
            dig_p6: read_s16(i2c, 0x98)?,
            dig_p7: read_s16(i2c, 0x9A)?,
            dig_p8: read_s16(i2c, 0x9C)?,
            dig_p9: read_s16(i2c, 0x9E)?,
        })
    }
}

fn read16(i2c: &mut I2c, reg: u8) -> Result<u16, arduino_hal::i2c::Error> {
    let mut buf = [0u8; 2];
    i2c.write_read(0x76, &[reg], &mut buf);
    // Combine the two bytes into a 16-bit unsigned integer
    let val = ((buf[0] as u16) << 8) | (buf[1] as u16);
    
    Ok(val)
}

fn read_s16(i2c: &mut I2c, reg: u8) -> Result<i16, arduino_hal::i2c::Error> {
    let mut buf = [0u8; 2];
    i2c.write_read(0x76, &[reg], &mut buf);

    // Combine the two bytes into a 16-bit signed integer
    let val = ((buf[0] as i16) << 8) | (buf[1] as i16);
    
    Ok(val)
}
