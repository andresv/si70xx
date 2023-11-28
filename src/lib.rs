//! This is a platform agnostic Rust driver for the Si70xx series
//! relative humidity and temperature sensor
//! based on [`embedded-hal`] and [`embedded-hal-async`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`embedded-hal-async`]: https://github.com/rust-embedded/embedded-hal-async
//!
//! Datasheet:
//! - [Si7006/13/20/21/34](https://www.silabs.com/sensors/humidity/si7006-13-20-21-34)
//!
//! ### Read humidity and temperature
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use si70xx::Si70xx;
//!
//! let i2c = some_i2c_from_hal();
//! let mut sensor = Si70xx::new(i2c);
//! // Start humidity and temperature measurement.
//! sensor.measure().unwrap();
//! // Read out measurement results.
//! let hum = sensor.read_humidity().unwrap();
//! let temp = sensor.read_temperature().unwrap();
//! // Values are scaled with 100, print them as floats.
//! println!("Humidity: {:.1}", hum as f32 / 100.);
//! println!("Temperature: {:.1}ºC", temp as f32 / 100.);
//! ```
//!
//! ### Read humidity and temperature using async
//! Async API becomes available by enabling `async` feature.
//! ```toml
//! si70xx = { version: 0.1.0, features = "async"}
//! ```
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use si70xx::Si70xx;
//!
//! let async_i2c = some_i2c_from_hal();
//! let mut sensor = Si70xx::new(async_i2c);
//! // Start humidity and temperature measurement.
//! sensor.measure().await.unwrap();
//! // Read out measurement results.
//! let hum = sensor.read_humidity().await.unwrap();
//! let temp = sensor.read_temperature().await.unwrap();
//! // Values are scaled with 100, print them as floats.
//! println!("Humidity: {:.1}", hum as f32 / 100.);
//! println!("Temperature: {:.1}ºC", temp as f32 / 100.);
//! ```
//! ### Read humidity and temperature with Si7013
//! Si7013 supports two I2C addresses, all other sensors use fixed 0x40 address.
//! To use Si7013, feature `si7013` must be enabled.
//!
//! ```toml
//! si70xx = { version: 0.1.0, features = "si7013"}
//! ```
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use si70xx::{Si70xx, Address};
//!
//! let i2c = some_i2c_from_hal();
//! let mut sensor = Si70xx::new(i2c, Address::H41);
//! // Measuring and reading out values is the same as in the example above.
//! ```

#![no_std]

#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[cfg(not(feature = "si7013"))]
const I2C_ADDR: u8 = 0x40;
#[cfg(feature = "si7013")]
/// Si7013 I2C address.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Address {
    /// 0x40
    H40 = 0x40,
    /// 0x41
    H41 = 0x41,
}

#[derive(Debug)]
pub enum Error<E> {
    /// Error on I²C bus.
    I2c(E),
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(PartialEq)]
enum Command {
    MeasureRhHoldMaster = 0xE5,
    ReadTemperatureFromRh = 0xE0,
}

pub struct Si70xx<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C, E> Si70xx<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Construct new Si70xx sensor.
    #[cfg(not(feature = "si7013"))]
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            addr: I2C_ADDR,
        }
    }

    /// Construct new Si7013 sensor.
    #[cfg(feature = "si7013")]
    pub fn new(i2c: I2C, addr: Address) -> Self {
        Self {
            i2c,
            addr: addr as u8,
        }
    }

    /// Initiates a measurement for relative humidity and temperature.
    ///
    /// This method starts both the relative humidity and temperature measurement.
    /// Use [`read_humidity`] and [`read_temperature`] to retrieve the measurements.
    ///
    /// [`read_humidity`]: Si70xx::read_humidity
    /// [`read_temperature`]: Si70xx::read_temperature
    #[cfg(not(feature = "async"))]
    pub fn measure(&mut self) -> Result<(), Error<E>> {
        self.i2c
            .write(self.addr, &[Command::MeasureRhHoldMaster as u8])
            .map_err(Error::I2c)?;
        Ok(())
    }

    /// Initiates a measurement for relative humidity and temperature.
    ///
    /// This method starts both the relative humidity and temperature measurement.
    /// Use [`read_humidity`] and [`read_temperature`] to retrieve the measurements.
    ///
    /// [`read_humidity`]: Si70xx::read_humidity
    /// [`read_temperature`]: Si70xx::read_temperature
    #[cfg(feature = "async")]
    pub async fn measure(&mut self) -> Result<(), Error<E>> {
        self.i2c
            .write(self.addr, &[Command::MeasureRhHoldMaster as u8])
            .await
            .map_err(Error::I2c)?;
        Ok(())
    }

    /// Retrieves the last measured relative humidity.
    ///
    /// This method should be called after [`measure`].
    /// It returns the relative humidity as a percentage multiplied by 100.
    /// For example, a return value of 4955 represents 49.55%.
    ///
    /// [`measure`]: Si70xx::measure
    #[cfg(not(feature = "async"))]
    pub fn read_humidity(&mut self) -> Result<u16, Error<E>> {
        let mut response = [0u8; 2];
        self.i2c
            .read(self.addr, &mut response)
            .map_err(Error::I2c)?;
        let rh_code = (u16::from_be_bytes([response[0], response[1]])) as u32;
        Ok(((12500 * rh_code) / 65536 - 600) as u16)
    }

    /// Retrieves the last measured relative humidity.
    ///
    /// This method should be called after [`measure`].
    /// It returns the relative humidity as a percentage multiplied by 100.
    /// For example, a return value of 4955 represents 49.55%.
    ///
    /// [`measure`]: Si70xx::measure
    #[cfg(feature = "async")]
    pub async fn read_humidity(&mut self) -> Result<u16, Error<E>> {
        let mut response = [0u8; 2];
        self.i2c
            .read(self.addr, &mut response)
            .await
            .map_err(Error::I2c)?;
        let rh_code = (u16::from_be_bytes([response[0], response[1]])) as u32;
        Ok(((12500 * rh_code) / 65536 - 600) as u16)
    }

    /// Retrieves the last measured temperature.
    ///
    /// This method should be called after [`measure`].
    /// It returns the emperature in Celsius, multiplied by 100.
    /// For example, a return value of 2550 represents 25.50°C.
    ///
    /// [`measure`]: Si70xx::measure
    #[cfg(not(feature = "async"))]
    pub fn read_temperature(&mut self) -> Result<i16, Error<E>> {
        let mut response = [0u8; 2];
        self.i2c
            .write_read(
                self.addr,
                &[Command::ReadTemperatureFromRh as u8],
                &mut response,
            )
            .map_err(Error::I2c)?;
        let temp_code = (u16::from_be_bytes([response[0], response[1]])) as u32;
        Ok(((17572 * temp_code) / 65536 - 4685) as i16)
    }

    /// Retrieves the last measured temperature.
    ///
    /// This method should be called after [`measure`].
    /// It returns the emperature in Celsius, multiplied by 100.
    /// For example, a return value of 2550 represents 25.50°C.
    ///
    /// [`measure`]: Si70xx::measure
    #[cfg(feature = "async")]
    pub async fn read_temperature(&mut self) -> Result<i16, Error<E>> {
        let mut response = [0u8; 2];
        self.i2c
            .write_read(
                self.addr,
                &[Command::ReadTemperatureFromRh as u8],
                &mut response,
            )
            .await
            .map_err(Error::I2c)?;
        let temp_code = (u16::from_be_bytes([response[0], response[1]])) as u32;
        Ok(((17572 * temp_code) / 65536 - 4685) as i16)
    }
}
