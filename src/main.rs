use std::{error::Error, ops::{Range, RangeBounds}, time::Instant};

use esp_idf_hal::{ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution}, units::Hertz};
use esp_idf_svc::hal::{adc::{attenuation, config::Config, AdcChannelDriver, AdcDriver}, delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};
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

    let mut relay = PinDriver::output(peripherals.pins.gpio17)?;

    relay.set_high()?;

    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0, 
        &TimerConfig::default()
            .frequency(Hertz::from(50))
            .resolution(Resolution::Bits14))?;
        
    let mut driver = LedcDriver::new(
            peripherals.ledc.channel0, timer_driver, peripherals.pins.gpio11)?;

    let max_duty = driver.get_max_duty();
    let min_limit = max_duty * 25 / 1000;
    let max_limit = max_duty * 125 / 1000;



    driver.set_duty(map(0, 0, 180, min_limit + 250, max_limit))?;

    FreeRtos::delay_ms(2000);

    //move_to(&mut driver, 30..200, min_limit, max_limit)?;
    


    let sun_model_str = "";
    let water_model_str = "";

    let mut sun_model = Network::deserialize_unda_fmt_string(sun_model_str.into(), Activations::SIGMOID);
    let mut water_model = Network::deserialize_unda_fmt_string(water_model_str.into(), esp_idf_unda::network::activations::Activations::SIGMOID); 

    let mut days_since = 0f32;
    let mut time_in_sun = 0f32;
    let mut shaded: bool = false;
    let mut plant_watered = false;

    let mut time_since_shade = Instant::now();

    const DELAY_TIME: u32 = /*100;*/60 * 60 * 1000;

    loop {
        //get water val
        let water_val = adc.read(&mut water_sensor)?;

        let dryness = water_val as f32 / 3500f32;
        let wetness = ((1f32 - dryness) / 0.60) - 0.25;

        //get light exposure

        let sunlight = adc.read(&mut light_sensor)?;
        let sunlight_percent = 1f32 - (sunlight as f32 / 3038f32);

        //Make watering inference
        /*let water_inf = water_model.predict(
            &vec![sunlight_percent, wetness, days_since / 10f32])[0];
        */
        if true {
            //Water plant
            relay.set_low()?;
            FreeRtos::delay_ms(500);
            relay.set_high()?;

            plant_watered = true;
        } else {
            plant_watered = false;
        }

        //Make sun inference
        let sun_inf = sun_model.predict(
            &vec![sunlight_percent, wetness, time_in_sun]
        )[0];

        if sun_inf > 0.7 && !shaded {

            move_to(&mut driver, 30..200, min_limit, max_limit)?;
            shaded = true;
            time_since_shade = Instant::now();
            
        } else if shaded {

            let total_sun_time = Instant::now().duration_since(time_since_shade);
            if total_sun_time.as_millis() >= 300000 {
                move_to(&mut driver, 200..30, min_limit, max_limit)?;
                shaded = false;
            }

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


fn map(x: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> u32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

fn move_to(driver:&mut LedcDriver, range: Range<usize>, min_limit: u32, max_limit: u32) -> Result<(), Box<dyn Error>> {
    for angle in (30..=200).step_by(2) {
        driver
            .set_duty(map(angle, 0, 180, min_limit, max_limit))
            .unwrap();
        FreeRtos::delay_ms(25);
    }
    Ok(())
}
