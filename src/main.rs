use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use dht11::{dht11_perform_readout, Dht11Pin, Dht11Timing};
use rppal::gpio::{Gpio, IoPin, Mode};

struct IoPinDht {
    pin: IoPin
}

impl IoPinDht {
    fn new(pin_number: u8) -> Self {
        let gpio: Gpio = Gpio::new().unwrap();
        IoPinDht{ pin: gpio.get(pin_number).unwrap().into_io(Mode::Output)}
    }
}

impl Dht11Pin for IoPinDht {
    fn is_low(&mut self) -> bool {
        return self.pin.is_low();
    }

    fn is_high(&mut self) -> bool {
        return self.pin.is_high();
    }

    fn set_low(&mut self) {
        self.pin.set_low();
    }

    fn set_high(&mut self) {
        self.pin.set_high();
    }

    fn set_mode_input(&mut self) {
        self.pin.set_mode(Mode::Input);
    }

    fn set_mode_output(&mut self) {
        self.pin.set_mode(Mode::Output);
    }
}

struct Timing;

impl Timing {
    fn new() -> Self {
        Timing{}
    }
}

impl Dht11Timing for Timing {
    fn wait(&self, microseconds: u32) {
        thread::sleep(Duration::from_micros(microseconds.into()));
    }

    fn get_time_us(&self) -> u128 {
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        return duration_since_epoch.as_micros();
    }
} 

fn main() {
    println!("Weather station started!");
    let mut pin = IoPinDht::new(23);
    let data = dht11_perform_readout(&mut pin, &Timing::new()).unwrap();

    println!("Weather station readout:");
    println!("Humidity: {}%", data.humidity);
    println!("Temperature: {}*C", data.temperature);
}
