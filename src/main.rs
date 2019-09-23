use std::fs::{metadata, File};
use std::io::Read;
use std::process::exit;

use clap::{crate_authors, crate_version, App, Arg};
use syn;
use walkdir::{DirEntry, WalkDir};

use ruml::{file_parser, render_plantuml, Entity};

fn main() {
    let matches = App::new("ruml")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Parse rust code and visualize")
        .arg(
            Arg::with_name("output_type")
                .short("t")
                .long("type")
                .value_name("OUTPUT_TYPE")
                .help("Output type. Default value is PlantUml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file or directory to use")
                .required(false)
                .index(1),
        )
        .get_matches();

    let source = matches.value_of("INPUT").unwrap_or("./");
    match metadata(source) {
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
        Ok(md) => {
            if md.is_dir() {
                let mut entities: Vec<Entity> = Vec::new();
                for entry in WalkDir::new(source) {
                    match entry {
                        Err(e) => {
                            println!("{}", e);
                            exit(1)
                        }
                        Ok(entry) => {
                            if is_rust_module(&entry) {
                                let path =
                                    entry.path().to_str().expect("fail to access rust module");
                                let mut ent = file_parser(parse_syntax(path));
                                entities.append(&mut ent)
                            }
                        }
                    }
                }
                println!("{}", render_plantuml(entities));
                exit(0)
            }

            let entities = file_parser(parse_syntax(source));
            println!("{}", render_plantuml(entities))
        }
    }
}

fn parse_syntax(path: &str) -> syn::File {
    let mut file = File::open(path).expect("Unable to open file");
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    syn::parse_file(&src).expect("Unable to parse file")
}

fn is_rust_module(entry: &DirEntry) -> bool {
    let path: String = entry
        .file_name()
        .to_str()
        .unwrap_or("")
        .chars()
        .rev()
        .take(3)
        .collect();
    &path == "sr."
}
