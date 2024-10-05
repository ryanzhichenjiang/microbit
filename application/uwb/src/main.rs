#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_halt as _;

use core::str;
use cortex_m_rt::entry;
use defmt::info;

use microbit::{hal::{uarte, twim, gpio::Level}, Board};
use microbit_cutebot::Cutebot;

#[entry]
fn main() -> ! {
    if let Some(board) = Board::take() {
        let i2c_pins = board.i2c_internal;
        let mut i2c = twim::Twim::new(board.TWIM0, i2c_pins.into(), twim::Frequency::K100);
        let mut cutebot = Cutebot::new(&mut i2c);

        /* Configure RX and TX pins accordingly */
        board.edge.e01.into_push_pull_output(Level::High);
        board.edge.e00.into_floating_input();

        let uart0 = board.UART0;
        /* Tell UART which pins to use for sending and receiving */
        uart0.psel.txd.write(|w| unsafe { w.bits(3) });
        uart0.psel.rxd.write(|w| unsafe { w.bits(2) });

        /* Set a typical baud rate of 115200 */
        uart0.baudrate.write(|w| w.baudrate().baud115200());

        /* Enable UART function */
        uart0.enable.write(|w| w.enable().enabled());

        /* Print a nice hello message */
        let _ = write_uart0(&uart0, "Please type characters to echo:\r\n");

        /* Fire up receiving task */
        uart0.tasks_startrx.write(|w| unsafe { w.bits(1) });

        /* Initialize motors with forward speed and check if successful */
        match cutebot.motors(10, 10) {
            Ok(_) => {
                info!("Motors initialized successfully with forward speed");
                let _ = write_uart0(&uart0, "Motors started moving forward\r\n");
            }
            Err(_) => {
                info!("Failed to initialize motors");
                let _ = write_uart0(&uart0, "Error: Failed to start motors\r\n");
            }
        }

        /* Endless loop */
        loop {
            /* Busy wait for reception of data */
            while uart0.events_rxdrdy.read().bits() == 0 {}

            /* We're going to pick up the data soon, let's signal the buffer is already waiting for
             * more data */
            uart0.events_rxdrdy.write(|w| unsafe { w.bits(0) });

            /* Read one 8bit value */
            let c = uart0.rxd.read().bits() as u8;

            /* What comes in must go out, we don't care what it is */
            let _ = write_uart0(&uart0, unsafe { str::from_utf8_unchecked(&[c; 1]) });
        }
    }

    loop {
        continue;
    }
}

fn write_uart0(uart0: &microbit::pac::UART0, s: &str) -> core::fmt::Result {
    /* Start UART sender */
    uart0.tasks_starttx.write(|w| unsafe { w.bits(1) });

    for c in s.as_bytes() {
        /* Write the current character to the output register */
        uart0.txd.write(|w| unsafe { w.bits(u32::from(*c)) });

        /* Wait until the UART is clear to send */
        while uart0.events_txdrdy.read().bits() == 0 {}

        /* And then reset it for the next round */
        uart0.events_txdrdy.write(|w| unsafe { w.bits(0) });
    }

    /* Stop UART sender */
    uart0.tasks_stoptx.write(|w| unsafe { w.bits(1) });
    Ok(())
}
