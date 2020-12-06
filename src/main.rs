#![feature(const_str_from_utf8_unchecked)]
#![feature(once_cell)]

use std::{env, path::Path, time::Instant};

use owo_colors::OwoColorize;
use textwrap::indent;

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

            "log" => log(args.next()),

            "checkout" => utils::checkout(args.next().expect("Missing argument")),

            "tag" => tag(args.next().expect("Missing argument"), args.next()),

            _ => eprintln!("MuGit: '{}' is not a MuGit command. See 'mugit -h'", arg),
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
    println!("{}", utils::get_object(utils::get_oid(object), None));
}

fn hash_object(object: String) {
    println!("{}", utils::hash_object(object, None));
}

fn write_tree() {
    println!("{}", utils::write_tree(Path::new(".")));
}

fn log(mut oid: Option<String>) {
    if oid.is_none() {
        oid = Some(String::from("@"));
    }

    while let Some(mut oid_p) = oid {
        oid_p = utils::get_oid(oid_p);

        let (_, parent, mut message, date) = utils::get_commit(oid_p.to_owned());

        textwrap::fill_inplace(&mut message, 80);

        println!(
            "{} {}\n{}\n\n{}",
            "commit".yellow(),
            oid_p.yellow(),
            date,
            indent(&message, "    ")
        );

        oid = parent;
    }
}

fn tag(name: String, mut oid: Option<String>) {
    if oid.is_none() {
        oid = Some(String::from("@"));
    }

    let oid = utils::get_oid(oid.unwrap());

    utils::create_tag(name, oid);
}
