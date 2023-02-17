use std::env;

use ember::Manager;

use log::LevelFilter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut log_level = LevelFilter::Info;
    if args.len() > 1 {
        if args[1] == "debug" {
            log_level = LevelFilter::Debug;
        } else if args[1] == "brooke" {
            println!("hi brooke");
        } else {
            println!("Get fucked that's not a valid log level");
        }
    }
    let mut app: ember::Application = ember::Application::create_application(log_level);
    app.run();
}