//! HAL interface to the SPI peripheral
//!

use gpio::gpio::PIN;
use gpio::{Input, Floating,Output,PushPull};
use nrf51::{SPI0,SPI1,spi0};
use hal::spi::FullDuplex;
use hal::blocking::spi::Transfer;

use core::ops::Deref;

/// SPI abstraction
pub struct Spi<SPI>{
    spi:SPI,
    pins:Pins,
}

pub struct Pins{
    pub sck:    PIN<Output<PushPull>>,
    pub mosi:   PIN<Output<PushPull>>,
    pub miso:   PIN<Input<Floating>>
}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
}

pub trait SpiExt : Deref<Target=spi0::RegisterBlock> + Sized {
    fn constrain(self, pins: Pins) -> Spi<Self>;
}

macro_rules! impl_spi_ext {
    ($($spi:ty,)*) => {
        $(
            impl SpiExt for $spi {
                fn constrain(self, pins: Pins) -> Spi<Self> {
                    Spi::new(self, pins)
                }
            }
        )*
    }
}

impl_spi_ext!(
    SPI0,
    SPI1,
);


impl<SPI> Spi<SPI>
where SPI:SpiExt
{
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
    pub fn new(spi: SPI, pins: Pins) -> Self
        where SPI:SpiExt {

        // The SPI peripheral requires the pins to be in a mode that is not
        // exposed through the GPIO API, and might it might not make sense to
        // expose it there.
        //
        // Select pins
        spi.pselsck.write(|w| {
            unsafe { w.bits(pins.sck.get_id().into()) }
        });
        spi.pselmosi.write(|w| {
            unsafe { w.bits(pins.mosi.get_id().into()) }
        });
        spi.pselmiso.write(|w| {
            unsafe { w.bits(pins.miso.get_id().into()) }
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

        Spi{spi:spi, pins:pins}
    }
    pub fn teardown(self) -> Pins {
         self.pins
    }
}

impl<SPI> Transfer<u8> for Spi<SPI>
where SPI:SpiExt
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

impl<SPI> FullDuplex<u8> for Spi<SPI>
where SPI:SpiExt {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {

        match self.spi.events_ready.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                // Read one 8bit value
                let byte = self.spi.rxd.read().bits() as u8;

                // Reset ready for receive event
                self.spi.events_ready.reset();

                Ok(byte)
            }
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self.spi.txd.write(|w| unsafe { w.bits(u32::from(byte)) });
        Ok(())
    }
}


