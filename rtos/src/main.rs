#![no_std]
#![no_main]

use assign_resources::assign_resources;
use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{self, Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "use_alloc")]
mod mem;

assign_resources! {
    usart1: USART1Resource {
        peri: USART6,
        tx: PG14,        // USART2_RX
        rx: PG9,        // USART2_TX
        tx_dma: DMA2_CH0,
        rx_dma: DMA2_CH1,
        rtc_power_key: PG10,
        wifi_reset: PH15,
    },
}

// NOTE: Only needed for testing with DMA
// bind_interrupts!(struct USART1Irqs {
//     UART8 => usart::InterruptHandler<peripherals::UART8>;
// });

pub fn init() -> (embassy_stm32::Peripherals, cortex_m::Peripherals) {
    info!("Initialising power stage...");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        // config.rcc.supply_config = SupplyConfig::LDO;
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

        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale0;

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

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
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
        unwrap!(usart.blocking_write(b"Hello\r\n"));
        info!("[{}]: wrote Hello, starting echo", &i);
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
