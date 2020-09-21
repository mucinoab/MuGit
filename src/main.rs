#![feature(const_str_from_utf8_unchecked)]

#[macro_use]
extern crate lazy_static;

use std::{env, path::Path, time::Instant};

mod utils;

fn main() {
    let now = Instant::now();
    let mut args = env::args();

    let current_dir = args.next().unwrap();

    // TODO use clap
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "init" => {
                utils::init();
                println!(
                    "Initialized empty Git repository in {}{}",
                    current_dir,
                    utils::GIT_DIR
                ); // TODO is this the current dir?
            }

            "write-tree" => write_tree(),

            "cat-file" => cat_file(args.next().expect("Missing argument")),

            "hash-object" => hash_object(args.next().expect("Missing argument")),

            _ => {}
        }
    }

    println!("{:?}", Instant::now().duration_since(now));
}

fn cat_file(object: String) {
    println!("{}", utils::get_object(object, None));
}

fn hash_object(object: String) {
    println!("{}", utils::hash_object(object, None));
}

fn write_tree() {
    println!("{}", utils::write_tree(Path::new(".")));
}
