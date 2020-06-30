# ftdi-rust

An attempt to port FDTI c library version into rust lang.

### Remarks
The FT2232H device has two independent ports, both of which can be configured using MPSSE while only Channel A and B of FT4232H can be configured using MPSSE. Using MPSSE can simplify the synchronous serial protocol (USB to SPI, I2C, JTAG, etc.) design.
