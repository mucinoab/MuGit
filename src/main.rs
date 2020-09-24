#![feature(const_str_from_utf8_unchecked)]

#[macro_use]
extern crate lazy_static;

use std::{env, path::Path, time::Instant};

use textwrap::indent;
use yansi::Paint;

mod utils;

fn main() {
    let now = Instant::now();
    let mut args = env::args();

    let current_dir = args.next().unwrap();

    // TODO use clap
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "init" => init(current_dir),

            "write-tree" => write_tree(),

            "read-tree" => utils::read_tree(args.next().expect("Missing argument")),

            "cat-file" => cat_file(args.next().expect("Missing argument")),

            "hash-object" => hash_object(args.next().expect("Missing argument")),

            "commit" => utils::commit(args.next().expect("Missing commit message")), // TODO -m and -message flags

            "log" => log(),

            _ => {}
        }
    }

    println!("{:?}", Instant::now().duration_since(now));
}

fn init(current_dir: String) {
    utils::init();
    println!(
        "Initialized empty Git repository in {}{}",
        current_dir,
        utils::GIT_DIR
    ); // TODO is this the current dir?
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

fn log() {
    let mut oid = utils::get_head();

    while let Some(oid_p) = oid {
        let (_, parent, message, date) = utils::get_commit(oid_p.to_owned());

        println!(
            "{} {}\n{}\n\n{}",
            Paint::yellow("commit"),
            Paint::yellow(oid_p),
            date,
            indent(&textwrap::fill(&message, 80), "    ")
        );

        oid = parent;
    }
}
