#![feature(const_str_from_utf8_unchecked)]

use std::{env, path::Path, time::Instant};

mod base;
mod data;

fn main() {
    let now = Instant::now();
    let mut args = env::args();

    let current_dir = args.next().unwrap();

    // TODO use clap
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "init" => {
                data::init();
                println!(
                    "Initialized empty Git repository in {}{}",
                    current_dir,
                    data::GIT_DIR
                ); // TODO is this the current dir?
            }

            "cat-file" => cat_file(args.next().unwrap()),

            "hash-object" => hash_object(args.next().unwrap()),

            _ => {}
        }
    }

    println!("{:?}", Instant::now().duration_since(now));
}

fn cat_file(object: String) {
    println!("{}", data::get_object(object, None));
}

fn hash_object(object: String) {
    println!("{}", data::hash_object(Path::new(&object), None));
}
