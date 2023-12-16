use embedded_hal::delay::DelayUs;

use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};

use si70xx::Si70xx;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config).unwrap();

    let mut sensor = Si70xx::new(i2c);

    loop {
        sensor.measure().unwrap();
        FreeRtos.delay_ms(30);
        // Read out measurement results.
        let hum = sensor.read_humidity().unwrap();
        // FreeRtos.delay_ms(20);
        let temp = sensor.read_temperature().unwrap();

        // Values are scaled with 100, print them as floats.
        log::info!("Humidity: {:.1}", hum as f32 / 100.);
        log::info!("Temperature: {:.1}ºC", temp as f32 / 100.);

        FreeRtos.delay_ms(500u32);
        log::info!("f read");
    }
}
