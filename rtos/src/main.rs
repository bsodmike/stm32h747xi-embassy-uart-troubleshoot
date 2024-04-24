#![no_std]
#![no_main]

use crate::{
    board::{
        config_portenta_giga_r1_wifi_leds, AssignedResources, FMCResources, GigaR1WifiBoardLeds,
        LedState, USART1Resource,
    },
    utils::interrupt_free,
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::cell::RefCell;
#[allow(unused_imports)]
use embassy_executor::{Executor, Spawner};
use embassy_stm32::usart::{BufferedUart, BufferedUartRx};
#[allow(unused_imports)]
use embassy_stm32::{
    bind_interrupts,
    gpio::Output,
    peripherals,
    usart::{self, Config, Uart},
    wdg,
};
use embassy_time::{Delay, Timer};
use embedded_io_async::{Read, Write};
use once_cell::sync::Lazy;
use static_cell::StaticCell;

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

extern crate alloc;

#[macro_use]
mod board;
#[cfg(feature = "use_alloc")]
mod mem;

pub static LED_RED: critical_section::Mutex<RefCell<Option<Output<'_>>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub static LED_GREEN: critical_section::Mutex<RefCell<Option<Output<'_>>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub static LED_BLUE: critical_section::Mutex<RefCell<Option<Output<'_>>>> =
    critical_section::Mutex::new(RefCell::new(None));

macro_rules! set_led (
        ($($fn_name:ident => ($mutex:expr)),+ $(,)*) => {
            $(
                pub fn $fn_name(state: LedState) {
                    match state {
                        LedState::On => interrupt_free(|cs| {
                            if let Some(pin) = &mut *$mutex.borrow_ref_mut(cs) {
                                pin.set_low()
                            };
                        }),
                        LedState::Off => interrupt_free(|cs| {
                            if let Some(pin) = &mut *$mutex.borrow_ref_mut(cs) {
                                pin.set_high()
                            };
                        }),
                    };
                }
            )+
        }
    );

set_led!(
    set_blue_led => (LED_BLUE),
    set_green_led => (LED_GREEN),
    set_red_led => (LED_RED),
);

// NOTE: Only needed for testing with DMA
// bind_interrupts!(struct USART1Irqs {
//     USART1 => usart::InterruptHandler<peripherals::USART1>;
// });

// BufferedInterruptHandler
bind_interrupts!(struct USART1Irqs {
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
});

pub const USART_BAUD: u32 = 115200;
pub const USART_READ_BUF_SIZE: usize = 32;
pub static MESSAGE: critical_section::Mutex<RefCell<Option<String>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub static TEMP_BUF: Lazy<critical_section::Mutex<RefCell<Box<[u8; 8]>>>> =
    Lazy::new(|| critical_section::Mutex::new(RefCell::new(Box::new([0u8; 8]))));

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("main()");
    let (p, mut core_peri) = board::init();
    let r = split_resources!(p);
    // FMC
    mem::init_sdram(r.fmc, &mut core_peri);

    // Configure LEDs
    #[cfg(feature = "board_giga_r1_wifi")]
    {
        config_portenta_giga_r1_wifi_leds(r.giga_r1_wifi_board_leds);
    }

    interrupt_free(|cs| {
        let buf = [0u8; 8];
        let message = String::default();

        MESSAGE.borrow(cs).replace(Some(message));
        let _ = *TEMP_BUF.borrow(cs).replace(Box::new(buf));
    });

    unwrap!(spawner.spawn(heatbeat_task()));
    // unwrap!(spawner.spawn(usart_task(r.usart1)));

    let (tx_pin, rx_pin, uart) = (r.usart1.tx, r.usart1.rx, r.usart1.peri);

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    let mut config = embassy_stm32::usart::Config::default();
    config.baudrate = USART_BAUD;
    let uart = BufferedUart::new(uart, USART1Irqs, rx_pin, tx_pin, tx_buf, rx_buf, config)
        .expect("Create UART");
    let (mut tx, rx) = uart.split();

    unwrap!(spawner.spawn(buffered_uart_reader(rx)));

    info!("Writing...");
    loop {
        let data = b"ATB\r\n";
        info!("TX {:?}", data);
        tx.write_all(data).await.unwrap();
        Timer::after_secs(1).await;
    }
}

#[allow(dead_code)]
#[embassy_executor::task]
async fn buffered_uart_reader(mut rx: BufferedUartRx<'static, embassy_stm32::peripherals::USART1>) {
    info!("Reading...");

    const LEADER: char = '+';
    const CR: char = '\r';
    const LF: char = '\n';
    // \+([\w\:\s\-\_\,]+)\n

    loop {
        let mut buf = [0; USART_READ_BUF_SIZE];

        rx.read_exact(&mut buf).await.unwrap();
        match String::from_utf8(buf.to_vec()) {
            Ok(response) => {
                info!("Received response (Bytes): {}", &buf);
                warn!("Received response: {}", &response.as_str());

                let mut words: alloc::vec::Vec<String> = alloc::vec::Vec::default();
                for character in response.chars().into_iter() {
                    // if character != CR && character != LF {
                    //     let s = character.to_string();
                    //     words.push(s);
                    // }

                    // if character == LF {
                    //     words.push(String::from("\0"));
                    // }
                }

                // FIXME this strips out the double \0\0 padding (converted from \r\n)
                // let intermediary = words.concat();
                // let mut words: alloc::vec::Vec<String> = alloc::vec::Vec::default();
                // for el in intermediary.chars().into_iter() {
                //     if el != '\0' {
                //         words.push(el.to_string());
                //     }
                // }

                let full_message = words.concat();

                // FIXME need to only take sub-strings that start and end between a '+' sign.
                // let v: alloc::vec::Vec<&str> = full_message.split('+').collect();
                // let full_message = v.concat();
                warn!("Received words: {}", full_message.as_str());
                info!("Received words: {}", full_message.as_bytes());
            }
            Err(_e) => {
                defmt::unimplemented!()
            }
        }
    }
}

#[embassy_executor::task]
pub async fn usart_task(r: USART1Resource) {
    info!("Running task: usart_task");

    let mut config = embassy_stm32::usart::Config::default();
    config.baudrate = USART_BAUD;
    let mut usart = defmt::unwrap!(embassy_stm32::usart::Uart::new_blocking(
        r.peri, r.rx, r.tx, config
    ));

    // write once
    unwrap!(usart.blocking_write(b"ATB\r\n"));
    unwrap!(usart.blocking_flush());
    debug!("usart_task: Completed blocking write");

    loop {
        let mut received_message = false;

        interrupt_free(|cs| {
            let buf_refmut = TEMP_BUF.borrow(cs);
            let mut new_buf: [u8; 8] = [0u8; 8];
            if let Err(e) = usart.blocking_read(&mut new_buf[..]) {
                error!("usart read error: {}", e);
            }

            buf_refmut.replace(Box::new(new_buf));
        });

        info!("blocking read completed");
        {
            interrupt_free(|cs| {
                let buf = **&mut *TEMP_BUF.borrow_ref_mut(cs);
                let new_buf: alloc::vec::Vec<u8> = buf.iter().map(|&el| el).collect();
                info!("buf: {}", &buf);
                info!("new_buf len: {}", &new_buf.len());

                let message_opt = &mut *MESSAGE.borrow_ref_mut(cs);
                let rx_char = String::from_utf8(new_buf).expect("Create string");

                if let Some(message) = message_opt {
                    info!("rx_char: {=str}", &rx_char);
                    debug!("rx_char len: {}", &rx_char.len());
                    match &rx_char[6..] {
                        "\r\n" => {
                            trace!("Found message termination");

                            let rx_bytes = rx_char.as_bytes();

                            debug!("This: {}", rx_bytes[5..6][0]);

                            let mut chars = rx_char[5..6].chars();
                            let _ = message.push(chars.next().expect("Take char"));
                            debug!("message size: {}", message.len());

                            debug!("5..6: {}", &rx_char[5..6]);
                            warn!("equal? {}", rx_char[5..6] == *"\0");
                            if rx_char[5..6] == *"\0" {
                                warn!("set received_message = true");
                                received_message = true;
                            }
                        }
                        _ => error!("Unable to locate end of message!"),
                    }
                };
            });
        }

        interrupt_free(|cs| {
            let message_opt = &mut *MESSAGE.borrow_ref_mut(cs);

            if let Some(message) = message_opt {
                trace!("Received message (Bytes): {}", &message.as_bytes());
            }
        });

        #[allow(unused_assignments)]
        if received_message {
            interrupt_free(|cs| {
                {
                    let message_opt = &mut *MESSAGE.borrow_ref_mut(cs);
                    if let Some(message) = message_opt {
                        warn!("Received message: {}", &message.as_str());
                    }
                }

                MESSAGE.borrow(cs).replace(Some(String::default()));
                warn!("Cleared Message buffer");
            });

            received_message = false;
        }
    }
}

#[embassy_executor::task]
pub async fn heatbeat_task() {
    let mut counter: u32 = 0;
    info!("Running task: heatbeat_task");

    loop {
        // trace!("BLUE led off");
        set_blue_led(LedState::Off);
        Timer::after_millis(50).await;

        // info!("BLUE led on");
        set_blue_led(LedState::On);
        Timer::after_millis(50).await;

        // HEARTBEAT_SIGNAL.signal(counter);
        counter = counter.wrapping_add(1);
    }
}

mod utils {
    use critical_section::CriticalSection;

    #[inline(always)]
    pub fn interrupt_free<F, R>(f: F) -> R
    where
        F: FnOnce(CriticalSection) -> R,
    {
        critical_section::with(f)
    }
}
