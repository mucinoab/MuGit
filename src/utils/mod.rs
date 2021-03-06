use std::{
    fs::{self, File},
    io::{prelude::*, BufReader},
    lazy::SyncLazy,
    path::Path,
};

use chrono::prelude::*;
use hashbrown::HashMap;
use walkdir::WalkDir;

pub const GIT_DIR: &str = "./.mu_git";
pub const HEAD: &str = "HEAD";
pub const TAGS: &str = "refs/tags/";
const NULL_CHAR: &str = unsafe { std::str::from_utf8_unchecked(&[0]) };

static GIT_IGNORE: SyncLazy<Vec<String>> = SyncLazy::new(|| {
    if let Ok(file) = File::open(".gitignore") {
        BufReader::new(file)
            .lines()
            .filter_map(Result::ok)
            .map(|x| x.trim().to_string())
            .collect()
    } else {
        Vec::new()
    }
});

pub fn init() {
    fs::create_dir(GIT_DIR).expect("Failed to create .mu_git directory");
    fs::create_dir_all(format!("{}/{}", GIT_DIR, TAGS)).expect("Failed to create directories");
    fs::create_dir(format!("{}/objects", GIT_DIR))
        .expect("Failed to create .mu_git/objects directory");
}

pub fn hash_object(data: String, type_: Option<&str>) -> String {
    let mut obj = String::from(type_.unwrap_or("blob"));

    obj.push_str(NULL_CHAR);
    obj.push_str(&data);

    let oid = sha1::Sha1::from(&obj).hexdigest();

    fs::write(format!("{}/objects/{}", GIT_DIR, oid), obj).expect("Could not write object file");

    oid
}

pub fn get_object(oid: String, expected: Option<&str>) -> String {
    let obj = fs::read_to_string(Path::new(&format!("{}/objects/{}", GIT_DIR, oid))).unwrap();
    let null = obj.find(NULL_CHAR).unwrap();
    let type_ = &obj[..null];
    let content = &obj[null + 1..];

    if let Some(expected) = expected {
        assert_eq!(type_, expected, "Expected {}, got {}", type_, expected)
    }

    content.to_owned()
}

pub fn write_tree(dir: &Path) -> String {
    let mut entries = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if is_ignored(&path) {
            continue;
        }

        let type_;
        let oid;
        let file_name = path.file_name().unwrap_or_default().to_str();

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

    entries.sort_unstable(); // TODO Is this really necessary?
    hash_object(entries.join("\n"), Some("tree"))
}

pub fn get_oid(mut name: String) -> String {
    if &name == "@" {
        name = String::from("HEAD")
    }

    let refs_to_try = vec![
        format!("{}", name),
        format!("refs/{}", name),
        format!("refs/tags/{}", name),
        format!("refs/heads/{}", name),
    ];

    for refr in refs_to_try {
        if let Some(name) = get_ref(&refr) {
            return name;
        }
    }

    let is_hex = name.chars().any(|chr| !chr.is_ascii_hexdigit());

    if name.len() == 40 && is_hex {
        return name;
    }

    unreachable!("Unknown name {}", name);
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

pub fn get_tree(oid: String, base_path: String) -> HashMap<String, (String, String)> {
    let mut result = HashMap::new();

    for (name, oid, type_) in tree_entries(oid) {
        let mut path = format!("{}{}", base_path, name);

        match type_.as_str() {
            "blob" => {
                result.insert(path, (oid, base_path.to_owned()));
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

pub fn read_tree(mut tree_oid: String) {
    empty_current_dir();
    tree_oid = get_oid(tree_oid);

    let tree = get_tree(tree_oid, String::from("./"));

    for (path, (oid, base_path)) in tree {
        fs::create_dir_all(Path::new(&base_path)).unwrap_or_default();
        fs::write(Path::new(&path), get_object(oid.clone(), None)).unwrap();
    }
}

fn empty_current_dir() {
    for entry in WalkDir::new("./")
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if entry.path().is_file() {
            fs::remove_file(path).unwrap();
        } else {
            fs::remove_dir(path).unwrap_or_default();
        }
    }
}

pub fn commit(message: String) {
    let mut commit = format!("tree {}\n", write_tree(Path::new("./")));

    if let Some(head) = get_ref(HEAD) {
        let head = format!("parent {}\n", head);
        commit.push_str(&head);
    }

    commit.push_str(&format!(
        "Date: {}\n{}\n",
        Local::now().format("%c %z"),
        message
    ));

    let oid = hash_object(commit, Some("commit"));
    update_ref(HEAD, &oid);
}

fn update_ref(refe: &str, oid: &str) {
    let ref_path = format!("{}/{}", GIT_DIR, refe);
    fs::write(ref_path, oid).expect("Could not write reference");
}

pub fn get_ref(refe: &str) -> Option<String> {
    let ref_path = format!("{}/{}", GIT_DIR, refe);

    if let Ok(head) = fs::read_to_string(ref_path) {
        Some(head.trim().into())
    } else {
        None
    }
}

pub fn get_commit(oid: String) -> (String, Option<String>, String, String) {
    let commit = get_object(oid, Some("commit"));
    let mut n = 0;

    let mut parent = None;
    let mut tree = String::with_capacity(40);
    let mut date = String::with_capacity(64);

    for line in commit.lines() {
        let mut kv = line.splitn(2, ' ');
        let key = kv.next().unwrap_or_default();
        let value = kv.next().unwrap_or_default();

        match key {
            "tree" => {
                tree = value.to_string();
            }

            "parent" => {
                parent = Some(value.to_string());
            }

            "Date:" => {
                date.push_str("Date:  ");
                date.push_str(value);
            }

            _ => break,
        }
        n += 1;
    }

    let message = commit.lines().skip(n).collect::<Vec<&str>>().join("\n");

    (tree, parent, message, date)
}

pub fn checkout(oid: String) {
    let (tree, ..) = get_commit(oid.to_owned());
    read_tree(tree);
    update_ref(HEAD, &oid);
}

pub fn create_tag(name: String, oid: String) {
    update_ref(&format!("{}{}", TAGS, name), &oid);
}
