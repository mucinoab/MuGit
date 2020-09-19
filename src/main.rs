use std::{env, path::Path, time::Instant};

mod data;

fn main() {
    let now = Instant::now();

    let current_dir = env::args().next().unwrap();

    // TODO use clap
    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "init" => {
                data::init();
                println!(
                    "Initialized empty Git repository in {}{}",
                    current_dir,
                    data::GIT_DIR
                ); // TODO is this the current dir?
            }
            "commit" => {}
            _ => {}
        }
    }

    data::get_object(
        data::hash_object(Path::new("./src/main.rs"), Some("blob")),
        Some("blob"),
    );

    println!("{:?}", Instant::now().duration_since(now));
}
