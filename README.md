# stm32h747xi-embassy-uart-troubleshoot

# Troubleshoot UART issues

Steps to replicate:
1. Click the reset button on the Portenta H7, this will cause the green LED to "throb" and pulsate.
2. Run `./upload r` to flash a Portenta H7
3. You can attach a Segger JLINK-mini and run the debugger for the release config

Refer to `issues/STM32h7hxi Embassy UART Issues.pdf`.

```
0.000000 INFO  Initialising power stage...
└─ stm32_rtos::init @ rtos/src/main.rs:101 
0.000000 TRACE rcc.voltage_scale = Voltage::Scale0
└─ stm32_rtos::init @ rtos/src/main.rs:185 
0.000000 DEBUG flash: latency=4 wrhighfreq=2
└─ embassy_stm32::rcc::_version::flash_setup @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/49807c0/embassy-stm32/src/fmt.rs:130 
0.000000 TRACE BDCR configured: 00008113
└─ embassy_stm32::rcc::bd::{impl#3}::init::{closure#4} @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/49807c0/embassy-stm32/src/fmt.rs:117 
0.000000 DEBUG rcc: Clocks { csi: Some(Hertz(4000000)), hclk1: Some(Hertz(240000000)), hclk2: Some(Hertz(240000000)), hclk3: Some(Hertz(240000000)), hclk4: Some(Hertz(240000000)), hse: None, hsi: Some(Hertz(64000000)), hsi48: Some(Hertz(48000000)), i2s_ckin: None, lse: None, lsi: None, pclk1: Some(Hertz(120000000)), pclk1_tim: Some(Hertz(240000000)), pclk2: Some(Hertz(120000000)), pclk2_tim: Some(Hertz(240000000)), pclk3: Some(Hertz(120000000)), pclk4: Some(Hertz(120000000)), pll1_q: Some(Hertz(120000000)), pll2_p: Some(Hertz(100000000)), pll2_q: None, pll2_r: None, pll3_p: None, pll3_q: None, pll3_r: None, rtc: Some(Hertz(32768)), sys: Some(Hertz(480000000)) }
└─ embassy_stm32::rcc::set_freqs @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/49807c0/embassy-stm32/src/fmt.rs:130 
0.000000 INFO  SDRAM Memory Size 0x18
└─ stm32_rtos::mem::init_sdram::log2minus1 @ rtos/src/mem/mod.rs:68  
0.000213 INFO  Bootup completed...
└─ stm32_rtos::____embassy_main_task::{async_fn#0} @ rtos/src/main.rs:204 
0.000213 INFO  Running task: wifi_task
└─ stm32_rtos::__wifi_task_task::{async_fn#0} @ rtos/src/main.rs:225 
0.000213 TRACE USART: presc=1, div=0x000030d4 (mantissa = 781, fraction = 4)
└─ embassy_stm32::usart::configure @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/49807c0/embassy-stm32/src/fmt.rs:117 
0.000213 TRACE Using 16 bit oversampling, desired baudrate: 9600, actual baudrate: 9600
└─ embassy_stm32::usart::configure @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/49807c0/embassy-stm32/src/fmt.rs:117 
0.006469 INFO  wrote Hello, starting echo
└─ stm32_rtos::__wifi_task_task::{async_fn#0} @ rtos/src/main.rs:234 
0.392578 ERROR panicked at 'unwrap failed: usart.blocking_read(& mut buf)'
error: `Noise`
└─ stm32_rtos::__wifi_task_task::{async_fn#0} @ rtos/src/main.rs:238 
0.392578 ERROR panicked at /Users/mdesilva/.cargo/registry/src/index.crates.io-6f17d22bba15001f/defmt-0.3.6/src/lib.rs:368:5:
explicit panic
└─ panic_probe::print_defmt::print @ /Users/mdesilva/.cargo/registry/src/index.crates.io-6f17d22bba15001f/panic-probe-0.3.1/src/lib.rs:104 
```

# Notes

This configures the [Arduino GIGA R1 WiFI board](https://docs.arduino.cc/hardware/giga-r1-wifi/).


- [STM32H747XI Datasheet](https://www.st.com/resource/en/datasheet/stm32h747ai.pdf)
- [RM0399: STM32H745/755 and STM32H747/757 advanced Arm®-based 32-bit MCUs](https://www.st.com/content/ccc/resource/technical/document/reference_manual/group0/82/40/1a/07/c9/16/40/35/DM00176879/files/DM00176879.pdf/jcr:content/translations/en.DM00176879.pdf)

With RTT enabled and `probe-rs` a typical bootup sequence will look like:
```
0.000000 DEBUG Voltage::Scale0
└─ stm32h747_async_quickstart::____embassy_main_task::{async_fn#0} @ src/main.rs:101 
0.000000 DEBUG flash: latency=4 wrhighfreq=2
└─ embassy_stm32::rcc::_version::flash_setup @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/4c7ed5e/embassy-stm32/src/fmt.rs:130 
0.000000 TRACE BDCR ok: 00008113
└─ embassy_stm32::rcc::bd::{impl#3}::init @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/4c7ed5e/embassy-stm32/src/fmt.rs:117 
0.000000 DEBUG rcc: Clocks { csi: Some(Hertz(4000000)), hclk1: Some(Hertz(240000000)), hclk2: Some(Hertz(240000000)), hclk3: Some(Hertz(240000000)), hclk4: Some(Hertz(240000000)), hse: None, hsi: Some(Hertz(64000000)), hsi48: Some(Hertz(48000000)), i2s_ckin: None, lse: None, lsi: None, pclk1: Some(Hertz(120000000)), pclk1_tim: Some(Hertz(240000000)), pclk2: Some(Hertz(120000000)), pclk2_tim: Some(Hertz(240000000)), pclk3: Some(Hertz(120000000)), pclk4: Some(Hertz(120000000)), per: None, pll1_q: Some(Hertz(120000000)), pll2_p: Some(Hertz(100000000)), pll2_q: None, pll2_r: None, pll3_p: None, pll3_q: None, pll3_r: None, rtc: Some(Hertz(32768)), sys: Some(Hertz(480000000)) }
└─ embassy_stm32::rcc::set_freqs @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/4c7ed5e/embassy-stm32/src/fmt.rs:130 
0.000000 INFO  Booting up...
└─ stm32h747_async_quickstart::____embassy_main_task::{async_fn#0} @ src/main.rs:105 
0.000000 INFO  ADC frequency set to 25000000 Hz
└─ embassy_stm32::adc::_version::{impl#7}::new @ /Users/mdesilva/.cargo/git/checkouts/embassy-9312dcb0ed774b29/4c7ed5e/embassy-stm32/src/fmt.rs:143 
0.017242 INFO  Got RTC! 1707983285
└─ stm32h747_async_quickstart::____embassy_main_task::{async_fn#0} @ src/main.rs:147 
0.017272 INFO  SDRAM Memory Size 0x18
└─ stm32h747_async_quickstart::mem::init_sdram::log2minus1 @ src/mem/mod.rs:65  
```

## Setup

Run `./upload.sh` for help menu and `./upload.sh f` to build and flash the board.

```
Usage: ./upload [COMMAND]

Commands:
	 build, b	Build the crate at ./h7-cm7
	 flash, f	Flash the output binary to the board
	 run, r		Build and flash
```

## Minimum supported Rust version

The Minimum Supported Rust Version (MSRV) at the moment is rustc **1.77.0-beta.3**.

## License

Refer to LICENSE.
