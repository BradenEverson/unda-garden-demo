use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_unda::network::{activations::Activations, network::Network};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let water_model_str = "";
    let sun_model_str = "";

    let sun_model = Network::deserialize_unda_fmt_string(sun_model_str.into(), Activations::SIGMOID);
    let water_model = Network::deserialize_unda_fmt_string(water_model_str.into(), esp_idf_unda::network::activations::Activations::SIGMOID); 

    let mut days_since = 0f32;
    let mut plant_watered = false;

    loop {

        //If we didn't do an operation, reset counter
        days_since = match plant_watered {
            true => 1f32 / 24f32,
            false => days_since + (1f32 / 24f32)
        };
        //Wait an hour
        FreeRtos::delay_ms(60 * 60 * 1000);
    }
}

