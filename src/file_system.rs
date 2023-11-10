use super::*;
use std::collections::{HashSet,BTreeSet};

#[derive(Debug,Clone,PartialEq)]
pub struct SystemRuntime{
    pub running_app_paths:HashSet<String>,
    file_system:FileSystem,
    task_bar_paths:Vec<String>,
}

impl SystemRuntime{
    pub fn new(file_system:FileSystem,task_bar_paths:Vec<String>) -> Self {
        Self{
            running_app_paths:HashSet::new(),
            task_bar_paths,
            file_system,
        }
    }
    pub fn swap_taskbar(&mut self, swappee_path:String,swapped_path:String) {
        let (swappee_idx,_) = self.task_bar_paths.iter().enumerate()
            .find(|(i,item)|item==&&swappee_path).unwrap();
        let (swapped_idx,_) = self.task_bar_paths.iter().enumerate()
            .find(|(i,item)|item==&&swapped_path).unwrap();
        self.task_bar_paths.swap(swappee_idx,swapped_idx);
    }
    pub fn task_bar_paths(&self) -> Vec<String> {
        self.task_bar_paths.clone()
    }
    pub fn run_app(&mut self, path:&str) {
        if let Some(file) = self.file_system.tree.get_mut(path) {
             // fix this
                    //file.metadata.accessed=
            self.running_app_paths.insert(String::from(path));
        }
  
    }
    pub fn app_img_src(&self,path:&str) -> String {
        self.file_system.get_file_metadata(path).unwrap().img_src.clone()
    }
    pub fn close_app(&mut self, path:String) {
        self.running_app_paths.remove(&path);
    }

}

use std::collections::BTreeMap;
#[derive(Debug,PartialEq,Clone)]
pub struct Metadata{
    pub accessed:i64,
    pub created:i64,
    pub modified:i64,
    pub file_type:FileType,
    pub img_src:String,
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum FileType{
    Directory,
    File,
    SymbolicLink,
}

impl Metadata{
    pub fn is_dir(&self) -> bool {
        &self.file_type == &FileType::Directory
    }
    pub fn is_file(&self) -> bool {
        &self.file_type == &FileType::File
    }
    pub fn is_sym(&self) -> bool {
        &self.file_type == &FileType::SymbolicLink
    }
}
// Define the file system node
#[derive(Debug,Clone,PartialEq)]
struct FileSystemNode {
    pub name: String,
    pub metadata: Metadata, // This struct provides metadata information about a file.
}

// Define the filesystem as a B-tree map
#[derive(Debug,PartialEq,Clone)]
pub struct FileSystem {
    tree: BTreeMap<String, FileSystemNode>,
}

impl FileSystem {
    // Creates a new, empty FileSystem
    pub fn new() -> FileSystem {
        FileSystem {
            tree: BTreeMap::new(),
        }
    }

    // Adds a file to the filesystem
    pub fn add_file(&mut self, path: String, metadata: Metadata) {
        let file_name = path.split('/').last().unwrap_or_default().to_string();
        let file_node = FileSystemNode {
            name: file_name,
            metadata,
        };
        self.tree.insert(path, file_node);
    }

    // Retrieves a file's metadata from the filesystem
    pub fn get_file_metadata(&self, path: &str) -> Option<&Metadata> {
        self.tree.get(path).map(|node| &node.metadata)
    }

    // Lists all the files in the filesystem
    pub fn list_files(&self) {
        for (path, file_node) in &self.tree {
            println!("{}: {:?}", path, file_node);
        }
    }

    // Removes a file from the filesystem
    pub fn remove_file(&mut self, path: &str) {
        self.tree.remove(path);
    }
}