#![no_main]
#![no_std]

use arduino_hal::{prelude::*, I2c};
use panic_halt as _;
mod bmp280;
use bmp280::Bmp280Calibration;

#[arduino_hal::entry]
unsafe fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    let calib = Bmp280Calibration::new(&mut i2c).unwrap();
    i2c.write(0x76, &[0xF4, 0x3F]).unwrap();
    let mut accel = [0; 3];
    
    loop {
        let adc_t = read24(&mut i2c, 0xFA);
        let t = compensate_temperature(adc_t as i32, &calib);
        ufmt::uwriteln!(&mut serial, "{:?}", ((t - 32.0) * 500. / 9.) as i32);

        arduino_hal::delay_ms(100);
    }
}

fn compensate_temperature(raw_temp: i32, calib: &Bmp280Calibration) -> f32 {
    let mut adc_t = raw_temp;
    // adc_t >>= 4;

    let t1 = calib.dig_t1 as i32;
    let t2 = calib.dig_t2 as i32;
    let t3 = calib.dig_t3 as i32;

    let var1 = (((adc_t >> 3) - (t1 << 1)) * t2) >> 11;
    let var2 = (((((adc_t >> 4) - t1) * ((adc_t >> 4) - t1)) >> 12) * t3) >> 14;

    let fine = var1 + var2;

    let t = ((fine * 5 + 128) >> 8) as f32;
    t / 100.
}

unsafe fn read24(i2c: &mut I2c, reg: u8) -> u32 {
    let mut buf = [0u8; 3];

    i2c.write_read(0x76, &[reg], &mut buf).unwrap();
    let mut val = 0;
    val |= (buf[0] as u32) << 0;
    val |= (buf[1] as u32) << 8;
    val |= (buf[2] as u32) << 16;

    val
}
