# ftdi-rust

An attempt to port FDTI c library version into rust lang.

## Driver installation
### Linux
You can watch video **Linux d2xx Driver Installation Guide**
https://www.youtube.com/watch?v=jynlynjOOek
Download appropriate driver archive from - https://www.ftdichip.com/Drivers/D2XX.htm
and follow included installation instructions in ReadMe.txt

### Remarks
The FT2232H device has two independent ports, both of which can be configured using MPSSE while only Channel A and B of FT4232H can be configured using MPSSE. Using MPSSE can simplify the synchronous serial protocol (USB to SPI, I2C, JTAG, etc.) design.


