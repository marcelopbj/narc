//! Serial

use core::marker::{PhantomData, Unsize};
use core::ptr;
use core::sync::atomic::{self, Ordering};

use cast::u16;
use nb;
use embedded_hal::serial::{Read, Write};

use stm32l052::{USART2, USART1};

use gpio::gpioa::{PA2, PA3, PA9, PA10};
use gpio::{AF4};
use dma::{dma1, CircBuffer, CircBufferLinear, Static, Transfer, R, W};
use rcc::{APB1, APB2, Clocks};
use time::Bps;


pub enum Event {
    Rxne,
    Txe,
    Cmie,
}

#[derive(Debug)]
pub enum Error {
    Framing,
    Noise,
    Overrun,
    Parity,
    #[doc(hidden)]
    _Extensible,
}

pub trait Pins<USART> {}

/// PA2 - USART2_tx PA3 - USART2_rx
impl Pins<USART2> for (PA2<AF4>, PA3<AF4>) {}

/// PA2 - USART2_tx PA3 - USART2_rx
impl Pins<USART1> for (PA9<AF4>, PA10<AF4>) {}


/// Serial Abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

/// Serial receiver
pub struct Rx<USART> {
    _usart: PhantomData<USART>,
}

/// Serial transmitter
pub struct Tx<USART> {
    _usart: PhantomData<USART>,
}

pub struct ReleaseInterupt<USART> {
    _usart: PhantomData<USART>,
}

pub trait ClearInterupt {
    fn clear_isr_cmie(&mut self);
}

macro_rules! usart {
    ($(
        $USARTX:ident: (
            $usartX:ident,
            $usartXen:ident,
            $usartXrst:ident,
            $APB:ident,
            csel: $csel_value:expr,
            rx: $rx_chan:path, $csr:ident,
            tx: $tx_chan:path, $cst:ident
        ),
    )+) => {
        $(
            impl<PINS> Serial<$USARTX, PINS> {
                pub fn $usartX (
                    usart: $USARTX,
                    pins: PINS,
                    baud_rate: Bps,
                    clocks: Clocks,
                    apb: &mut $APB,
                    character_match: Option<u8>,
                ) -> Self
                where 
                    PINS: Pins<$USARTX>
                {
                    apb.enr().modify(|_, w| w.$usartXen().set_bit());
                    apb.rstr().modify(|_, w| w.$usartXrst().set_bit());
                    apb.rstr().modify(|_, w| w.$usartXrst().clear_bit());

                    usart.cr3.modify(|_, w| w.dmat().set_bit().dmar().set_bit());

                    // TODO, olhar isso
                    let brr = clocks.pclk2().0 / baud_rate.0;
                    assert!(brr >= 16, "impossible baud rate");
                    usart.brr.write(|w| unsafe { w.bits(brr) });

                    if let Some(value) = character_match {
                        let add4_7 = value / 16;
                        let add0_3 = value % 16;

                        usart.cr2.modify(|_, w| unsafe{ w.add4_7().bits(add4_7).add0_3().bits(add0_3) });
                    }
                    
                    usart
                        .cr1
                        .write(|w| w.ue().set_bit().re().set_bit().te().set_bit());

                    Serial { usart, pins }
                }

                pub fn listen(&self, event: Event) {
                    match event {
                        Event::Rxne => self.usart.cr1.modify(|_, w| w.rxneie().set_bit()),
                        Event::Txe => self.usart.cr1.modify(|_, w| w.txeie().set_bit()),
                        Event::Cmie => self.usart.cr1.modify(|_, w| w.cmie().set_bit()),
                    }
                }

                pub fn unlisten(&self, event: Event) {
                    match event {
                        Event::Rxne => self.usart.cr1.modify(|_, w| w.rxneie().clear_bit()),
                        Event::Txe => self.usart.cr1.modify(|_, w| w.txeie().clear_bit()),
                        Event::Cmie => self.usart.cr1.modify(|_, w| w.cmie().clear_bit()),
                    }
                }

                pub fn release(self) -> ($USARTX, PINS) {
                    (self.usart, self.pins)
                }

                pub fn split(self) -> (Tx<$USARTX>, Rx<$USARTX>, ReleaseInterupt<$USARTX>) {
                    (
                        Tx {
                            _usart: PhantomData,
                        },
                        Rx {
                            _usart: PhantomData,
                        },
                        ReleaseInterupt {
                            _usart: PhantomData,
                        },
                    )
                }
            }

            impl ClearInterupt for ReleaseInterupt<$USARTX> {
                fn clear_isr_cmie(&mut self) {
                    unsafe { (*$USARTX::ptr()).icr.write(|w| w.cmcf().set_bit()) };
                }
            }
            
            impl Read<u8> for Rx<$USARTX> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let sr = unsafe { (*$USARTX::ptr()).isr.read() };

                    Err(if sr.pe().bit_is_set() {
                        nb::Error::Other(Error::Parity)
                    } else if sr.fe().bit_is_set() {
                        nb::Error::Other(Error::Framing)
                    } else if sr.nf().bit_is_set() {
                        nb::Error::Other(Error::Noise)
                    } else if sr.ore().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) see `write_volatile` below
                        return Ok(unsafe {
                            ptr::read_volatile(&(*$USARTX::ptr()).rdr as *const _ as *const _)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl Rx<$USARTX> {
                pub fn circ_buf<B>(
                    self,
                    mut chan: $rx_chan,
                    buffer: &'static mut [B; 2],
                ) -> CircBuffer<B, $rx_chan> 
                where 
                    B: Unsize<[u8]>
                {
                    {
                        let buffer: &[u8] = &buffer[0];
                        chan.cmar().write(|w| unsafe {
                            w.ma().bits(buffer.as_ptr() as usize as u32)
                        });
                        chan.cndtr().write(|w| unsafe{
                            w.ndt().bits(u16(buffer.len() * 2).unwrap())
                        });
                        chan.cpar().write(|w| unsafe {
                            w.pa().bits(&(*$USARTX::ptr()).rdr as *const _ as usize as u32)
                        });
                        chan.cselr().modify(|_, w| unsafe {
                            w.$csr().bits($csel_value)
                        });

                        // TODO can we weaken this compiler barrier?
                        // NOTE(compiler_fence) operations on `buffer` should not be reordered after
                        // the next statement, which starts the DMA transfer
                        atomic::compiler_fence(Ordering::SeqCst);

                        let medium = 0b01;
                        let bit_8 = 0b00;
                        chan.ccr().modify(|_, w| unsafe {
                            w.mem2mem()
                            .clear_bit()
                            .pl()
                            // .medium()
                            .bits(medium)
                            .msize()
                            // .bit8()
                            .bits(bit_8)
                            .psize()
                            // .bit8()
                            .bits(bit_8)
                            .minc()
                            .set_bit()
                            .pinc()
                            .clear_bit()
                            .circ()
                            .set_bit()
                            .dir()
                            .clear_bit()
                            .en()
                            .set_bit()
                        });
                    }

                    CircBuffer::new(buffer, chan)
                }

                pub fn circ_buf_linear<B>(
                    self,
                    mut chan: $rx_chan,
                    buffer: &'static mut [B; 1],
                ) -> CircBufferLinear<B, $rx_chan> 
                where 
                    B: Unsize<[u8]>
                {
                    {
                        let buffer: &[u8] = &buffer[0];
                        chan.cmar().write(|w| unsafe {
                            w.ma().bits(buffer.as_ptr() as usize as u32)
                        });
                        chan.cndtr().write(|w| unsafe{
                            w.ndt().bits(u16(buffer.len() * 1).unwrap())
                        });
                        chan.cpar().write(|w| unsafe {
                            w.pa().bits(&(*$USARTX::ptr()).rdr as *const _ as usize as u32)
                        });
                        chan.cselr().modify(|_, w| unsafe {
                            w.$csr().bits($csel_value)
                        });

                        // TODO can we weaken this compiler barrier?
                        // NOTE(compiler_fence) operations on `buffer` should not be reordered after
                        // the next statement, which starts the DMA transfer
                        atomic::compiler_fence(Ordering::SeqCst);

                        let medium = 0b01;
                        let bit_8 = 0b00;
                        chan.ccr().modify(|_, w| unsafe {
                            w.mem2mem()
                            .clear_bit()
                            .pl()
                            // .medium()
                            .bits(medium)
                            .msize()
                            // .bit8()
                            .bits(bit_8)
                            .psize()
                            // .bit8()
                            .bits(bit_8)
                            .minc()
                            .set_bit()
                            .pinc()
                            .clear_bit()
                            .circ()
                            .set_bit()
                            .dir()
                            .clear_bit()
                            .en()
                            .set_bit()
                        });
                    }

                    CircBufferLinear::new(buffer, chan)
                }
            
                pub fn read_exact<B>(
                    self,
                    mut chan: $rx_chan,
                    buffer: &'static mut B,
                ) -> Transfer<W, &'static mut B, $rx_chan, Self>
                where
                    B: Unsize<[u8]>,
                {
                    {
                        let buffer: &[u8] = buffer;
                        chan.cmar().write(|w| unsafe {
                            w.ma().bits(buffer.as_ptr() as usize as u32)
                        });
                        chan.cndtr().write(|w| unsafe{
                            w.ndt().bits(u16(buffer.len()).unwrap())
                        });
                        chan.cpar().write(|w| unsafe {
                            w.pa().bits(&(*$USARTX::ptr()).rdr as *const _ as usize as u32)
                        });
                        chan.cselr().modify(|_, w| unsafe {
                            w.$csr().bits($csel_value)
                        });

                        // TODO can we weaken this compiler barrier?
                        // NOTE(compiler_fence) operations on `buffer` should not be reordered after
                        // the next statement, which starts the DMA transfer
                        atomic::compiler_fence(Ordering::SeqCst);

                        let medium = 0b01;
                        let bit_8 = 0b00;
                        chan.ccr().modify(|_, w| unsafe {
                            w.mem2mem()
                            .clear_bit()
                            .pl()
                            // .medium()
                            .bits(medium)
                            .msize()
                            // .bit8()
                            .bits(bit_8)
                            .psize()
                            // .bit8()
                            .bits(bit_8)
                            .minc()
                            .set_bit()
                            .pinc()
                            .clear_bit()
                            .circ()
                            .clear_bit()
                            .dir()
                            .clear_bit()
                            .en()
                            .set_bit()
                            });
                        }

                    Transfer::w(buffer, chan, self)
                }
            }

            impl Tx<$USARTX> {
                pub fn write_all<A, B>(
                    self,
                    mut chan: $tx_chan,
                    buffer: B,
                ) -> Transfer<R, B, $tx_chan, Self>
                where
                    A: Unsize<[u8]>,
                    B: Static<A>,
                {
                    {
                        let buffer: &[u8] = buffer.borrow();
                        chan.cmar().write(|w| unsafe {
                            w.ma().bits(buffer.as_ptr() as usize as u32)
                        });
                        chan.cndtr().write(|w| unsafe{
                            w.ndt().bits(u16(buffer.len()).unwrap())
                        });
                        chan.cpar().write(|w| unsafe {
                            w.pa().bits(&(*$USARTX::ptr()).tdr as *const _ as usize as u32)
                        });
                        chan.cselr().modify(|_, w| unsafe {
                            w.$cst().bits($csel_value)
                        });
                        // TODO can we weaken this compiler barrier?
                        // NOTE(compiler_fence) operations on `buffer` should not be reordered after
                        // the next statement, which starts the DMA transfer
                        atomic::compiler_fence(Ordering::SeqCst);

                        let medium = 0b01;
                        let bit_8 = 0b00;
                        chan.ccr().modify(|_, w| unsafe {
                            w.mem2mem()
                            .clear_bit()
                            .pl()
                            // .medium()
                            .bits(medium)
                            .msize()
                            // .bit8()
                            .bits(bit_8)
                            .psize()
                            // .bit8()
                            .bits(bit_8)
                            .minc()
                            .set_bit()
                            .pinc()
                            .clear_bit()
                            .circ()
                            .clear_bit()
                            .dir()
                            .set_bit()
                            .en()
                            .set_bit()
                        });
                    }

                    Transfer::r(buffer, chan, self)
                }
            }

            impl Write<u8> for Tx<$USARTX> {
                type Error = !;

                fn flush(&mut self) -> nb::Result<(), !> {
                    // NOTE(unsafe) atomic read with no side effects
                    let sr = unsafe { (*$USARTX::ptr()).isr.read() };

                    if sr.tc().bit_is_set() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn write(&mut self, byte: u8) -> nb::Result<(), !> {
                    // NOTE(unsafe) atomic read with no side effects
                    let sr = unsafe { (*$USARTX::ptr()).isr.read() };

                    if sr.txe().bit_is_set() {
                        // NOTE(unsafe) atomic write to stateless register
                        // NOTE(write_volatile) 8-bit write that's not possible through the svd2rust API
                        unsafe {
                            ptr::write_volatile(&(*$USARTX::ptr()).tdr as *const _ as *mut _, byte)
                        }
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

        )+
    };
}

usart! {
    USART2: (
        usart2,
        usart2en,
        usart2rst,
        APB1,
        csel: 0b0100,
        rx: dma1::C5, c5s,
        tx: dma1::C4, c4s
    ),
}

usart! {
    USART1: (
        usart1,
        usart1en,
        usart1rst,
        APB2,
        csel: 0b0011,
        rx: dma1::C3, c3s,
        tx: dma1::C2, c2s
    ),
}
