#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported");

use colored::*;
use glob::glob;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let devices = get_devices().unwrap();
    for dev in &devices {
        dev.show();
        if dev != devices.last().unwrap() {
            println!()
        }
    }
    Ok(())
}

// https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface
fn get_devices() -> Result<Vec<Device>, Box<dyn Error>> {
    let mut devices: Vec<Device> = Vec::new();
    for entry in glob("/sys/class/hwmon/*").unwrap() {
        let mut dev = Device::new();
        dev.dir = entry?.as_path().to_str().unwrap().into();
        dev.init_data().unwrap();
        dev.calculate_avg();
        devices.push(dev);
    }
    Ok(devices)
}

#[derive(PartialEq)]
pub struct Device {
    name: String,
    dir: PathBuf,
    temps: Vec<f32>,
    avg_temp: f32,
}

impl Device {
    fn new() -> Self {
        Device {
            name: String::new(),
            dir: PathBuf::new(),
            temps: Vec::new(),
            avg_temp: 0.0,
        }
    }

    fn show(&self) {
        println!("Name: {}", self.name.red());
        let mut idx: u32 = 0;
        for temp in &self.temps {
            println!("Sensor{}: {}", idx, format!("{}", temp).yellow());
            idx += 1;
        }
        if self.avg_temp == 0.0 {
            println!("Avg temp: {}", format!("{}", "N/A").blue());
        } else {
            println!("Avg temp: {}", format!("{}", self.avg_temp).blue());
        }
    }

    fn init_data(&mut self) -> Result<(), Box<dyn Error>> {
        for entry in
            glob(format!("{}/temp*_input", self.dir.display()).as_str())
                .unwrap()
        {
            let contents = fs::read_to_string(entry?).unwrap();
            let temp: f32 = contents.trim().parse().unwrap();
            self.temps.push(temp / 1000f32);
        }
        let name = fs::read_to_string(format!("{}/name", self.dir.display()))
            .expect("Should have been able to read the file");
        self.name = name.trim().to_string();
        Ok(())
    }

    fn calculate_avg(&mut self) -> () {
        if self.temps.len() == 0 {
            // zero denotes no sensores found
            self.avg_temp = 0.0;
            return;
        }

        let mut avg: f32 = 0.0;
        for temp in self.temps.iter() {
            avg += temp;
        }
        if !(self.temps.len() == 0) {
            self.avg_temp = avg / self.temps.len() as f32
        }
    }
}
