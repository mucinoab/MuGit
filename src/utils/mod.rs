use std::{
    collections::HashMap,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

use walkdir::WalkDir;

pub const GIT_DIR: &str = "./.mu_git";
const NULL_CHAR: &str = unsafe { std::str::from_utf8_unchecked(&[0]) };

lazy_static! {
    static ref GIT_IGNORE: Vec<String> = {
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
        for item in GIT_IGNORE.iter() {
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

pub fn get_tree(oid: String, base_path: String) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (name, oid, type_) in tree_entries(oid) {
        let mut path = format!("{}{}", base_path, name);
        match type_.as_str() {
            "blob" => {
                result.insert(path, oid);
            }

            "tree" => {
                path.push('/');
                result.extend(get_tree(oid, path));
            }

            _ => unreachable!("Unknown entry {}", type_),
        }
    }

    result
}

pub fn read_tree(tree_oid: String) {
    empty_current_dir();
    for (path, oid) in get_tree(tree_oid, String::from("./")) {
        println!("{}", path);
        fs::create_dir_all(Path::new(&path)).unwrap_or_default();
        fs::write(Path::new(&path), get_object(oid, None)).unwrap();
    }
}

pub fn empty_current_dir() {
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(|e| e.ok())
    {
        println!("{}", entry.path().display());
        let path = entry.path();

        if entry.path().is_file() {
            fs::remove_file(path).unwrap();
        } else {
            fs::remove_dir(path).unwrap();
        }
    }
}

// def _empty_current_directory ():
//     for root, _, filenames in os.walk ('.'):
//         for filename in filenames:
//             path = os.path.relpath (f'{root}/{filename}')
//             if is_ignored (path) or not os.path.isfile (path):
//                 continue
//             os.remove (path)
//
