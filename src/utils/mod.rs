use std::{fs, path::Path};

pub const GIT_DIR: &str = "./.mu_git";
const NULL_CHAR: &str = unsafe { std::str::from_utf8_unchecked(&[0]) };

pub fn init() {
    fs::create_dir(GIT_DIR).expect("Failed to create .mu_git directory");
    fs::create_dir(format!("{}/objects", GIT_DIR))
        .expect("Failed to create .mu_git/objects directory");
}

pub fn hash_object(path: &Path, type_: Option<&str>) -> String {
    let mut obj = String::from(type_.unwrap_or("blob"));

    let data = fs::read_to_string(path).unwrap();
    obj.push_str(NULL_CHAR);
    obj.push_str(&data);

    let oid = sha1::Sha1::from(&obj).hexdigest();

    fs::write(format!("{}/objects/{}", GIT_DIR, oid), obj).expect("Could not write object file");

    return oid;
}

pub fn get_object(oid: String, expected: Option<&str>) -> String {
    let obj = fs::read_to_string(Path::new(&format!("{}/objects/{}", GIT_DIR, oid))).unwrap();
    let null = obj.find(NULL_CHAR).unwrap();
    let type_ = &obj[..null];
    let content = &obj[null + 1..];

    match expected {
        Some(expected) => assert_eq!(type_, expected, "Expected {}, got {}", type_, expected),
        None => {}
    }

    content.to_owned()
}

pub fn write_tree(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if is_ignored(&path) {
            continue;
        }

        if path.is_dir() {
            write_tree(&path);
        } else {
            println!("{}", hash_object(&path, Some("full")));
        }
    }
}

fn is_ignored(path: &Path) -> bool {
    let ignore = [".git", ".mu_git", "debug", "release"];
    let path = path.to_str().unwrap_or("");

    for item in &ignore {
        if path.contains(item) {
            return true;
        }
    }
    false
}
