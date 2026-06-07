//! Multi-constellation GNSS receiver driver.
//! Implements: UAV::Avionics::GPSReceiver

pub struct Fix { pub lat: f64, pub lon: f64, pub quality: u8 }
pub fn read_fix() -> Option<Fix> { None }
