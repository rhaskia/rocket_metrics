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

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    let calib = Bmp280Calibration::new(&mut i2c).unwrap();
    let mut bmp = bmp280::Bmp280::new(calib);
    
    i2c.write(0x76, &[0xF4, 0x3F]).unwrap();
    let adc_p = read24(&mut i2c, 0xF7);
    bmp.zero(adc_p as i32);
    
    loop {
        let adc_t = read24(&mut i2c, 0xFA);
        let adc_p = read24(&mut i2c, 0xF7);
        let t = bmp.compensate_temperature(adc_t as i32);
        let height = bmp.altitude_m(adc_p as i32);
        ufmt::uwriteln!(&mut serial, "{:?}", (height * 1000.0) as i32).unwrap();

        arduino_hal::delay_ms(100);
    }
}

unsafe fn read24(i2c: &mut I2c, reg: u8) -> u32 {
    let mut buf = [0u8; 3];

    i2c.write_read(0x76, &[reg], &mut buf).unwrap();
    let mut val = 0;
    val |= (buf[2] as u32) << 0;
    val |= (buf[1] as u32) << 8;
    val |= (buf[0] as u32) << 16;

    val
}
