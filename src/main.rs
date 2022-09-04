use std::{fs, env};

mod build;

fn main() {
    let mut args = env::args();

    let op = args.nth(1).expect("No operation specified.");

    match op.as_str() {
        "build" => {
            let fname = args.next().unwrap_or("index.vasc".to_owned());
            fs::write("./tmp.vasm", build(fname)).expect("temp file write failed");
        },
        _ => {
            println!("Unsupported operation {}", op);
            return;
        }
    }
}

fn build(fname : String) -> String {
    let f = fs::read_to_string(fname).expect("No index.vasc or otherwise named file in the working directory.");
    build::build(f).expect("Build failed:")
}