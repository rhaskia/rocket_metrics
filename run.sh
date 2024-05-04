cargo build --release
avr-objcopy -j .text -j .data -O ihex "target\avr-atmega328p\release\metrics.elf" sketch.ino.hex
avrdude -p atmega328p -F -P COM5 -c arduino -U flash:w:sketch.ino.hex
