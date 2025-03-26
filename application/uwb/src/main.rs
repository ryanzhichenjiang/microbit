#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_halt as _;

use core::str;
use cortex_m_rt::entry;
use defmt::info;

use microbit::{hal::{uarte, twim, gpio::Level, Timer}, Board};
use microbit_cutebot::{Cutebot, Rgb};

#[entry]
fn main() -> ! {
    if let Some(board) = Board::take() {
        let i2c_pins = board.i2c_internal;
        let mut timer = Timer::new(board.TIMER0);
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

        /* Set RGB LEDs: left to red, right to blue */
        match cutebot.rgb_left(Rgb::RED) {
            Ok(_) => {
                info!("Left RGB LED set to RED successfully");
                let _ = write_uart0(&uart0, "Left RGB LED initialized\r\n");
            }
            Err(_) => {
                info!("Failed to set left RGB LED");
                let _ = write_uart0(&uart0, "Error: Failed to set left RGB LED\r\n");
            }
        }

        match cutebot.rgb_right(Rgb::BLUE) {
            Ok(_) => {
                info!("Right RGB LED set to BLUE successfully");
                let _ = write_uart0(&uart0, "Right RGB LED initialized\r\n");
            }
            Err(_) => {
                info!("Failed to set right RGB LED");
                let _ = write_uart0(&uart0, "Error: Failed to set right RGB LED\r\n");
            }
        }



        // Buffer to store incoming UART data
        let mut buffer = [0u8; 32];
        let mut buffer_idx = 0;
        
        // Target distance and angle values
        let target_distance = 150; // 150 cm
        let target_angle = 0; // 0 degrees (straight ahead)
        
        // Tolerance values
        let angle_tolerance = 2; // ±2 degrees
        let distance_tolerance_percent = 10; // 10% of target distance
        
        /* Print initial status */
        info!("Ready to receive distance and angle data");
        info!("Format: D: XXXX, A: YYY");
        
        /* Endless loop */
        loop {
            // Check if there's data available to read
            if uart0.events_rxdrdy.read().bits() != 0 {
                // Reset the event
                uart0.events_rxdrdy.write(|w| unsafe { w.bits(0) });
                
                // Read one byte
                let c = uart0.rxd.read().bits() as u8;
                
                // Echo the character back
                let _ = write_uart0(&uart0, unsafe { str::from_utf8_unchecked(&[c; 1]) });
                
                // Add to buffer if not full and not a newline
                if c != b'\n' && buffer_idx < buffer.len() - 1 {
                    buffer[buffer_idx] = c;
                    buffer_idx += 1;
                } else {
                    // Null-terminate the buffer
                    buffer[buffer_idx] = 0;
                    
                    // Process the complete line
                    if buffer_idx > 0 {
                        // Convert buffer to string slice for easier parsing
                        if let Ok(data_str) = str::from_utf8(&buffer[0..buffer_idx]) {
                            // Try to parse distance and angle from the data
                            if let Some((distance, angle)) = parse_distance_angle(data_str) {
                                // Log the parsed values
                                info!("Received - Distance: {}, Angle: {}", distance, angle);
                                
                                // Calculate distance difference percentage
                                let distance_diff_percent = if target_distance > 0 {
                                    ((distance as i32 - target_distance as i32).abs() * 100) / target_distance as i32
                                } else {
                                    100 // If target is 0, any difference is 100%
                                };
                                
                                // Calculate angle difference
                                let angle_diff = (angle - target_angle).abs();
                                
                                // Determine motor speeds based on angle
                                if angle_diff <= angle_tolerance && distance_diff_percent <= distance_tolerance_percent {
                                    // We're at the target position, stop motors
                                    match cutebot.motors(0, 0) {
                                        Ok(_) => {
                                            info!("Target reached! Motors stopped");
                                            let _ = write_uart0(&uart0, "Target reached! Motors stopped\r\n");
                                        }
                                        Err(_) => {
                                            info!("Failed to stop motors");
                                            let _ = write_uart0(&uart0, "Error: Failed to stop motors\r\n");
                                        }
                                    }
                                } else {
                                    // We need to adjust position
                                    // Calculate motor speeds based on angle
                                    let (left_speed, right_speed) = calculate_motor_speeds(angle);
                                    
                                    // Apply motor speeds
                                    match cutebot.motors(left_speed, right_speed) {
                                        Ok(_) => {
                                            info!("Motors adjusted - Left: {}, Right: {}", left_speed, right_speed);
                                        }
                                        Err(_) => {
                                            info!("Failed to adjust motors");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Reset buffer for next line
                    buffer_idx = 0;
                }
            }
            
            // Small delay to prevent CPU hogging
            // timer.delay_us(10000_u32);
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

/// Parse distance and angle from a string in format "D: XXXX, A: YYY"
/// Returns (distance, angle) as (u16, i16) if parsing is successful
fn parse_distance_angle(data: &str) -> Option<(u16, i16)> {
    // Look for distance pattern "D: XXXX"
    let d_pos = data.find("D:")?
        .checked_add(3)?; // Skip "D: "
    
    // Look for angle pattern "A: YYY"
    let a_pos = data.find("A:")?
        .checked_add(3)?; // Skip "A: "
    
    // Extract distance substring
    let d_end = data[d_pos..].find(',').map_or(data.len(), |pos| d_pos + pos);
    let d_str = data[d_pos..d_end].trim();
    
    // Extract angle substring
    let a_end = data[a_pos..].find(',').map_or(data.len(), |pos| a_pos + pos);
    let a_str = data[a_pos..a_end].trim();
    
    // Parse distance and angle as numbers
    let distance = d_str.parse::<u16>().ok()?;
    let angle = a_str.parse::<i16>().ok()?;
    
    Some((distance, angle))
}

/// Calculate motor speeds based on the angle
/// Returns (left_speed, right_speed) as (i8, i8)
fn calculate_motor_speeds(angle: i16) -> (i8, i8) {
    // Base speed for forward movement
    let base_speed = 20;
    
    // Calculate speed difference based on angle
    // Positive angle means target is to the right, so right motor should be slower
    // Negative angle means target is to the left, so left motor should be slower
    let angle_factor = (angle as f32 / 45.0).clamp(-1.0, 1.0); // Normalize to -1.0 to 1.0
    
    // Calculate motor speeds
    if angle > 0 {
        // Target is to the right, slow down right motor
        let right_factor = 1.0 - angle_factor.abs();
        let left_speed = base_speed;
        let right_speed = (base_speed as f32 * right_factor) as i8;
        (left_speed, right_speed)
    } else if angle < 0 {
        // Target is to the left, slow down left motor
        let left_factor = 1.0 - angle_factor.abs();
        let left_speed = (base_speed as f32 * left_factor) as i8;
        let right_speed = base_speed;
        (left_speed, right_speed)
    } else {
        // Angle is 0, go straight
        (base_speed, base_speed)
    }
}
