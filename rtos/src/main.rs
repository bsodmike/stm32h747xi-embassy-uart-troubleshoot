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
    config.baudrate = 115200;
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
