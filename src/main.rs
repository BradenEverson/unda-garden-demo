use std::error::Error;

use esp_idf_svc::hal::{adc::{attenuation, config::Config, AdcChannelDriver, AdcDriver}, delay::FreeRtos, peripherals::Peripherals};
use esp_idf_unda::network::{activations::Activations, network::Network};

fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true))?;
    let mut water_sensor: AdcChannelDriver<{attenuation::DB_11}, _> = AdcChannelDriver::new(peripherals.pins.gpio4)?;
    let mut light_sensor: AdcChannelDriver<{attenuation::DB_11}, _> = AdcChannelDriver::new(peripherals.pins.gpio5)?;


    let sun_model_str = "";
    let water_model_str = "";

    let mut sun_model = Network::deserialize_unda_fmt_string(sun_model_str.into(), Activations::SIGMOID);
    let mut water_model = Network::deserialize_unda_fmt_string(water_model_str.into(), esp_idf_unda::network::activations::Activations::SIGMOID); 

    let mut days_since = 0f32;
    let mut time_in_sun = 0f32;
    let mut shaded = false;
    let mut plant_watered = false;

    const DELAY_TIME: u32 = 100;//60 * 60 * 1000;

    loop {
        //get water val
        let water_val = adc.read(&mut water_sensor)?;

        let dryness = water_val as f32 / 3500f32;
        let wetness = ((1f32 - dryness) / 0.60) - 0.25;

        //get light exposure

        let sunlight = adc.read(&mut light_sensor)?;
        let sunlight_percent = 1f32 - (sunlight as f32 / 3038f32);

        //println!("{:.2}", sunlight_percent);
        //println!("{:.2}", wetness);

        //Make watering inference
        let water_inf = water_model.predict(
            &vec![sunlight_percent, wetness, days_since / 10f32])[0];
        if water_inf >= 0.9 {
            //Water plant

            plant_watered = true;
        } else {
            plant_watered = false;
        }

        //Make sun inference
        let sun_inf = sun_model.predict(
            &vec![sunlight_percent, wetness, time_in_sun]
        )[0];

        if sun_inf > 0.7 {
            //Move servo
        }

        //If we didn't do an operation, reset counter
        days_since = match plant_watered {
            true => 1f32 / 24f32,
            false => days_since + (1f32 / 24f32)
        };
        //Wait an hour
        FreeRtos::delay_ms(DELAY_TIME);
    }
}

