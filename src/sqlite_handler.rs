use rusqlite::{params, Connection, Result};
use rusqlite::NO_PARAMS;

// use chrono::prelude::*;

use crate::index_calculator::Metrics;

#[derive(PartialEq)]
#[repr(u8)]
pub enum MainCrateState{
    Unknown = 0,
    Downloaded = 1,
    DepsPrepped = 2,
    Built = 3,
    BaseCallgraph = 4,
    FullCallgraph = 5,
}

impl MainCrateState {
    pub fn from(mode: u8) -> MainCrateState {
        match mode {
            0 => MainCrateState::Unknown,
            1 => MainCrateState::Downloaded,
            2 => MainCrateState::DepsPrepped,
            3 => MainCrateState::Built,
            4 => MainCrateState::BaseCallgraph,
            5 => MainCrateState::FullCallgraph,
            _ => MainCrateState::Unknown,
        }
    }
}

pub struct SqliteHandler{
    conn: rusqlite::Connection
}

impl SqliteHandler{
    pub fn new(database: &str) -> Self {
        let conn = Connection::open(database).unwrap();
        // let conn = Connection::open("/database/prazi.db").unwrap();
        // let conn = Connection::open("prazi.db").unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                crate_name VARCHAR(256),
                crate_version VARCHAR(256),
                total_func_count INT,
                local_func_count INT,
                std_func_count INT,
                total_dep_func_count INT,
                used_dep_func_count INT,
                total_dep_public_func_count INT,
                used_dep_public_func_count INT,
                total_LOC INT,
                local_LOC INT,
                total_dep_LOC INT,
                used_dep_LOC INT,
                total_std_LOC INT,
                total_public_LOC INT,
                used_public_LOC INT
            )",
            NO_PARAMS,
        ).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS dep_metrics (
                name VARCHAR(100) NOT NULL,
                version VARCHAR(100) NOT NULL,
                total_count INT NOT NULL,
                used_count INT NOT NULL,
                total_LOC INT NOT NULL,
                used_LOC INT NOT NULL,
                crate_id INT NOT NULL,
                FOREIGN KEY(crate_id) REFERENCES metrics(id)
            )",
            NO_PARAMS,
        ).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metric_errors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name VARCHAR(100) NOT NULL,
                version VARCHAR(100) NOT NULL,
                error VARCHAR(256) NOT NULL
            )",
            NO_PARAMS,
        ).unwrap();

        SqliteHandler { conn : conn }
    }

    pub fn insert_error(&self, error: String, crate_name: &String, crate_version: &String){
        let result = self.conn.execute(
            "INSERT INTO metric_errors (
                name, 
                version, 
                error)
                VALUES(?1, ?2, ?3)",
            params![crate_name,
                crate_version,
                error]
        );
        match result {
            Err(why) => println!("{:?}", why),
            Ok(_val) => return
        }
    }
    
    pub fn insert_metric(&self, metrics: &Metrics, crate_name: &String, crate_version: &String){
        let result = self.conn.execute(
            "INSERT INTO metrics (
                crate_name, 
                crate_version, 
                total_func_count, 
                local_func_count, 
                std_func_count, 
                total_dep_func_count, 
                used_dep_func_count,
                total_dep_public_func_count,
                used_dep_public_func_count,
                total_LOC,
                local_LOC,
                total_dep_LOC,
                used_dep_LOC,
                total_std_LOC,
                total_public_LOC,
                used_public_LOC) 
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![crate_name,
                crate_version,
                metrics.TotalFuncCount as u32,
                metrics.LocalFuncCount as u32,
                metrics.StdFuncCount as u32,
                metrics.TotalDepFuncCount as u32,
                metrics.UsedDepFuncCount as u32,
                metrics.TotalDepPublicFuncCount as u32,
                metrics.UsedDepPublicFuncCount as u32,
                metrics.TotalLOC as u32,
                metrics.LocalLOC as u32,
                metrics.TotalDepLOC as u32,
                metrics.UsedDepLOC as u32,
                metrics.TotalStdLOC as u32,
                metrics.TotalDepPublicLOC as u32,
                metrics.UsedDepPublicLOC as u32]
        );
        match result {
            Err(why) => println!("{:?}", why),
            Ok(_) => {
                let id = self.conn.last_insert_rowid();
                for dep_metric in &metrics.depMetrics{
                    let res = self.conn.execute(
                        "INSERT INTO dep_metrics (name, version, total_count, used_count, total_LOC, used_LOC, crate_id) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![&dep_metric.crate_name,
                            dep_metric.crate_version,
                            dep_metric.totalCount as u32,
                            dep_metric.usedCount as u32,
                            dep_metric.total_loc as u32,
                            dep_metric.used_loc as u32,
                            id]
                    );
                    match res {
                        Err(why) => {
                            println!("{:?}", why);
                        },
                        Ok(_) => ()
                    }
                }
            }
        }
    }

    // pub fn insert_error(&self, crate_name: &String, error_message: &String, status: String){
    //     let result = self.conn.execute(
    //         "INSERT INTO compiler_errors VALUES(?1, ?2, ?3, ?4, ?5)",
    //         params![crate_name, 
    //             Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(), 
    //             error_message, 
    //             status]
    //     );
    //     match result {
    //         Err(why) => println!("{:?}", why),
    //         Ok(_val) => return
    //     }
    // }
}