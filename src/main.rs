#[macro_use]
extern crate slog;

use std::{fmt, result};

use slog::*;

pub struct OutputSerializer;

impl Serializer for OutputSerializer {
    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments) -> Result {
        print!(", {}={}", key, val);
        Ok(())
    }
}

pub struct OutputDrain;

impl Drain for OutputDrain {
    type Ok = ();
    type Err = ();

    fn log(&self, record: &Record, values: &OwnedKVList) -> result::Result<Self::Ok, Self::Err> {
        print!("{}", record.msg());

        record
            .kv()
            .serialize(record, &mut OutputSerializer)
            .unwrap();
        values.serialize(record, &mut OutputSerializer).unwrap();

        println!();
        Ok(())
    }
}

fn main() {
    let log = Logger::root(Fuse(OutputDrain), o!("version"=>0,"subversion"=>0.1));
    info!(log, "celeritas is {v}", v = "0.0.1");
}
