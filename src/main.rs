use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_unda::network::network::Network;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    //Serialized models can be loaded here either from explicity Unda string or file if board has an SD card slot
    let model_str = "";
    let model = Network::deserialize_unda_fmt_string(model_str.into()); 

    //Collect params from IO and generate inferences!
    loop {
        log::info!("Model Inference:");
        FreeRtos::delay_ms(5000);
    }
}
