pub mod index_calculator;
pub mod sqlite_handler;
#[macro_use] extern crate quick_error;
use index_calculator::get_index;
use sqlite_handler::SqliteHandler;
use std::fs;
use semver::{Version};
use std::path::PathBuf;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let database_path = match args.get(1) {
        None => {
            "/database/prazi.db"
        },
        Some(v) => {
            v
        }
    };
    let db_handler = SqliteHandler::new(database_path);

    let mut counter = 0;
    let data_path = "/data/praezi/batch/data/";
    let updated_data_path = "/data/praezi_algirdas/datasets/";
    // let data_path = "/home/algirdas/crate_data/";
    // let updated_data_path = "/home/algirdas/crate_data2/";
    println!("Processing data in {}", data_path);
    let total_paths = fs::read_dir(data_path).unwrap().count();
    let paths = fs::read_dir(data_path).unwrap();
    for path in paths {
        let pather = path.unwrap();
        match fs::read_dir(pather.path()){
            Err(why) => {
                println!("Failed reading versions in {:?} - {:?}", pather, why);
            },
            Ok(versions) => {
                let mut highest_version = Version::parse(&"0.0.0").unwrap();
                let mut highest_update_path = PathBuf::from(updated_data_path);
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
                    highest_update_path = highest_update_path.join(updated_data_path).join(&crate_name).join(&highest_ver_str);

                    println!("Processing {} - {}", &crate_name, &highest_version);
                    
                    match get_index(&highest_path, &highest_update_path, &crate_name, &highest_ver_str){
                        // Err(ref e) if e. == std::io::ErrorKind::NotFound => {
                            
                        // },
                        Err(why) => {
                            db_handler.insert_error(format!("{:?}", why), &crate_name, &highest_ver_str);
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
