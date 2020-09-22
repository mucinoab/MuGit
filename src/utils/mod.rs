use std::{
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

pub const GIT_DIR: &str = "./.mu_git";
const NULL_CHAR: &str = unsafe { std::str::from_utf8_unchecked(&[0]) };

lazy_static! {
    static ref GITIGNORE: Vec<String> = {
        if let Ok(file) = File::open(".gitignore") {
            BufReader::new(file)
                .lines()
                .filter_map(Result::ok)
                .map(|x| x.trim().to_string())
                .collect()
        } else {
            Vec::new()
        }
    };
}

pub fn init() {
    fs::create_dir(GIT_DIR).expect("Failed to create .mu_git directory");
    fs::create_dir(format!("{}/objects", GIT_DIR))
        .expect("Failed to create .mu_git/objects directory");
}

pub fn hash_object(data: String, type_: Option<&str>) -> String {
    let mut obj = String::from(type_.unwrap_or("blob"));

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

pub fn write_tree(dir: &Path) -> String {
    let mut entries = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let type_;
        let oid;
        let file_name = path.file_name().unwrap_or_default().to_str();

        if is_ignored(&path) {
            continue;
        }

        if path.is_dir() {
            type_ = "tree";
            oid = write_tree(&path);
        } else {
            type_ = "blob";
            oid = hash_object(fs::read_to_string(&path).unwrap_or_default(), None);
        }

        entries.push(format!(
            "{} {} {}",
            file_name.unwrap_or_default(),
            oid,
            type_
        ));
    }

    entries.sort_unstable();
    hash_object(entries.join("\n"), Some("tree"))
}

fn is_ignored(path: &Path) -> bool {
    if let Some(path) = path.to_str() {
        for item in GITIGNORE.iter() {
            if path.contains(item) {
                return true;
            }
        }
    }
    false
}

fn tree_entries(oid: String) -> Vec<(String, String, String)> {
    get_object(oid, Some("tree"))
        .lines()
        .map(|line| line.split(' '))
        .map(|mut entry| {
            (
                entry.next().unwrap().into(),
                entry.next().unwrap().into(),
                entry.next().unwrap().into(),
            )
        })
        .collect()
}

pub fn get_tree(oid: String, mut base_path: &mut String) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for (name, oid, type_) in tree_entries(oid) {
        base_path.push_str(&name);
        match type_.as_str() {
            "blob" => {
                result.push((base_path.to_owned(), oid));
            }

            "tree" => {
                base_path.push('/');
                result = get_tree(oid, &mut base_path);
            }

            _ => unreachable!("Unknown tree entry {}", type_),
        }
    }

    result
}

fn read_tree(tree_oid: String) {
    for (path, oid) in get_tree(tree_oid, &mut String::from("./")) {}
}
