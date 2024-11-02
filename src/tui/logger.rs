use std::{fs::OpenOptions, io::Write};


pub struct Logger;

const PATH: &str = "logs.txt";

impl Logger {
    pub fn log(log: &str) {
        let mut file = OpenOptions::new().append(true).open(PATH).unwrap();

        let _ = writeln!(file, "{}", log);
    }
}
