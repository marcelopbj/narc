#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate narc_hal;
extern crate embedded_hal;

use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use cortex_m::asm::bkpt;
 
use narc_hal::stm32l052;
use narc_hal::rcc::RccExt;
use narc_hal::gpio::GpioExt;
use narc_hal::pwm::PwmExt;
use narc_hal::flash::FlashExt;
use narc_hal::time::U32Ext;
use narc_hal::qei::*;
use narc_hal::delay::Delay;
use narc_hal::spi::*;
use narc_hal::stm32l052::SPI1;

use embedded_hal::spi::{MODE_0, Phase, Polarity};
use embedded_hal::digital::OutputPin;
use embedded_hal::digital::InputPin;
use embedded_hal::{Direction, Qei};
use embedded_hal::prelude::*;
 
use embedded_hal::PwmPin;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {     
    let hw = stm32l052::Peripherals::take().unwrap();

    let mut rcc = hw.RCC.constrain();
    let mut flash = hw.FLASH.constrain();

    let cp = cortex_m::Peripherals::take().unwrap();

    let mut gpioa = hw.GPIOA.split(&mut rcc.iop);
    let mut gpiob = hw.GPIOB.split(&mut rcc.iop);
    let mut gpioc = hw.GPIOC.split(&mut rcc.iop);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut led = gpioa.pa5.into_output(&mut gpioa.moder).push_pull(&mut gpioa.otyper);
    let button = gpioa.pa4.into_input(&mut gpioa.moder).pull_up(&mut gpioa.pupdr);
    let mot1 = gpioa.pa1.into_alternate(&mut gpioa.moder).af2(&mut gpioa.afrl);
    let mot2 = gpioa.pa0.into_alternate(&mut gpioa.moder).af2(&mut gpioa.afrl);
    let mut mot2_in2 = gpiob.pb6.into_output(&mut gpiob.moder).push_pull(&mut gpiob.otyper);
    let mut mot2_in1 = gpiob.pb7.into_output(&mut gpiob.moder).push_pull(&mut gpiob.otyper);
    let mut mot1_in1 = gpioc.pc14.into_output(&mut gpioc.moder).push_pull(&mut gpioc.otyper);
    let mut mot1_in2 = gpioc.pc15.into_output(&mut gpioc.moder).push_pull(&mut gpioc.otyper);
    let mot2_enca = gpioa.pa6.into_alternate(&mut gpioa.moder).af5(&mut gpioa.afrl);
    let mot2_encb = gpioa.pa7.into_alternate(&mut gpioa.moder).af5(&mut gpioa.afrl);
    let mot1_enca = gpioa.pa2.into_alternate(&mut gpioa.moder).af0(&mut gpioa.afrl);
    let mot1_encb = gpioa.pa3.into_alternate(&mut gpioa.moder).af0(&mut gpioa.afrl);
    let nrf24_sck = gpiob.pb3.into_alternate(&mut gpiob.moder).af0(&mut gpiob.afrl);
    let nrf24_miso = gpiob.pb4.into_alternate(&mut gpiob.moder).af0(&mut gpiob.afrl);
    let nrf24_mosi = gpiob.pb5.into_alternate(&mut gpiob.moder).af0(&mut gpiob.afrl);
    let mut nrf24_ce = gpioa.pa8.into_output(&mut gpioa.moder).push_pull(&mut gpioa.otyper);
    let mut nrf24_csn = gpioa.pa15.into_output(&mut gpioa.moder).push_pull(&mut gpioa.otyper);

    let mut delay = Delay::new(cp.SYST, clocks);

    nrf24_ce.set_high();
    nrf24_csn.set_high();

    let mut spi = Spi::
            spi1(
                hw.SPI1,
                (nrf24_sck, nrf24_miso, nrf24_mosi),
                MODE_0,
                1000.khz(),
                clocks,
                &mut rcc.apb2);
    
     loop{
        if button.is_low(){
                spi.send(b'a');
            }
        else{
            if let Ok(r) = spi.read(){
                if r == b'a'{
                led.set_high();
                delay.delay_ms(1_000_u32);
            }
            };
        }

    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
     bkpt();
 
     loop {
         atomic::compiler_fence(Ordering::SeqCst)
     }
}