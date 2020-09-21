use std::path::Path;

mod utils;

pub fn write_tree(dir: Path) -> String {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                println!("{}", entry);
            }
        }
    }

    String::new()
}
