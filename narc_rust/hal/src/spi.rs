use core::ptr;

use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};
use nb;
use stm32l052::{SPI1};

use gpio::gpioa::{PA8, PA15};
use gpio::gpiob::{PB3, PB4, PB5};
use gpio::{AF0};
use rcc::{APB1, APB2, Clocks};
use time::Hertz;

/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}

// FIXME these should be "closed" traits
/// SCK pin -- DO NOT IMPLEMENT THIS TRAIT
pub trait SckPin<SPI> {}

/// MISO pin -- DO NOT IMPLEMENT THIS TRAIT
pub trait MisoPin<SPI> {}

/// MOSI pin -- DO NOT IMPLEMENT THIS TRAIT
pub trait MosiPin<SPI> {}

impl SckPin<SPI1> for PB3<AF0> {}

impl MisoPin<SPI1> for PB4<AF0> {}

impl MosiPin<SPI1> for PB5<AF0> {}

/// SPI peripheral operating in full duplex master mode
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

impl<SCK, MISO, MOSI> Spi<SPI1, (SCK, MISO, MOSI)> {
                pub fn spi1<F>(
                    spi: SPI1,
                    pins: (SCK, MISO, MOSI),
                    mode: Mode,
                    freq: F,
                    clocks: Clocks,
                    apb: &mut APB2,
                ) -> Self
                where
                    F: Into<Hertz>,
                    SCK: SckPin<SPI1>,
                    MISO: MisoPin<SPI1>,
                    MOSI: MosiPin<SPI1>,
                
                {
                    Spi::_spi1(spi, pins, mode, freq.into(), clocks, apb)
                }
}

macro_rules! hal {
    ($($SPIX:ident: ($spiX:ident, $APBX:ident, $spiXen:ident, $spiXrst:ident, $pclkX:ident),)+) => {
        $(
            impl<SCK, MISO, MOSI> Spi<$SPIX, (SCK, MISO, MOSI)> {
                /// Configures the SPI peripheral to operate in full duplex master mode
                fn $spiX(
                    spi: $SPIX,
                    pins: (SCK, MISO, MOSI),
                    mode: Mode,
                    freq: Hertz,
                    clocks: Clocks,
                    apb: &mut $APBX,
                ) -> Self{

                    // enable or reset $SPIX
                    apb.enr().modify(|_, w| w.$spiXen().set_bit());
                    apb.rstr().modify(|_, w| w.$spiXrst().set_bit());
                    apb.rstr().modify(|_, w| w.$spiXrst().clear_bit());

                    // FRXTH: RXNE event is generated if the FIFO level is greater than or equal to
                    //        8-bit
                    // DS: 8-bit data size
                    // SSOE: Slave Select output disabled
                    //spi.cr2
                    //    .write(|w| unsafe {
                    //        w.frxth().set_bit().ds().bits(0b111).ssoe().clear_bit()
                    //    });

                    // disable SS output
                    //spi.cr2.write(|w| w.ssoe().clear_bit());//reset state


                    let br: u8 = match clocks.$pclkX().0 / freq.0 {
                        0 => unreachable!(),
                        1...2 => 0b000,
                        3...5 => 0b001,
                        6...11 => 0b010,
                        12...23 => 0b011,
                        24...39 => 0b100,
                        40...95 => 0b101,
                        96...191 => 0b110,
                        _ => 0b111,
                    };

                    //let br = 0b000;

                    // CPHA: phase
                    // CPOL: polarity
                    // MSTR: master mode
                    // BR: 1 MHz
                    // SPE: SPI disabled
                    // LSBFIRST: MSB first
                    // SSM: enable software slave management (NSS pin free for other uses)
                    // SSI: set nss high = master mode
                    // CRCEN: hardware CRC calculation disabled
                    // BIDIMODE: 2 line unidirectional (full duplex)
                    spi.cr1.write(|w| unsafe {
                        w
                            .mstr()
                            .set_bit()
                            .spe()
                            .set_bit()
                            .bidimode()
                            .clear_bit()
                            .rxonly()
                            .clear_bit()
                            .cpha()
                            .bit(mode.phase == Phase::CaptureOnSecondTransition)
                            .cpol()
                            .bit(mode.polarity == Polarity::IdleHigh)
                            .dff()
                            .clear_bit()
                            .lsbfirst()
                            .clear_bit()
                            .br()
                            .bits(br)
                            /*.ssi()
                            .set_bit()
                            .ssm()
                            .set_bit()
                            .crcen()
                            .clear_bit()*/
                    });

                    Spi { spi, pins }
                }

                /// Releases the SPI peripheral and associated pins
                pub fn free(self) -> ($SPIX, (SCK, MISO, MOSI)) {
                    (self.spi, self.pins)
                }
            }

            impl<PINS> FullDuplex<u8> for Spi<$SPIX, PINS> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
                        // reading a half-word)
                        return Ok(unsafe {
                            ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }

                fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.txe().bit_is_set() {
                        // NOTE(write_volatile) see note above
                        unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
                        return Ok(());
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl<PINS> embedded_hal::blocking::spi::transfer::Default<u8> for Spi<$SPIX, PINS> {}

            impl<PINS> embedded_hal::blocking::spi::write::Default<u8> for Spi<$SPIX, PINS> {}
        )+
    }
}

hal! {
    SPI1: (_spi1, APB2, spi1en, spi1rst, pclk2),
    //SPI2: (spi2, APB1, spi2en, spi2rst, pclk1),
}