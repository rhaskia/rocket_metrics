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
            dig_t1: read16_le(i2c, 0x88)?,
            dig_t2: read_s16_le(i2c, 0x8A)?,
            dig_t3: read_s16_le(i2c, 0x8C)?,
            dig_p1: read16_le(i2c, 0x8E)?,
            dig_p2: read_s16_le(i2c, 0x90)?,
            dig_p3: read_s16_le(i2c, 0x92)?,
            dig_p4: read_s16_le(i2c, 0x94)?,
            dig_p5: read_s16_le(i2c, 0x96)?,
            dig_p6: read_s16_le(i2c, 0x98)?,
            dig_p7: read_s16_le(i2c, 0x9A)?,
            dig_p8: read_s16_le(i2c, 0x9C)?,
            dig_p9: read_s16_le(i2c, 0x9E)?,
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

fn read16_le(i2c: &mut I2c, reg: u8) -> Result<u16, arduino_hal::i2c::Error> {
    let mut buf = [0u8; 2];
    i2c.write_read(0x76, &[reg], &mut buf);
    // Combine the two bytes into a 16-bit unsigned integer
    let val = ((buf[1] as u16) << 8) | (buf[0] as u16);
    
    Ok(val)
}

fn read_s16_le(i2c: &mut I2c, reg: u8) -> Result<i16, arduino_hal::i2c::Error> {
    let mut buf = [0u8; 2];
    i2c.write_read(0x76, &[reg], &mut buf);

    // Combine the two bytes into a 16-bit signed integer
    let val = ((buf[1] as i16) << 8) | (buf[0] as i16);
    
    Ok(val)
}

pub struct Bmp280 {
    t_fine: i32,
    calib: Bmp280Calibration,
    ground_pressure: f32,
}

impl Bmp280 {
    pub fn new(calib: Bmp280Calibration) -> Self {
         Self {
             ground_pressure:0.0,
             calib, t_fine: 0,
         }
    }

    pub fn compensate_temperature(&mut self, adc_t: i32) -> f32 {
        let adc_t = adc_t >> 4;
        let t1 = self.calib.dig_t1 as i32;
        let t2 = self.calib.dig_t2 as i32;
        let t3 = self.calib.dig_t3 as i32;

        let var1 = (((adc_t >> 3) - (t1 << 1)) * t2) >> 11;
        let var2 = (((((adc_t >> 4) - t1) * ((adc_t >> 4) - t1)) >> 12) * t3) >> 14;

        self.t_fine = var1 + var2;

        let t = ((self.t_fine * 5 + 128) >> 8) as f32;
        t / 100.
    }

    pub fn compensate_pressure(&mut self, adc_p: i32) -> f32 {
        let var1 = (self.t_fine as i64) - 128000;

        let var2 = var1 * var1 * self.calib.dig_p6 as i64
            + ((var1 * self.calib.dig_p5 as i64) << 17)
            + ((self.calib.dig_p4 as i64) << 35);

        let var1 = (((var1 * var1 * (self.calib.dig_p3 as i64) >> 8) + ((var1 * (self.calib.dig_p2 as i64)) << 12))
            * ((1i64 << 47) + var1)
            * (self.calib.dig_p1 as i64)) >> 33;

        let p: i64 = 1048576 - adc_p as i64;
        let p = (((p << 31) - var2) * 3125) / var1;

        let var1 = ((self.calib.dig_p9 as i64) * (p >> 13)) * (p >> 13) >> 25;
        let var2 = ((self.calib.dig_p8 as i64) * p) >> 19;

        let p = ((p + var1 + var2) >> 8) + ((self.calib.dig_p7 as i64) << 4);

        p as f32 / 256000.
    }

    // pub fn zero(&mut self, adc_p: i32) -> f32 {
    //     self.ground_pressure = self.compensate_pressure(adc_p) * 1000.;
    //
    //     self.ground_pressure
    // }

    pub fn altitude_m_relative(&mut self, adc_p: i32, sea_level_pa: f32) -> f32 {
        let pressure = self.compensate_pressure(adc_p) * 1000.;

        let altitude = 44330. * (1. - pow_float(pressure / sea_level_pa, 0.1903));
        altitude
    }

    pub fn altitude_m(&mut self, adc_p: i32) -> f32 {
        let pressure = self.ground_pressure;

        self.altitude_m_relative(adc_p, pressure)
    }

}

fn pow_float(base: f32, exponent: f32) -> f32 {
    let base_as_u32 = base.to_bits();
    let exponent_as_u32 = exponent.to_bits();

    let product_as_u32 = base_as_u32.wrapping_mul(exponent_as_u32);
    f32::from_bits(product_as_u32)
}
