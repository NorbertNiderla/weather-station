const DHT11_STARTING_TIME_US: u32 = 20 * 1000;
const DHT11_WAIT_FOR_START_US: u32 = 10;
const DHT11_STATE_CHANGE_TIMEOUT_US: u32 = 1000 * 1000;

pub trait Dht11Pin {
    fn is_low(&mut self) -> bool;
    fn is_high(&mut self) -> bool;
    fn set_low(&mut self);
    fn set_high(&mut self);
    fn set_mode_input(&mut self);
    fn set_mode_output(&mut self);
}

pub trait Dht11Timing {
    fn wait(&self, microseconds: u32);

    /// # Returns
    /// Current time in microseconds
    fn get_time_us(&self) -> u128;
}


fn wait_for_level(level: bool, pin: &mut dyn Dht11Pin, timing: &dyn Dht11Timing) -> Result<(), Dht11Error>{
    let timeout = timing.get_time_us() + DHT11_STATE_CHANGE_TIMEOUT_US as u128;
    loop {
        if level {
            if pin.is_high() == true {
                return Ok(());
            }
        } else {
            if pin.is_low() == true {
                return Ok(());
            }
        }
        
        if timing.get_time_us() > timeout {
            return Err(Dht11Error::Timeout);
        }
    }
}

const fn bits_to_u8(bits: &[bool]) -> u8 {
    assert!(bits.len() == 8);

    let mut result: u8 = 0;

        if bits[0] {
            result += 128;
        }

        if bits[1] {
            result += 64;
        }

        if bits[2] {
            result += 32;
        }

        if bits[3] {
            result += 16;
        }

        if bits[4] {
            result += 8;
        }

        if bits[5] {
            result += 4;
        }

        if bits[6] {
            result += 2;
        }

        if bits[7] {
            result |= 1;
        }

    result
}

struct Dht11RawData {
    integral_rh_data: u8,
    decimal_rh_data: u8,
    integral_t_data: u8,
    decimal_t_data: u8,
    checksum: u8
}

impl Dht11RawData {
    fn new(bits: &[bool]) -> Self {
        assert!(bits.len() == 40);

        Dht11RawData{
            integral_rh_data: bits_to_u8(&bits[0..8]),
            decimal_rh_data: bits_to_u8(&bits[8..16]),
            integral_t_data: bits_to_u8(&bits[16..24]),
            decimal_t_data: bits_to_u8(&bits[24..32]),
            checksum: bits_to_u8(&bits[32..40]),
        }
    }

    const fn is_checksum_correct(&self) -> bool {
        let checksum: u8 = ((self.integral_rh_data as u32 + 
            self.decimal_rh_data as u32 + 
            self.integral_t_data as u32 + 
            self.decimal_t_data as u32) % 256) as u8;
        self.checksum == checksum
    }
}

#[derive(Debug)]
pub enum Dht11Error {
    Timeout,
    ChecksumError,
}

pub struct Dht11Readout {
    ///
    /// # Unit
    /// Percents.
    pub humidity: f64,

    ///
    /// # Unit
    /// Celcius degrees.
    pub temperature: f64,
}

impl Dht11Readout {
    fn new(data: &Dht11RawData) -> Self {
        Dht11Readout{
            humidity: data.integral_rh_data as f64 + data.decimal_rh_data as f64 / 10.0,
            temperature: data.integral_t_data as f64 + data.decimal_t_data as f64 / 10.0
        }
    }
}

///
/// # Parameters
/// time = microseconds
const fn convert_time_to_bit(time: u128) -> bool {
    assert!(time < 1000000);
    if time < 50 {
        return false;
    } else {
        return true;
    }
}

fn dht11_init_readout(pin: &mut dyn Dht11Pin, timing: &dyn Dht11Timing) -> Result<(), Dht11Error>{
    pin.set_mode_output();
    pin.set_high();
    pin.set_low();
    timing.wait(DHT11_STARTING_TIME_US);
    pin.set_high();
    timing.wait(DHT11_WAIT_FOR_START_US);

    pin.set_mode_input();
    wait_for_level(false, pin, timing)?;
    wait_for_level(true, pin, timing)?;
    wait_for_level(false, pin, timing)?;
    Ok(())
}

fn dht11_read_bit(pin: &mut dyn Dht11Pin, timing: &dyn Dht11Timing) -> Result<bool, Dht11Error> {
    wait_for_level(true, pin, timing)?;
    let start_time: u128 = timing.get_time_us();
    wait_for_level(false, pin, timing)?;
    let elapsed_time = timing.get_time_us() - start_time;
    Ok(convert_time_to_bit(elapsed_time))
}

pub fn dht11_perform_readout(pin: &mut dyn Dht11Pin, timing: &dyn Dht11Timing) -> Result<Dht11Readout, Dht11Error> {
    dht11_init_readout(pin, timing)?;

    let mut bits: [bool; 40] = [false; 40];

    for bit in bits.iter_mut() {
        *bit = dht11_read_bit(pin, timing)?;
    }

    let raw_data = Dht11RawData::new(&bits);

    if raw_data.is_checksum_correct() == false {
        return Err(Dht11Error::ChecksumError);
    }

    return Ok(Dht11Readout::new(&raw_data));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_to_u8() {
        let bits: [bool; 8] = [true,false,true,false,true,false,true,false];
        let byte: u8 = bits_to_u8(&bits);
        assert_eq!(byte, 128+32+8+2);
    }

    #[test]
    fn conversion_to_readout() {
        let readout = Dht11Readout::new(&Dht11RawData { 
            integral_rh_data: 48, 
            decimal_rh_data: 0, 
            integral_t_data: 23, 
            decimal_t_data: 8, 
            checksum: 0 });
        
        assert_eq!(readout.humidity, 48.0);
        assert_eq!(readout.temperature, 23.8);
    }
}