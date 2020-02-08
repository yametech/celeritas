// #[macro_use(log_and_exit)]
extern crate logger;
use logger::{Level, Logger};
use std::thread;
use std::time::Duration;

fn main() {
    let logger = Logger::new(Level::Warning);
    logger.log(Level::Warning, "hello Celeritas".to_owned(), None);

    thread::sleep(Duration::new(0, 1));
}
