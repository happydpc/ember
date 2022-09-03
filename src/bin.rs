use std::env;

use mylib::Manager;

use log::LevelFilter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut log_level = Some(LevelFilter::Info);
    if args.len() > 1 {
        if args[1] == "debug" {
            log_level = Some(LevelFilter::Debug);
        } else if args[1] == "brooke" {
            println!("hi brooke");
        } else {
            println!("Get fucked that's not a valid log level");
        }
    }
    let mut app: mylib::Application = mylib::Application::create_application(log_level);
    app.startup();
    app.run();
}