//! HAL interface to the SPI peripheral
//!

use gpio::gpio::PIN;
use gpio::{Input, Floating,Output,PushPull};
use nrf51::{SPI0,SPI1};
use hal::spi::FullDuplex;
use hal::blocking::spi::Transfer;
// extern crate embedded_hal;
// use hal::blocking::spi::Write;
// use hal::spi::FullDuplex;
/// SPI abstraction
pub struct Spi<SPI> {
    spi: SPI,
    sckpin:  PIN<Output<PushPull>>,
    mosipin: PIN<Output<PushPull>>,
    misopin: PIN<Input<Floating>>,
}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
}

impl Spi<SPI0> {
        /// Interface to a SPI instance
        ///
        /// This is a very basic interface that comes with the following limitation:
        /// The SPI instances share the same address space with instances of SPIM,
        /// SPIS, SPI, TWIS, and TWI. For example, SPI0 conflicts with SPIM0, SPIS0,
        /// etc.; SPI1 conflicts with SPIM1, SPIS1, etc. You need to make sure that
        /// conflicting instances are disabled before using `SPI`. Please refer to the
        /// product specification for mo
        /// 
        /// 
    pub fn spi0(spi: SPI0, sck: PIN<Output<PushPull>>, mosi:PIN<Output<PushPull>>, miso: PIN<Input<Floating>>) -> Self {

        // The SPI peripheral requires the pins to be in a mode that is not
        // exposed through the GPIO API, and might it might not make sense to
        // expose it there.
        //
        // Select pins
        spi.pselsck.write(|w| {
            unsafe { w.bits(sck.get_id().into()) }
        });
        spi.pselmosi.write(|w| {
            unsafe { w.bits(mosi.get_id().into()) }
        });
        spi.pselmiso.write(|w| {
            unsafe { w.bits(miso.get_id().into()) }
        });

        // Enable SPIM instance
        spi.enable.write(|w|
            w.enable().enabled()
        );

        // Set to SPI mode 0
        spi.config.write(|w|
            w
                .order().msb_first()
                .cpha().leading()
                .cpol().active_high()
        );

        // Configure frequency
        spi.frequency.write(|w|
            w.frequency().m4() // 4MHz
        );

        Spi{spi:spi,sckpin: sck, mosipin: mosi, misopin: miso}
    }

    /// Return the raw interface to the underlying SPI peripheral
    pub fn release(self) -> (SPI0, PIN<Output<PushPull>>,PIN<Output<PushPull>>,PIN<Input<Floating>> ) {
        (self.spi, self.sckpin,self.mosipin,self.misopin)
    }
}
pub struct Spim<T>(T);

impl Transfer<u8> for Spi<SPI0>

// impl<T> Transfer<u8> for S<T> where
//      T: SpimExt,
//      S: FullDuplex<u8>,
{
   type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        for word in words.iter_mut() {
            block!(self.send(word.clone()))?;
            *word = block!(FullDuplex::read(self))?;
        }

        Ok(words)
    }
}

impl FullDuplex<u8> for Spi<SPI0> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let spi = unsafe { &*SPI0::ptr() };
        match spi.events_ready.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                // Read one 8bit value
                let byte = spi.rxd.read().bits() as u8;

                // Reset ready for receive event
                spi.events_ready.reset();

                Ok(byte)
            }
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        let spi = unsafe { &*SPI0::ptr() };
        spi.txd.write(|w| unsafe { w.bits(u32::from(byte)) });
        Ok(())
    }
}


