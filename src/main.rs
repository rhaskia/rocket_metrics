#![no_main]
#![no_std]

use arduino_hal::{prelude::*, I2c};
use panic_halt as _;

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

    i2c.write(0x76, &[0xF7, 0x3F]);
    let mut accel = [0; 3];
    loop {
        let adc_t = read24(&mut i2c, 0xf4);
        let t = temp(adc_t);
        ufmt::uwriteln!(&mut serial, "{:?}", (t * 100.0) as i32);

        arduino_hal::delay_ms(100);
    }
}

unsafe fn temp(data: u32) -> f32 {
    let mut data = data;
    data >>= 4;

    let t1 = 0x88;
    let t2 = 0x8A;
    let t3 = 0x8C;

    let var1 = (((data >> 3) - (t1 << 1)) * t2) >> 11;
    let var2 = (((((data >> 4) - t1) * ((data >> 4) - t1)) >> 12) * t3) >> 14;

    let fine = var1 + var2;

    let t = ((fine * 5 + 128) >> 8) as f32;
    t / 100.
}

unsafe fn read24(i2c: &mut I2c, reg: u8) -> u32 {
    let mut buf = [0u8; 3];

    i2c.write_read(0x76, &[reg], &mut buf);
    let mut value: u32 = 0;
    value |= (buf[2] as u32) << 24;  // Shift last byte 24 bits to the left (MSB)
    value |= (buf[1] as u32) << 16;  // Shift second byte 16 bits to the left
    value |= buf[0] as u32;

    value
}
