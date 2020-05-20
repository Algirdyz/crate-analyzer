use serde_json::{to_string_pretty, from_reader};
use serde::{Serialize, Deserialize};
use std::fs::{File, write, copy};
use std::path::PathBuf;
use std::io::{BufReader};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::str;
use cargo_lock::Lockfile;
use semver::{Version};
use std::error;
use std::{thread, time};


type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Deserialize, Serialize, Clone)]
pub struct Node {
    pub id: usize,
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub crate_name: String,
    pub relative_def_id: String,
    pub inward_edges: Vec<Edge>,
    pub outward_edges: Vec<Edge>,
    pub num_lines: isize,
    pub is_externally_visible: bool,
    pub node_type: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Edge {
    pub target: usize,
    pub some_bool: bool
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CrateData {
    pub usedCount: usize,
    pub totalCount: usize
}

pub struct DepMetric {
    pub crate_name: String,
    pub crate_version: String,
    pub usedCount: usize,
    pub totalCount: usize,
    pub total_loc: usize,
    pub used_loc: usize
}

pub struct Metrics {
    pub TotalFuncCount: usize,
    pub LocalFuncCount: usize,
    pub StdFuncCount: usize,
    pub TotalDepFuncCount: usize,
    pub UsedDepFuncCount: usize,
    pub TotalDepPublicFuncCount: usize,
    pub UsedDepPublicFuncCount: usize,
    pub TotalDepLOC: usize,
    pub TotalLOC: usize,
    pub TotalStdLOC: usize,
    pub depMetrics: Vec<DepMetric> 
}

// #[derive(Deserialize, Serialize, Clone)]
// pub struct Graph {
//     pub nodes: HashMap<String, Node>,
//     pub edges: Vec<Vec<String>>,
//     pub nodes_info: Vec<NodeInfo>
// }

pub fn get_index(callgraph_directory: &PathBuf, crate_name: &String, crate_version: &String) -> Result<Metrics>{
    let deps = get_deps(&callgraph_directory, crate_name, Version::parse(crate_version).unwrap())?;

    let callgraph_path = update_with_python(callgraph_directory)?;    
    let graph = analyze_graph_for_package(&callgraph_path, crate_name)?;

    let mut output = Metrics{
        TotalFuncCount: graph.iter().count(),
        LocalFuncCount: graph.iter().filter(|n| n.node_type == Some("local_func".to_string())).count(),
        StdFuncCount: graph.iter().filter(|n| n.package_name == None).count(),
        TotalDepFuncCount: graph.iter().filter(|n| n.package_name != None && &n.crate_name != crate_name).count(),
        UsedDepFuncCount: graph.iter().filter(|n| n.package_name != None && n.node_type == Some("used_dep_func".to_string())).count(),
        TotalDepPublicFuncCount: graph.iter().filter(|n| n.package_name != None && &n.crate_name != crate_name && n.is_externally_visible).count(),
        UsedDepPublicFuncCount: graph.iter().filter(|n| n.package_name != None && n.node_type == Some("used_dep_func".to_string()) && n.is_externally_visible).count(),
        TotalDepLOC: graph.iter().filter(|n| n.package_name != None && &n.crate_name != crate_name).map(|n| if n.num_lines >= 0 { n.num_lines } else { 0 } as usize).sum(),
        TotalLOC: graph.iter().map(|n| if n.num_lines >= 0 { n.num_lines } else { 0 } as usize).sum(),
        TotalStdLOC: graph.iter().filter(|n| n.package_name == None).map(|n| if n.num_lines >= 0 { n.num_lines } else { 0 } as usize).sum(),
        depMetrics: Vec::new()
    };
    // let total_count = graph.iter().count();
    // let total_non_std = graph.iter().filter(|n| n.package_name != None).count();
    // let non_used_count = graph.iter().filter(|n| n.package_name != None && n.node_type == None).count();
    // let total_dep_func_count = graph.iter().filter(|n| n.package_name != None && &n.crate_name != crate_name).count();
    // let local_count = graph.iter().filter(|n| n.node_type == Some("local_func".to_string())).count();
    // let used_dep_count = graph.iter().filter(|n| n.package_name != None && n.node_type == Some("used_dep_func".to_string())).count();
    // println!("Total func count     = {}", total_non_std);
    // println!("Local func count     = {}", local_count);
    // println!("Total dep func count = {}", total_dep_func_count);
    // println!("Used dep funcs Count = {}", used_dep_count);
    // println!("Unused dep funcs     = {}", non_used_count);
    // println!("Own code share       = {}", local_count as f32 / total_non_std as f32);
    // println!("Leanness index (n)   = {}", used_dep_count as f32 / total_dep_func_count as f32);
    // let total_dep_func_count_lines: isize = graph.iter().filter(|n| n.package_name != None && &n.crate_name != crate_name).map(|n| n.num_lines).sum();
    // let used_dep_count_lines: isize = graph.iter().filter(|n| n.package_name != None && n.node_type == Some("used_dep_func".to_string())).map(|n| n.num_lines).sum();
    // println!("Leanness index (l)   = {}", used_dep_count_lines as f32 / total_dep_func_count_lines as f32);
    // println!("Dependency index     = {}", total_dep_func_count as f32 / total_count as f32);


    for n in deps{
        let tr_deps = get_all_deps(&callgraph_directory, &n.0, Version::parse(&n.1).unwrap())?;
        let dep_graph = analyze_graph_for_package2(&callgraph_path, &n.0, crate_name, &tr_deps);
        let total = dep_graph.iter().filter(|n| n.node_type != None && n.node_type != Some("std_func".to_string())).count();
        let total_used = dep_graph.iter().filter(|n| n.node_type == Some("local_func_pub".to_string()) || n.node_type == Some("used_dep_func_pub".to_string())).count();
        let total_loc = dep_graph.iter().filter(|n| n.node_type != None && n.node_type != Some("std_func".to_string())).map(|n| if n.num_lines >= 0 { n.num_lines } else { 0 } as usize).sum();
        let used_loc = dep_graph.iter().filter(|n| n.node_type == Some("local_func_pub".to_string()) || n.node_type == Some("used_dep_func_pub".to_string())).map(|n| if n.num_lines >= 0 { n.num_lines } else { 0 } as usize).sum();
        
        output.depMetrics.push(
            DepMetric{
                crate_name: (&n.0).to_string(),
                crate_version: (&n.1).to_string(),
                totalCount: total,
                usedCount: total_used,
                total_loc: total_loc,
                used_loc: used_loc
            }
        )
        // println!("{}  = {}/{}", &n.0, total_used, total);
    }

    Ok(output)
}

fn analyze_graph_for_package(callgraph_path: &PathBuf, crate_name: &String) -> Result<Vec<Node>>{
    let mut dep_graph = get_call_graph(callgraph_path)?;
    let mut node_index: usize = 0;
    while dep_graph.len() > node_index{
        let node = dep_graph.get(node_index).unwrap();

        if &node.crate_name == crate_name{
            traverse_node_downwards(&mut dep_graph, node_index.clone(), crate_name, false);
        }

        node_index += 1;
    }

    Ok(dep_graph)
}

fn analyze_graph_for_package2(callgraph_path: &PathBuf, crate_name: &String, main_package: &String, deps: &Vec<(String, String)>) -> Vec<Node>{
    let mut dep_graph = get_call_graph(callgraph_path).expect("No graph?");
    let mut node_index: usize = 0;
    let mut private_list: Vec<usize> = Vec::new();

    while dep_graph.len() > node_index{
        let node = dep_graph.get(node_index).unwrap();
        let mut called = false;

        if &node.crate_name == crate_name && node.is_externally_visible{
            for e in &node.inward_edges{
                if main_package == &dep_graph.get(e.target).unwrap().crate_name{
                    called = true;
                    break;
                }
            }
            if called {
                traverse_node_downwards(&mut dep_graph, node_index.clone(), crate_name, true);
            }else{
                private_list.push(node_index);
            }
        }

        node_index += 1;
    }

    for mut d in &mut dep_graph{
        if d.node_type == None{
            for dep in deps{
                if &d.crate_name == &dep.0 && d.package_version == Some(dep.1.to_string()){
                    d.node_type = Some("local_func".to_string());
                }
            }
        }
    }

    for i in private_list{
        traverse_node_downwards(&mut dep_graph, i.clone(), crate_name, false);
    }

    dep_graph
}

fn traverse_node_downwards(graph: &mut Vec<Node>, node_index: usize, package_name: &String, public: bool){
    let mut current_level_index = 0;

    let mut current_level_indexes: Vec<usize> = Vec::new();
    current_level_indexes.push(node_index.clone());
    let mut next_level_indexes: Vec<usize> = Vec::new();


    while current_level_indexes.len() > current_level_index{ 
        let current_index = current_level_indexes.get(current_level_index).cloned().unwrap();       
        let current_node = graph.get_mut(current_index).unwrap();
        if public{
            if current_node.node_type == None{
                for edge in &current_node.outward_edges{
                    next_level_indexes.push(edge.target.clone());
                }
            }
            if &current_node.crate_name == package_name{
                current_node.node_type = Some("local_func_pub".to_string());
            }else if current_node.package_name == None{
                current_node.node_type = Some("std_func".to_string());
            }else {
                current_node.node_type = Some("used_dep_func_pub".to_string());
            }
        }else{
            match current_node.node_type {
                None => {
                    if &current_node.crate_name == package_name{
                        current_node.node_type = Some("local_func".to_string());
                    }else if current_node.package_name == None{
                        current_node.node_type = Some("std_func".to_string());
                    }else {
                        current_node.node_type = Some("used_dep_func".to_string());
                    }
    
                    for edge in &current_node.outward_edges{
                        next_level_indexes.push(edge.target.clone());
                    }
                },
                Some(_) => ()
            }
        }        

        if current_level_index + 1 == current_level_indexes.len(){
            // This is the last node in this row. We move to the next one.
            current_level_indexes.clear();
            current_level_indexes.append(&mut next_level_indexes);
            current_level_index = 0;
            continue;
        }else{
            current_level_index += 1;
        }
    }
}


fn update_with_python(path: &PathBuf) -> Result<PathBuf>{
    let res_path = path.join("updated_callgraph.json");
    if res_path.exists() {
        return Ok(res_path)
    }

    let output = Command::new("python3")
        .arg("grapher.py")
        .arg(path)
        .output()?;
    
    if output.status.success() {
        return Ok(res_path)
    }else{
        println!("{}", str::from_utf8(&output.stdout).unwrap());
        println!("{}", str::from_utf8(&output.stderr).unwrap());
        return Err("Bad request")?;
    }
}

fn get_call_graph(path: &PathBuf) -> Result<Vec<Node>>{
    if path.exists() {
        let file = File::open(path).expect("could not open graph file");
        let buffered_reader = BufReader::new(file);
        let res = from_reader(buffered_reader)?;
        return Ok(res);
    }else{
        return Err("Bad request")?;
    }
}

fn get_deps(crate_path: &PathBuf, crate_name: &String, version: Version) -> Result<Vec<(String, String)>>{
    let lockfile = Lockfile::load(crate_path.join("Cargo.lock"))?;
    let mut result: Vec<(String, String)> = Vec::new();

    let deps = lockfile.packages.iter().find(|d| &d.name.to_string() == crate_name && &d.version == &version).unwrap();

    for d in &deps.dependencies{
        result.push((d.name.to_string(), d.version.to_string()));
    }        

    Ok(result)   
}

fn get_all_deps(crate_path: &PathBuf, crate_name: &String, version: Version) -> Result<Vec<(String, String)>>{
    let lockfile = Lockfile::load(crate_path.join("Cargo.lock"))?;
    let mut result: Vec<(String, String)> = Vec::new();

    let deps = lockfile.packages.iter().find(|d| &d.name.to_string() == crate_name && &d.version == &version).unwrap();

    let mut current_level_index = 0;
    let mut current_level_deps: Vec<&cargo_lock::dependency::Dependency> = Vec::new();
    let mut next_level_deps: Vec<&cargo_lock::dependency::Dependency> = Vec::new();

    for tr_dep in &deps.dependencies{
        result.push((tr_dep.name.to_string(), tr_dep.version.to_string()));
        current_level_deps.push(tr_dep);
    }
    while current_level_deps.len() > current_level_index{ 
        let current_dep = current_level_deps.get(current_level_index).unwrap();

        
        // println!("Dep - {} - {}", current_dep.name, current_dep.version);
        let dependency = &lockfile.packages.iter().find(|d| d.name == current_dep.name && d.version == current_dep.version).unwrap();

        // let ten_millis = time::Duration::from_millis(100);
        // let now = time::Instant::now();

        // thread::sleep(ten_millis);

        for tr_dep in &dependency.dependencies{
            if !result.contains(&(tr_dep.name.to_string(), tr_dep.version.to_string())){
                result.push((tr_dep.name.to_string(), tr_dep.version.to_string()));
                next_level_deps.push(tr_dep);
            }
        }

        if current_level_index + 1 == current_level_deps.len(){
            // println!("Dropping level.");
            // This is the last node in this row. We move to the next one.
            current_level_deps.clear();
            current_level_deps.append(&mut next_level_deps);
            current_level_index = 0;
            continue;
        }else{
            current_level_index += 1;
        }
    }

    Ok(result)
}


// fn recursive_deps(lockfile: &Lockfile, deps: &Vec<cargo_lock::dependency::Dependency>) -> Vec<(String, String)>{
//     let mut result: Vec<(String, String)> = Vec::new();
//     for dep in deps{
//         let dependency = &lockfile.packages.iter().find(|d| &d.name == &dep.name && &d.version == &dep.version).unwrap();
//         // result.extend(dependency.dependencies.iter().copied());
//         if dependency.dependencies.len() > 0{
//             result.append(&mut recursive_deps(lockfile, &dependency.dependencies));
//         }
//         for tr_dep in &dependency.dependencies{
//             result.push((tr_dep.name.to_string(), tr_dep.version.to_string()));
//         }
//     }

//     result
// }