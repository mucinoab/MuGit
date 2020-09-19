use std::{fs, path::Path};

pub const GIT_DIR: &str = "./.mu_git";

pub fn init() {
    fs::create_dir(GIT_DIR).expect("Failed to create .mu_git directory");
    fs::create_dir(format!("{}/objects", GIT_DIR))
        .expect("Failed to create .mu_git/objects directory");
}

pub fn hash_object(path: &Path, type_: Option<&str>) -> String {
    let mut obj = String::from(type_.unwrap_or("blob"));

    let data = fs::read_to_string(&path).unwrap();
    obj.push_str(&String::from_utf8(vec![0u8]).unwrap());
    obj.push_str(&data);

    let oid = sha1::Sha1::from(&obj).hexdigest();
    fs::write(format!("{}/objects/{}", GIT_DIR, oid), obj).unwrap();

    return oid;
}

pub fn get_object(oid: String, expected: Option<&str>) -> String {
    let obj = fs::read_to_string(Path::new(&format!("{}/objects/{}", GIT_DIR, &oid))).unwrap();
    let null = obj.find(&String::from_utf8(vec![0u8]).unwrap()).unwrap();
    let type_ = &obj[..null];
    let content = &obj[null + 1..];

    match expected {
        Some(expected) => assert_eq!(type_, expected, "Expected {}, got {}", type_, expected),
        None => {}
    }

    content.to_owned()
}
