use std::fs;
use std::path::Path;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, SystemTime};
use std::thread::sleep;

use console::{style, Term};

use rand::prelude::*;

use json;

use std::collections::HashMap;

struct RenderData {
    air_quality: String,
    rating: String,
    date_observed: String,
    last_checked: SystemTime,
}

pub struct Config {
    pub key: String,
    pub zip_code: String,
    pub is_setup: bool,
    pub service_root: String,
}

impl Config {
    pub fn new(filename_and_path: &String) -> Config {
        let mut config = Config {
            key: String::new(),
            zip_code: String::new(),
            is_setup: false,
            service_root: String::from("http://www.airnowapi.org/aq/observation/zipCode/current/?format=application/json"),
        };

        if !Path::new(filename_and_path).exists() {
            return config;
        }

        let contents = fs::read_to_string(filename_and_path).unwrap();

        for line in contents.lines() {
            if line.contains("key") {
                config.key = String::from(line);
                config.set_is_setup();
            } else {
                println!("Unexpected config: {}", line);
            }
        }

        config
    }

    fn set_is_setup(&mut self) {
        if !self.is_setup {
            self.is_setup = true;
        }
    }

    pub fn to_string(&self) -> String {
        let content = format!("{}{}", "key: ", self.key);
        content
    }

    pub fn save(&self, path: String) {
        println!("self.to_string: {}", self.to_string());
        fs::write(path, self.to_string()).unwrap_or_else(|error| {
            println!("There was an error: {}", error);
        });
    }
}

pub struct Runtime {
    pub config: Config,
    display_width: u16,
    display_height: u16,
    last_call: SystemTime,
    terminal: Term,
    data: RenderData,
}

impl Runtime {
    pub fn new(config: Config) -> Runtime {
        Runtime {
            config: config,
            display_height: 0,
            display_width: 0,
            last_call: SystemTime::now(),
            terminal: Term::stdout(),
            data: RenderData {
                air_quality: String::new(),
                rating: String::new(),
                date_observed: String::from("NA"),
                last_checked: SystemTime::now(),
            }
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.clear_screen().unwrap();
        self.terminal.hide_cursor().unwrap();

        let (heigth, width) = self.terminal.size();
        self.display_height = heigth;
        self.display_width =width;

        let service = DataService::new(self.config.service_root.clone(), self.config.key.clone());

        self.fill_data(&service);
        self.draw();

        // loop
        loop {
            match self.last_call.elapsed() {
                Ok(elapsed) => {
                    // all good
                    if elapsed.as_secs() > 60 {
                        // get
                        self.fill_data(&service);
                        self.draw();
                    }
                }
                Err(e) => {
                    // uuummm....sure.....?
                }
            }

            self.draw();
            sleep(Duration::from_secs(6));
        }
    }

    fn fill_data(&mut self, service: &DataService) {
        let data = service.get_data_for_zip(self.config.zip_code.clone());

        self.data.air_quality = data["AQI"].to_string();
        self.data.date_observed = data["DateObserved"].to_string();
        self.data.rating = data["Quality"].to_string();
        self.data.last_checked = SystemTime::now();
    }

    fn draw(&self) {
        let mut rng = thread_rng();
        let x = rng.gen_range(0, self.display_width);
        let y = rng.gen_range(0, self.display_height);

        self.terminal.clear_screen().unwrap();

        self.terminal.move_cursor_to(x as usize, y as usize).unwrap();
        let display = format!("air quality: {}", self.data.air_quality);
        self.terminal.write_str(&display).unwrap();

        let y = y + 1;
        self.terminal.move_cursor_to(x as usize, y as usize).unwrap();
        let display = format!("rating: {}", self.data.rating);
        self.terminal.write_str(&display).unwrap();
    }
}

struct DataService {
    base_url: String,
    api_key: String,
}

impl DataService {
    pub fn new(base_url: String, api_key: String) -> DataService {
        DataService {
            base_url: base_url,
            api_key: api_key,
        }
    }

    pub fn get_data_for_zip(&self, zip: String) -> HashMap<String, String> {
        let query = format!("{}&zipCode={}&distance=25", self.get_query_params(), zip);
        let resp = reqwest::blocking::get(query.as_str()).unwrap()
            .text().unwrap();
        let resp_obj = json::parse(&resp).unwrap();
        //     .json::<HashMap<String, String>>().unwrap();  // I know, bad bad.  But just playing with a prototype for the time being.

        let mut data = HashMap::new();
        data.insert(String::from("AQI"), resp_obj[0]["AQI"].to_string());
        data.insert(String::from("DateObserved"), resp_obj[0]["DateObserved"].to_string());
        data.insert(String::from("Quality"), resp_obj[0]["Category"]["Name"].to_string());

        data
    }

    fn get_query_params(&self) -> String {
        String::from(format!("{}&API_KEY={}", self.base_url, self.api_key))
            .trim()
            .to_string()
    }
}