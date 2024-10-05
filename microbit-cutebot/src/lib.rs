#![no_std]

use embedded_hal::i2c::I2c;

const CUTEBOT_ADDR: u8 = 0x10;
const LEFT_MOTOR_REG: u8 = 0x01;
const RIGHT_MOTOR_REG: u8 = 0x02;
const LEFT_RGB_REG: u8 = 0x03;
const RIGHT_RGB_REG: u8 = 0x04;

#[derive(Clone, Copy)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const RED: Rgb = Rgb(255, 0, 0);
    pub const GREEN: Rgb = Rgb(0, 255, 0);
    pub const BLUE: Rgb = Rgb(0, 0, 255);
    pub const BLACK: Rgb = Rgb(0, 0, 0);
    pub const WHITE: Rgb = Rgb(255, 255, 255);
}

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
        self.i2c.write(CUTEBOT_ADDR, &[LEFT_MOTOR_REG, 2, left_speed])?;
        self.i2c.write(CUTEBOT_ADDR, &[RIGHT_MOTOR_REG, 2, right_speed])?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), E> {
        self.motors(0, 0)
    }

    pub fn set_rgb(&mut self, left: Rgb, right: Rgb) -> Result<(), E> {
        // Write RGB values to respective LED registers
        let Rgb(left_r, left_g, left_b) = left;
        let Rgb(right_r, right_g, right_b) = right;

        self.i2c.write(CUTEBOT_ADDR, &[LEFT_RGB_REG, left_r, left_g, left_b])?;
        self.i2c.write(CUTEBOT_ADDR, &[RIGHT_RGB_REG, right_r, right_g, right_b])?;

        Ok(())
    }

    pub fn rgb_left(&mut self, color: Rgb) -> Result<(), E> {
        let current_right = Rgb(0, 0, 0); // Default to off
        self.set_rgb(color, current_right)
    }

    pub fn rgb_right(&mut self, color: Rgb) -> Result<(), E> {
        let current_left = Rgb(0, 0, 0); // Default to off
        self.set_rgb(current_left, color)
    }

    pub fn rgb_off(&mut self) -> Result<(), E> {
        self.set_rgb(Rgb(0, 0, 0), Rgb(0, 0, 0))
    }
}