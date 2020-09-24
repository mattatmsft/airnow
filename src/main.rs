use std::io::{self, Read, Write};

fn setup(config: &mut airquality::Config) {
    println!("Please enter your API Key:");
    let mut api_key = String::new();
    match io::stdin().read_line(&mut api_key) {
        Ok(buffer) => println!("buffer read: {}", buffer),
        Err(error) => panic!("There was an error reading from input."),
    };

    config.key = api_key;
    // config.save(String::from(""));  // todo so we can save.
}

fn main() {
    let config_file_name = String::from("config.mjw");
    println!("Getting all set up with the new and improved air quality app.");

    let mut config = airquality::Config::new(&config_file_name);

    if !config.is_setup {
        setup(&mut config);
    }

    // now we're all set up, so let's render some stuff.
    let mut runtime = airquality::Runtime::new(config);
    runtime.start().unwrap();
}
