pub mod index_calculator;
pub mod sqlite_handler;
#[macro_use] extern crate quick_error;
use index_calculator::get_index;
use sqlite_handler::SqliteHandler;
use std::fs;
use semver::{Version};
use std::path::PathBuf;

fn main() {
    let db_handler = SqliteHandler::new();
    let mut counter = 0;
    let total_paths = fs::read_dir("/data/praezi/batch/data/").unwrap().count();
    let paths = fs::read_dir("/data/praezi/batch/data/").unwrap();
    for path in paths {
        let pather = path.unwrap();
        match fs::read_dir(pather.path()){
            Err(why) => {
                println!("{:?}", why);
            },
            Ok(versions) => {
                let mut highest_version = Version::parse(&"0.0.0").unwrap();
                let mut highest_path = PathBuf::new();
                let mut highest_ver_str = String::new();
                for version_folder in versions{
                    let crate_path = version_folder.unwrap().path();
                    let crate_version = crate_path.file_name().unwrap().to_string_lossy().to_string();
                    let v = Version::parse(&crate_version).unwrap();
                    if v > highest_version{
                        highest_version = v;
                        highest_path = crate_path;
                        highest_ver_str = crate_version;
                    }
                }

                if !highest_ver_str.is_empty() {
                    let crate_name = pather.path().file_name().unwrap().to_string_lossy().to_string();
                    match get_index(&highest_path, &crate_name, &highest_ver_str){
                        Err(why) => {
                            println!("Failed for {:?}", why);
                        },
                        Ok(val) => {
                            db_handler.insert_metric(&val, &crate_name, &highest_ver_str);
                            println!("Success ({}/{}) for - {} - {}", counter, total_paths, &crate_name, &highest_version);
                        }
                    }
    
                }
                counter+=1;
            }
        }
    }
}
