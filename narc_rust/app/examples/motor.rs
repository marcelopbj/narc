<<<<<<< HEAD
#![feature(panic_implementation)]
=======
>>>>>>> temp
#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate narc_hal;
extern crate embedded_hal;

use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use cortex_m::asm::bkpt;
<<<<<<< HEAD

use narc_hal::stm32l052;
use narc_hal::stm32l052::GPIOB;
use narc_hal::rcc::RccExt;
use narc_hal::gpio::GpioExt;
use narc_hal::gpio::{Output, PushPull, gpiob::PB6};
use narc_hal::pwm::PwmExt;
use narc_hal::flash::FlashExt;
use narc_hal::time::U32Ext;

use embedded_hal::digital::OutputPin;
use embedded_hal::digital::InputPin;

use embedded_hal::PwmPin;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
   
    let hw = stm32l052::Peripherals::take().unwrap();

    let mut rcc = hw.RCC.constrain();
    let mut flash = hw.FLASH.constrain();

    let mut gpioa = hw.GPIOA.split(&mut rcc.iop);


    let mut gpiob = hw.GPIOB.split(&mut rcc.iop);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mot2 = gpioa.pa0.into_alternate(&mut gpioa.moder).af2(&mut gpioa.afrl);
    let mut mot2_in2: PB6<Output<PushPull>> = gpiob.pb6.into_output(&mut gpiob.moder).push_pull(&mut gpiob.otyper);
    let mut mot2_in1 = gpiob.pb7.into_output(&mut gpiob.moder).push_pull(&mut gpiob.otyper);
    
    mot2_in2.set_low();
    mot2_in1.set_high();

    let mut mot2_pwm = hw.TIM2
                .pwm(
                    mot2,
=======
 
use narc_hal::stm32l052;
use narc_hal::rcc::RccExt;
use narc_hal::gpio::GpioExt;
use narc_hal::pwm::PwmExt;
use narc_hal::flash::FlashExt;
use narc_hal::time::U32Ext;
use narc_hal::gpio::gpiob::{PB6, PB7};
use narc_hal::gpio::{Output, PushPull};
use narc_hal::delay::Delay;
use narc_hal::qei::QeiFunc;

use embedded_hal::digital::OutputPin;
use embedded_hal::digital::InputPin;
use embedded_hal::prelude::*;
 
use embedded_hal::PwmPin;
use cortex_m_rt::entry;
 
#[entry]
fn main() -> ! {     
    
    let hw = stm32l052::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    
    let mut rcc = hw.RCC.constrain();
    let mut flash = hw.FLASH.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = hw.GPIOA.split(&mut rcc.iop);
    let mut gpiob = hw.GPIOB.split(&mut rcc.iop);
    let mut gpioc = hw.GPIOC.split(&mut rcc.iop);

    let mot2 = gpioa.pa0.into_alternate(&mut gpioa.moder).af2(&mut gpioa.afrl);
    let mot1 = gpioa.pa1.into_alternate(&mut gpioa.moder).af2(&mut gpioa.afrl);
    let mut led = gpioa.pa5.into_output(&mut gpioa.moder).push_pull(&mut gpioa.otyper);
    let mut mot2_in2 = gpiob.pb6.into_output(&mut gpiob.moder).push_pull(&mut gpiob.otyper);
    let mut mot2_in1 = gpiob.pb7.into_output(&mut gpiob.moder).push_pull(&mut gpiob.otyper);
    let mut mot1_in1 = gpioc.pc14.into_output(&mut gpioc.moder).push_pull(&mut gpioc.otyper);
    let mut mot1_in2 = gpioc.pc15.into_output(&mut gpioc.moder).push_pull(&mut gpioc.otyper);
    let mot2_enca = gpioa.pa6.into_alternate(&mut gpioa.moder).af5(&mut gpioa.afrl);
    let mot2_encb = gpioa.pa7.into_alternate(&mut gpioa.moder).af5(&mut gpioa.afrl);
    let mot1_enca = gpioa.pa2.into_alternate(&mut gpioa.moder).af5(&mut gpioa.afrl);
    let mot1_encb = gpioa.pa3.into_alternate(&mut gpioa.moder).af5(&mut gpioa.afrl);

    let mut delay = Delay::new(cp.SYST, clocks);


    /*let (mut mot2_pwm, mut mot1_pwm) = hw.TIM2
                .pwm(
                    (mot2, mot1),
                    60.hz(),
                    clocks,
                    &mut rcc.apb1,
                );*/

    let (mut mot1_pwm, mut mot2_pwm) = hw.TIM2
                .pwm(
                    (mot1, mot2),
>>>>>>> temp
                    60.hz(),
                    clocks,
                    &mut rcc.apb1,
                );

<<<<<<< HEAD
    let max = mot2_pwm.get_max_duty();
    mot2_pwm.enable(); 
    mot2_pwm.set_duty(max/2);


    loop{
        
    }
}

#[allow(deprecated)]
#[panic_implementation]
fn panic(_info: &PanicInfo) -> ! {
    bkpt();

    loop {
        atomic::compiler_fence(Ordering::SeqCst)
    }
}
=======
    /*let qei_1 = hw.TIM21
            .qei(
                (mot1_enca, mot1_encb),
                &mut rcc.apb2);
    
    qei_1.reset();*/

    let qei_2 = hw.TIM22
            .qei(
                (mot2_enca, mot2_encb),
                &mut rcc.apb2);
    
    qei_2.reset();

        //let max = mot2_pwm.get_max_duty();

        
        //mot2_pwm.enable();
        //mot2_pwm.set_duty(max/5);

        //mot1_pwm.enable();
        //mot1_pwm.set_duty(max/5);


    //short_brake(&mut mot1_in1, &mut mot1_in2, &mut mot2_in1, &mut mot2_in2);
    //forward(&mut mot1_in1, &mut mot1_in2, &mut mot2_in1, &mut mot2_in2);
    //turn_left(&mut mot1_in1, &mut mot1_in2, &mut mot2_in1, &mut mot2_in2);
    //turn_right(&mut mot1_in1, &mut mot1_in2, &mut mot2_in1, &mut mot2_in2);


    led.set_high();

     loop{
       if qei_2.count() < 65176 {
            led.set_high();
        }
        else{
            led.set_low();
        }      
     }
 }

fn short_brake <T,U,V,W> (mot1_in1: &mut T, mot1_in2: &mut U, mot2_in1: &mut V, mot2_in2: &mut W) where T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin {

    mot1_in1.set_high();
    mot1_in2.set_high();
    mot2_in1.set_high();
    mot2_in2.set_high();
} 

fn forward <T,U,V,W> (mot1_in1: &mut T, mot1_in2: &mut U, mot2_in1: &mut V, mot2_in2: &mut W) where T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin {

    mot1_in1.set_high();
    mot1_in2.set_low();
    mot2_in1.set_low();
    mot2_in2.set_high();
} 


fn backward <T,U,V,W> (mot1_in1: &mut T, mot1_in2: &mut U, mot2_in1: &mut V, mot2_in2: &mut W) where T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin {

    mot1_in1.set_low();
    mot1_in2.set_high();
    mot2_in1.set_high();
    mot2_in2.set_low();
}

fn turn_left <T,U,V,W> (mot1_in1: &mut T, mot1_in2: &mut U, mot2_in1: &mut V, mot2_in2: &mut W) where T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin {

    mot1_in1.set_high();
    mot1_in2.set_high();
    mot2_in1.set_low();
    mot2_in2.set_high();
} 

fn turn_right <T,U,V,W> (mot1_in1: &mut T, mot1_in2: &mut U, mot2_in1: &mut V, mot2_in2: &mut W) where T: OutputPin, U: OutputPin, V: OutputPin, W: OutputPin {

    mot1_in1.set_high();
    mot1_in2.set_low();
    mot2_in1.set_high();
    mot2_in2.set_high();
} 


 #[panic_handler]
 fn panic(_info: &PanicInfo) -> ! {
     bkpt();
 
     loop {
         atomic::compiler_fence(Ordering::SeqCst)
     }
 }
>>>>>>> temp
