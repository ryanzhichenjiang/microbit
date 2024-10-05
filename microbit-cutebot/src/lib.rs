#![no_std]

use embedded_hal::i2c::I2c;

const CUTEBOT_ADDR: u8 = 0x10;
const LEFT_MOTOR_REG: u8 = 0x01;
const RIGHT_MOTOR_REG: u8 = 0x02;

pub struct Cutebot<I2C> {
    i2c: I2C,
}

impl<I2C, E> Cutebot<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn motors(&mut self, left: i8, right: i8) -> Result<(), E> {
        // Convert signed speed (-100 to 100) to unsigned (0 to 200)
        let left_speed = ((left as i16 + 100) as u8).min(200);
        let right_speed = ((right as i16 + 100) as u8).min(200);

        // Write speeds to respective motor registers
        self.i2c.write(CUTEBOT_ADDR, &[LEFT_MOTOR_REG, left_speed])?;
        self.i2c.write(CUTEBOT_ADDR, &[RIGHT_MOTOR_REG, right_speed])?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), E> {
        self.motors(0, 0)
    }
}