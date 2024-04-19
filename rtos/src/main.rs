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
    // Refer to resources/Arduino_GIGA_R1_pins.xlsx for FMC pin config.
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
    usart1: USART1Resource {
        peri: UART8,
        tx: PJ8,        // UART8 tx
        rx: PJ9,        // UART8 rx
        tx_dma: DMA2_CH0,
        rx_dma: DMA2_CH1,
        rtc_power_key: PG10,
        wifi_reset: PH15,
    },
}

bind_interrupts!(struct USART1Irqs {
    UART8 => usart::InterruptHandler<peripherals::UART8>;
});

pub fn init() -> (embassy_stm32::Peripherals, cortex_m::Peripherals) {
    info!("Initialising power stage...");

    let config = embassy_stm32::Config::default();
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
    config.baudrate = 9600;
    let mut usart = defmt::unwrap!(embassy_stm32::usart::Uart::new(
        r.peri, r.rx, r.tx, USART1Irqs, NoDma, NoDma, config
    ));

    unwrap!(usart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.blocking_read(&mut buf));
        unwrap!(usart.blocking_write(&buf));
    }
}
