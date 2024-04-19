use crate::Delay;
use crate::FMCResources;
use cortex_m_alloc::Heap;
use defmt::info;
use embassy_stm32::fmc::Fmc;

// Heap allocator
#[global_allocator]
pub static ALLOCATOR: Heap = Heap::empty();

pub const HEAP_SIZE: usize = 32 * 1024 * 1024;

pub fn init_sdram(r: FMCResources, core_peri: &mut cortex_m::Peripherals) {
    // taken from stm32h7xx-hal
    core_peri.SCB.enable_icache();
    // See Errata Sheet 2.2.1
    // core_peri.SCB.enable_dcache(&mut core_peri.CPUID);
    core_peri.DWT.enable_cycle_counter();
    // ----------------------------------------------------------
    // Configure MPU for external SDRAM
    // MPU config for SDRAM write-through
    {
        let mpu = &core_peri.MPU;
        let scb = &mut core_peri.SCB;
        let size = HEAP_SIZE;

        // Refer to ARM®v7-M Architecture Reference Manual ARM DDI 0403
        // Version E.b Section B3.5
        const MEMFAULTENA: u32 = 1 << 16;

        unsafe {
            /* Make sure outstanding transfers are done */
            cortex_m::asm::dmb();

            scb.shcsr.modify(|r| r & !MEMFAULTENA);

            /* Disable the MPU and clear the control register*/
            mpu.ctrl.write(0);
        }

        const REGION_NUMBER0: u32 = 0x00;
        const REGION_BASE_ADDRESS: u32 = 0xD000_0000;

        const REGION_FULL_ACCESS: u32 = 0x03;
        const REGION_CACHEABLE: u32 = 0x01;
        const REGION_WRITE_BACK: u32 = 0x01;
        const REGION_ENABLE: u32 = 0x01;

        crate::assert_eq!(
            size & (size - 1),
            0,
            "SDRAM memory region size must be a power of 2"
        );
        crate::assert_eq!(
            size & 0x1F,
            0,
            "SDRAM memory region size must be 32 bytes or more"
        );
        fn log2minus1(sz: u32) -> u32 {
            for i in 5..=31 {
                if sz == (1 << i) {
                    return i - 1;
                }
            }
            crate::panic!("Unknown SDRAM memory region size!");
        }

        info!("SDRAM Memory Size 0x{:x}", log2minus1(size as u32));

        // Configure region 0
        //
        // Cacheable, outer and inner write-back, no write allocate. So
        // reads are cached, but writes always write all the way to SDRAM
        unsafe {
            mpu.rnr.write(REGION_NUMBER0);
            mpu.rbar.write(REGION_BASE_ADDRESS);
            mpu.rasr.write(
                (REGION_FULL_ACCESS << 24)
                    | (REGION_CACHEABLE << 17)
                    | (REGION_WRITE_BACK << 16)
                    | (log2minus1(size as u32) << 1)
                    | REGION_ENABLE,
            );
        }

        const MPU_ENABLE: u32 = 0x01;
        const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

        // Enable
        unsafe {
            mpu.ctrl
                .modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

            scb.shcsr.modify(|r| r | MEMFAULTENA);

            // Ensure MPU settings take effect
            cortex_m::asm::dsb();
            cortex_m::asm::isb();
        }
    }

    let mut sdram =
    // Refer to resources/Arduino_GIGA_R1_pins.xlsx for FMC pin config.
    Fmc::sdram_a12bits_d16bits_4banks_bank1(
        r.fmc,
        r.a0, // A0
        r.a1,
        r.a2,
        r.a3,
        r.a4,
        r.a5,
        r.a6,
        r.a7,
        r.a8,
        r.a9, // A9
        r.a10,
        r.a11,  // A11
        r.ba0,  // BA0
        r.ba1,  // BA1
        r.d0, // D0
        r.d1, // D1
        r.d2,  // D2
        r.d3,  // D3
        r.d4,
        r.d5,
        r.d6,
        r.d7,
        r.d8,
        r.d9,
        r.d10,
        r.d11,
        r.d12,
        r.d13,
        r.d14,
        r.d15, // D15
        r.nbl0,  // NBL0
        r.nbl1,  // NBL1
        r.sdcke0,  //
        r.sdclk,
        r.sdncas, // SDCKE0
        r.sdne0,  // SDNE0
        r.sdnras, // SDNRAS
        r.sdnwe,  // SDNWE
        stm32_fmc::devices::as4c4m16sa_6::As4c4m16sa {},
    );

    // Initialise controller and SDRAM
    let mut delay = Delay;
    let ram_ptr: *mut u32 = sdram.init(&mut delay) as *mut _;
    // NOTE: for testing purposes only.
    // mem::check_sdram(ram_ptr, sdram_size);

    unsafe {
        ALLOCATOR.init(ram_ptr as usize, HEAP_SIZE);
    }
}

#[allow(dead_code)]
pub fn check_sdram(ram_ptr: *mut u32, sdram_size: usize) {
    let ram_slice = unsafe {
        // Convert raw pointer to slice
        core::slice::from_raw_parts_mut(ram_ptr, sdram_size / core::mem::size_of::<u32>())
    };

    // // ----------------------------------------------------------
    // // Use memory in SDRAM
    // ram_slice.fill(0); // NOTE this is optional.
    info!("RAM contents before writing: {:x}", ram_slice[..10]);

    ram_slice[0] = 1;
    ram_slice[1] = 2;
    ram_slice[2] = 3;
    ram_slice[3] = 4;

    info!("RAM contents after writing: {:x}", ram_slice[..10]);

    crate::assert_eq!(ram_slice[0], 1);
    crate::assert_eq!(ram_slice[1], 2);
    crate::assert_eq!(ram_slice[2], 3);
    crate::assert_eq!(ram_slice[3], 4);

    info!("Assertions succeeded.");
}
