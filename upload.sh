#!/bin/bash 

set -e
RELEASE=release
BOARD="portenta_h7"

if [ ! -z $1 ]; then
    # Handle bootloader start address for Portenta H7
    if [ ${BOARD} == "portenta_h7" ]; then
      sed -i '' -e 's/0x08000000/0x08040000/g' memory.x
    else
      sed -i '' -e 's/0x08040000/0x08000000/g' memory.x
    fi

    if [ $1 == "build" ] || [ $1 == "run" ] || [ $1 == "b" ] || [ $1 == "r" ]; then
        if [ ${RELEASE} == "release" ]; then
            # ./test_mac.sh && cargo b --release
            cargo b --release

        else
            ./test_mac.sh && cargo b
        fi
    fi
else
    echo "Usage: ./upload [COMMAND]"
    echo ""
    echo "Commands:"
    echo -e '\t build, b\tBuild the crate'
    echo -e '\t flash, f\tFlash the output binary to the board'
    echo -e '\t run, r\t\tBuild and flash'
fi

TARGET_DIR=$(pwd)
RELEASE_DIR=${TARGET_DIR}/target/thumbv7em-none-eabihf/${RELEASE}
CRATE_NAME=stm32-rtos

if [ ! -z $1 ]; then
    if [ $1 == "flash" ] || [ $1 == "run" ] || [ $1 == "f" ] || [ $1 == "r" ]; then
        cd ${RELEASE_DIR}
        /opt/homebrew/bin/arm-none-eabi-objcopy \
            -v -O binary ${CRATE_NAME} ${CRATE_NAME}.bin
            
        size=`ls -la *.bin | awk '{ print $5 }'`
        sizeFriendly=`ls -lah *.bin | awk '{ print $5 }'`
        file=`ls -la *.bin | awk '{ print $9 }'`
        echo "Created ${file} with size: ${size} bytes (${sizeFriendly})"

        DFU_ID=$(dfu-util -l| grep "Found DFU" | head \
            -n 1 | awk '{ print $3 }' | sed 's/[][]//g')

        if [ ! -z ${DFU_ID} ]; then
            if [ ${DFU_ID} == "0483:df11" ]; then
                echo "Arduino GIGA R1 WiFi Board is now in DFU mode with ID 0483:df11"
                echo ""

                dfu-util -a 0 -d 0483:df11 -s 0x08000000 -D *.bin
            elif [ ${DFU_ID} == "2341:035b" ]; then
                echo "Arduino Portenta H7 Board is now in DFU mode with ID 2341:035b"
                echo ""

                dfu-util -a 0 -d 2341:035b -s 0x08040000 -D *.bin
            else
                echo "Board has an incorrect DFU ID: ${DFU_ID}"
                echo ""
            fi
        else
            echo "Board is not connected or not in DFU mode!!"
            echo ""
        fi
    fi

    if [ $1 == "restore" ]; then
        DFU_ID=$(dfu-util -l| grep "Found DFU" | head \
            -n 1 | awk '{ print $3 }' | sed 's/[][]//g')

        if [ ! -z ${DFU_ID} ]; then
            if [ ${DFU_ID} == "0483:df11" ]; then
                echo "Arduino GIGA R1 WiFi Board is now in DFU mode with ID 0483:df11"
                echo ""

                ~/Library/Arduino15/packages/arduino/tools/dfu-util/0.10.0-arduino1/dfu-util --device ,0x0483:0xdf11 -D ~/Library/Arduino15/packages/arduino/hardware/mbed_giga/4.1.1/bootloaders/GIGA/bootloader.bin -a0 --dfuse-address=0x08000000

                echo "Make sure you power cycle the board now!"
            elif [ ${DFU_ID} == "2341:035b" ]; then
                echo "Arduino Portenta H7 Board is now in DFU mode with ID 2341:035b"
                echo ""

                ~/Library/Arduino15/packages/arduino/tools/dfu-util/0.10.0-arduino1/dfu-util --device ,0x2341:0x035b -D ~/Library/Arduino15/packages/arduino/hardware/mbed_giga/4.1.1/bootloaders/GIGA/bootloader.bin -a0 --dfuse-address=0x08040000

                # FIXME: Check this is the correct bootloader?
                #                ~/Library/Arduino15/packages/arduino/hardware/mbed_portenta/4.1.1/bootloaders/PORTENTA_H7/portentah7_bootloader_mbed_hs_v2.bin

                echo "Make sure you power cycle the board now!"
            else
                echo "Board has an incorrect DFU ID: ${DFU_ID}"
                echo ""
            fi
        else
            echo "Board is not connected or not in DFU mode!!"
            echo ""
        fi
    fi

    if [ $1 == "detect" ]; then
        dfu-util -l
    fi
fi




