# ftdi-rust

An attempt to port FDTI c library version into rust lang.

## Driver installation
### Linux
You can watch video **Linux d2xx Driver Installation Guide**
https://www.youtube.com/watch?v=jynlynjOOek
Download appropriate driver archive from - https://www.ftdichip.com/Drivers/D2XX.htm
and follow included installation instructions in ReadMe.txt

## Build
### Linux
Here is a short tutorial on how to build from git under Ubuntu and other similar Linux distros.

#### Install the build tools
> sudo apt-get install build-essential

(yum install make automake gcc gcc-c++ kernel-devel)
> sudo apt-get install git-core

(yum install git)

#### Install dependencies
> sudo apt-get install libusb-1.0-devel

(yum install libusb-devel)
(if the system comes with older version like 1.0.8 or earlier, it is recommended you build libusbx-1.0.14 or later).

#### Install Rust compiler (I'm sure you have it)

> cargo build


### Remarks
The FT2232H device has two independent ports, both of which can be configured using MPSSE while only Channel A and B of FT4232H can be configured using MPSSE. Using MPSSE can simplify the synchronous serial protocol (USB to SPI, I2C, JTAG, etc.) design.


