avr-objcopy -j .text -j .data -O ihex target\avr-atmega328p\debug\metrics.elf sketch.ino.hex
avrdude -p atmega328p -F -P COM5 -c arduino -U flash:w:sketch.ino.hex
