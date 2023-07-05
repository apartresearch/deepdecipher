pub mod data;
pub mod server;

#[cfg(feature = "python")]
mod pyo3;

pub fn setup_keyboard_interrupt() {
    if let Err(error) = ctrlc::set_handler(move || {
        println!("Keyboard interrupt received, exiting...");
        std::process::abort();
    }) {
        match error {
            ctrlc::Error::MultipleHandlers => {
                eprintln!("A handler already exists for keyboard interrupts.");
            }
            ctrlc::Error::NoSuchSignal(signal_type) => {
                eprintln!("Signal type not found on system: {signal_type:?}");
            }
            ctrlc::Error::System(error) => {
                eprintln!(
                    "Unexpected system error while setting keyboard interrupt handler: {error}"
                );
            }
        }
    }
}
