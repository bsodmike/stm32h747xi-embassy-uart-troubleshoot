MEMORY
{
    FLASH1   (rx) : ORIGIN = 0x08000000, LENGTH = 1M
    FLASH2   (rx) : ORIGIN = 0x08100000, LENGTH = 1M

    DTCM     (rw) : ORIGIN = 0x20000000, LENGTH = 128K
    ITCM    (rxw) : ORIGIN = 0x00000000, LENGTH = 64K
    /* Use AXISRAM as application 'ROM' */
    AXISRAM (rxw) : ORIGIN = 0x24000000, LENGTH = 512K
    /* SRAM1, SRAM2 and SRAM 3 are contiguous */
    SRAM1   (rxw) : ORIGIN = 0x30000000, LENGTH = 128K
    SRAM2   (rxw) : ORIGIN = 0x30020000, LENGTH = 128K
    SRAM3   (rxw) : ORIGIN = 0x30040000, LENGTH = 32K
                                      /* TOTAL = 288K */
    SRAM4    (rw) : ORIGIN = 0x38000000, LENGTH = 64K
    BSRAM    (rw) : ORIGIN = 0x38800000, LENGTH = 4K
}

REGION_ALIAS(FLASH, FLASH1);
REGION_ALIAS(RAM,   DTCM);

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
/*_cpu2_stack_start = ORIGIN(SRAM2) + LENGTH(SRAM2);*/

_ram_start = ORIGIN(RAM);
_ram_end = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS {
  /*.flash2 : ALIGN(4) {
    LONG(_cpu2_stack_start);
    KEEP(*(.flash2.reset_vector));
    KEEP(*(.flash2.vector_table));
    *(.flash2 .flash2.*);
    . = ALIGN(4);
    } > FLASH2*/

  .itcm : ALIGN(8) {
    *(.itcm .itcm.*);
    . = ALIGN(8);
    } > ITCM AT > FLASH

  .axisram (NOLOAD) : ALIGN(8) {
    *(.axisram .axisram.*);
    . = ALIGN(8);
    } > AXISRAM

  .sram1 (NOLOAD) : ALIGN(4) {
    *(.sram1 .sram1.*);
    . = ALIGN(4);
    } > SRAM1

  .sram2 (NOLOAD) : ALIGN(4) {
    *(.sram2 .sram2.*);
    . = ALIGN(4);
    } > SRAM2

  .sram3 (NOLOAD) : ALIGN(4) {
    *(.sram3 .sram3.*);
    . = ALIGN(4);
    } > SRAM3

  .sram4 (NOLOAD) : ALIGN(4) {
    *(.sram4 .sram4.*);
    . = ALIGN(4);
    } > SRAM4

  .bsram (NOLOAD) : ALIGN(4) {
    *(.bsram .bsram.*);
    . = ALIGN(4);
    } > BSRAM

}
