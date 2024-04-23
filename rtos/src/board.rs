use crate::{utils::interrupt_free, LED_BLUE, LED_GREEN, LED_RED};
use assign_resources::assign_resources;
#[allow(unused_imports)]
use defmt::{debug, info, trace};
use embassy_stm32::gpio::{Level, Output, Pin, Speed};
use embassy_stm32::peripherals;

assign_resources! {
    // Refer to resources/Arduino_GIGA_R1_pins.xlsx for FMC pin config.
    // This also works on the Portenta H7
    fmc: FMCResources {
        fmc: FMC,
        a0: PF0,        // A0
        a1: PF1,
        a2: PF2,
        a3: PF3,
        a4: PF4,
        a5: PF5,
        a6: PF12,
        a7: PF13,
        a8: PF14,
        a9: PF15,       // A9
        a10: PG0,
        a11: PG1,       // A11
        ba0: PG4,       // BA0
        ba1: PG5,       // BA1
        d0: PD14,       // D0
        d1: PD15,       // D1
        d2: PD0,        // D2
        d3: PD1,        // D3
        d4: PE7,
        d5: PE8,
        d6: PE9,
        d7: PE10,
        d8: PE11,
        d9: PE12,
        d10: PE13,
        d11: PE14,
        d12: PE15,
        d13: PD8,
        d14: PD9,
        d15: PD10,      // D15
        nbl0: PE0,      // NBL0
        nbl1: PE1,      // NBL1
        sdcke0: PH2,    // SDCKE0
        sdclk: PG8,     // SDCLK
        sdncas: PG15,   // SDNCAS
        sdne0: PH3,     // SDNE0
        sdnras: PF11,   // SDNRAS
        sdnwe: PH5,     // SDNWE
    },

    // Portenta H7
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

// FIXME needs testing
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
