#![no_std]
#![no_main]

use crate::board::config_portenta_giga_r1_wifi_leds;
use assign_resources::assign_resources;
use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{self, Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_stm32::{gpio::Output, wdg};
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

use crate::board::LedState;
use crate::utils::interrupt_free;
use core::cell::RefCell;

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

assign_resources! {
    // usart1: USART1Resource {
    //     peri: UART8,
    //     tx: PJ8,            // UART3 tx
    //     rx: PJ9,            // UART3 rx
    //     tx_dma: DMA2_CH0,
    //     rx_dma: DMA2_CH1,
    //     rtc_power_key: PG10,
    //     wifi_reset: PH15,
    // },

    // GIGA R1 WiFI
    usart1: USART1Resource {
        peri: USART1,
        tx: PA9,            // USART1 tx
        rx: PB7,            // USART1 rx
        tx_dma: DMA2_CH0,
        rx_dma: DMA2_CH1,
        rtc_power_key: PG10,
    },
    giga_r1_wifi_board_leds: GigaR1WifiBoardLeds {
        red: PI12,
        green: PJ13,
        blue: PE3,
    },
}

// NOTE: Only needed for testing with DMA
// bind_interrupts!(struct USART1Irqs {
//     UART8 => usart::InterruptHandler<peripherals::UART8>;
// });

// pub fn init() -> (embassy_stm32::Peripherals, cortex_m::Peripherals) {
//     info!("Initialising power stage...");

//     let mut config = embassy_stm32::Config::default();
//     {
//         use embassy_stm32::rcc::*;
//         config.rcc.supply_config = SupplyConfig::LDO;
//         config.rcc.hsi = Some(HSIPrescaler::DIV1); // // 64MHz
//         config.rcc.csi = true;
//         config.rcc.hsi48 = Some(Hsi48Config {
//             sync_from_usb: true,
//         }); // needed for USB

//         #[cfg(feature = "stm32h747_480")]
//         {
//             config.rcc.pll1 = Some(Pll {
//                 source: PllSource::HSI,
//                 prediv: PllPreDiv::DIV16,
//                 mul: PllMul::MUL240,      // Plln
//                 divp: Some(PllDiv::DIV2), // ((64/8)*120)/2 = 480MHz
//                 divq: Some(PllDiv::DIV2), // ((64/8)*120)/8 = 120MHz / SPI1 cksel defaults to pll1_q
//                 divr: Some(PllDiv::DIV2),
//             });
//             // config.rcc.pll2 = Some(Pll {
//             //     source: PllSource::HSI,
//             //     prediv: PllPreDiv::DIV8,
//             //     mul: PllMul::MUL50,
//             //     divp: Some(PllDiv::DIV4), // ((64/8)*50)/4 = 100MHz
//             //     divq: None,
//             //     divr: None,
//             // });
//         }

//         config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
//         config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
//         config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
//         config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
//         config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
//         config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
//         config.rcc.voltage_scale = VoltageScale::Scale0;

//         trace!(
//             "rcc.voltage_scale = Voltage::Scale{=i32}",
//             config.rcc.voltage_scale as i32
//         );
//     }

//     let p: embassy_stm32::Peripherals = embassy_stm32::init(config);
//     let core_peri = defmt::unwrap!(cortex_m::Peripherals::take());

//     (p, core_peri)
// }

// GIGA R1 WIFI
pub fn init() -> (embassy_stm32::Peripherals, cortex_m::Peripherals) {
    info!("Initialising power stage...");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.supply_config = SupplyConfig::LDO;
        config.rcc.hsi = Some(HSIPrescaler::DIV1); // // 64MHz
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        }); // needed for USB

        #[cfg(feature = "stm32h747_400")]
        {
            config.rcc.pll1 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL50,
                divp: Some(PllDiv::DIV2), // ((64/4)*50)/2 = 400MHz
                divq: Some(PllDiv::DIV8), // ((64/4)*50)/8 = 100MHz / SPI1 cksel defaults to pll1_q
                divr: None,
            });
            config.rcc.pll2 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV8,
                mul: PllMul::MUL50,
                divp: Some(PllDiv::DIV4), // ((64/8)*50)/4 = 100MHz
                divq: None,
                divr: None,
            });
        }
        #[cfg(feature = "stm32h747_480")]
        {
            config.rcc.pll1 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV8,
                mul: PllMul::MUL120,
                divp: Some(PllDiv::DIV2), // ((64/8)*120)/2 = 480MHz
                divq: Some(PllDiv::DIV8), // ((64/8)*120)/8 = 120MHz / SPI1 cksel defaults to pll1_q
                divr: None,
            });
            config.rcc.pll2 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV8,
                mul: PllMul::MUL50,
                divp: Some(PllDiv::DIV4), // ((64/8)*50)/4 = 100MHz
                divq: None,
                divr: None,
            });
        }
        #[cfg(feature = "stm32h747_slow")]
        {
            config.rcc.pll1 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV8,
                mul: PllMul::MUL120,
                divp: Some(PllDiv::DIV50), // ((64/8)*120)/50 = 19.2MHz
                divq: Some(PllDiv::DIV80), // ((64/8)*120)/8 = 12MHz / SPI1 cksel defaults to pll1_q
                divr: None,
            });
            config.rcc.pll2 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV8,
                mul: PllMul::MUL50,
                divp: Some(PllDiv::DIV4), // ((64/8)*50)/4 = 100MHz
                divq: None,
                divr: None,
            });
        }
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;

        let mut mux = embassy_stm32::rcc::mux::ClockMux::default();
        mux.adcsel = embassy_stm32::rcc::mux::Adcsel::PLL2_P;
        config.rcc.mux = mux;

        // RTC
        config.rcc.ls = LsConfig::default_lse();

        trace!(
            "rcc.voltage_scale = Voltage::Scale{=i32}",
            config.rcc.voltage_scale as i32
        );
    }

    let p: embassy_stm32::Peripherals = embassy_stm32::init(config);
    let core_peri = defmt::unwrap!(cortex_m::Peripherals::take());

    (p, core_peri)
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("main()");
    let (p, _core_peri) = init();
    let r = split_resources!(p);

    // Configure LEDs
    #[cfg(feature = "board_giga_r1_wifi")]
    {
        config_portenta_giga_r1_wifi_leds(r.giga_r1_wifi_board_leds);
    }

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(heatbeat_task()));
        unwrap!(spawner.spawn(usart_task(r.usart1)));
    })
}

const BUF_SIZE: usize = 2048;

#[embassy_executor::task]
pub async fn usart_task(r: USART1Resource) {
    info!("Running task: usart_task");

    let mut config = embassy_stm32::usart::Config::default();
    config.baudrate = 115200;
    let mut usart = defmt::unwrap!(embassy_stm32::usart::Uart::new_blocking(
        r.peri, r.rx, r.tx, config
    ));

    for i in 0..2 {
        unwrap!(usart.blocking_write(b"Hello from Rust\r\n"));
        info!("[{}]: wrote Hello, starting echo", &i);

        unwrap!(usart.blocking_flush());
    }

    let mut buf = [0u8; 1];
    loop {
        if let Err(e) = usart.blocking_read(&mut buf) {
            error!("usart read error: {}", e);
        }

        if let Err(_e) = usart.blocking_write(&buf) {
            //
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

#[macro_use]
mod board {
    use crate::{utils::interrupt_free, GigaR1WifiBoardLeds, LED_BLUE, LED_GREEN, LED_RED};
    use embassy_stm32::gpio::{Level, Output, Pin, Speed};

    pub enum LedState {
        On,
        Off,
    }

    #[allow(dead_code)]
    pub fn config_portenta_giga_r1_wifi_leds(leds: GigaR1WifiBoardLeds) {
        {
            interrupt_free(|cs| {
                let pin = leds.red.degrade();
                let mut led_red = Output::new(pin, Level::High, Speed::Low);
                let pin = leds.green.degrade();
                let mut led_green = Output::new(pin, Level::High, Speed::Low);
                let pin = leds.blue.degrade();
                let mut led_blue = Output::new(pin, Level::High, Speed::Low);

                led_red.set_high();
                led_green.set_high();
                led_blue.set_high();

                LED_RED.borrow(cs).replace(Some(led_red));
                LED_GREEN.borrow(cs).replace(Some(led_green));
                LED_BLUE.borrow(cs).replace(Some(led_blue));
            });
        }
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
