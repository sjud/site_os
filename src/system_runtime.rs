use super::*;
use std::collections::{HashSet,HashMap};
use std::str::FromStr;
#[derive(Debug,Clone,PartialEq)]
pub struct SystemRuntime{
    pub active_proccesses:ActiveProccesses,
    pub file_system:FileSystem,
}

#[derive(Debug,PartialEq,Clone)]
pub struct ActiveProccesses(pub HashMap<Uuid,ActiveProcess>);

impl ActiveProccesses{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

}

#[derive(Default,Debug,PartialEq,Clone,Copy)]
pub struct ActiveProcess{
    pub start_time:f64,
    pub window_stack_idx:usize,
    pub minimized:bool,
}
impl SystemRuntime{
    pub fn new(file_system:FileSystem) -> Self {
        Self {
            active_proccesses:ActiveProccesses::new(),
            file_system,
        }
    }
    pub fn file_type(&self,file_id:Uuid) -> FileType {
        self.file_system.tree.get(&file_id).unwrap().metadata.file_type
    }
    pub fn is_jumping(&self,id:Uuid) -> bool {
        self.file_system.tree.get(&id).as_ref().unwrap()
            .metadata.task_bar.as_ref().map(|t|t.is_jumping)
            .unwrap_or_default()
    }
    pub fn is_running(&self,id:Uuid) -> bool {
        self.active_proccesses.0.contains_key(&id)
    }
    pub fn set_jumping(&mut self,id:Uuid) {
        self.file_system.tree.get_mut(&id).unwrap()
            .metadata.task_bar.as_mut().map(|d|d.is_jumping = !d.is_jumping);
    }

    pub fn swap_taskbar(&mut self, id_a:Uuid,id_b:Uuid) {
        self.file_system.swap_taskbar(id_a, id_b);
    }

    pub fn task_bar_ids(&self) -> Vec<Uuid> {
        self.file_system.task_bar_ids()
    }
    pub fn file_ids_direct_children_of_path(&self,path:std::path::PathBuf) -> Vec<Uuid> {
        self.file_system.tree.iter().filter(|node|
            node.1.path.to_path_buf().parent().map(
                |parent| parent.to_path_buf() == path.to_path_buf()
            ).unwrap_or_default()).map(|node|node.1.file_id)
                .collect::<Vec<Uuid>>()
    }
    pub fn path_from_file_id(&self,file_id:Uuid) -> String {
        self.file_system.tree.get(&file_id).and_then(|node|node.path.to_str()).unwrap().to_string()
    }
    pub fn run_app(&mut self, id:Uuid,time:f64) {
        if let Some(file) = self.file_system.tree.get_mut(&id) {
            file.metadata.accessed = time;
            let stack_size = self.active_proccesses.0.len();
            let start_time = time;
            self.active_proccesses.0.insert(
                id,
                ActiveProcess { 
                start_time,
                window_stack_idx: stack_size+1,
                 minimized: false 
                }
            );
        }
    }

    pub fn close_app(&mut self, id:Uuid) {
        if let Some(idx) =  self.active_proccesses.0.remove(&id).map(|p|p.window_stack_idx) {
            for (file_id,process) in self.active_proccesses.0.iter_mut() {
                if process.window_stack_idx > idx {
                    process.window_stack_idx -= 1;
                }
            }
        }
    }

    pub fn file_is_in_taskbar(&self,id:Uuid) -> bool {
        if let Some(file) = self.file_system.tree.get(&id) {
            file.metadata.task_bar.is_some()
        } else {
            false
        }
    }
    pub fn img_src(&self,file_id:Uuid) -> String {
        self.file_system.tree.get(&file_id)
            .map(|node|node.metadata.img_src.clone()).unwrap_or("".to_string())
    }


  

}

use std::collections::BTreeMap;
#[derive(Debug,PartialEq,Clone,Default)]
pub struct Metadata{
    pub accessed:f64,
    pub created:f64,
    pub modified:f64,
    pub file_type:FileType,
    pub img_src:String,
    // if file has task bar data, it's in the taskbar.
    pub task_bar:Option<TaskBarData>
}

#[derive(Debug,PartialEq,Clone,Default)]
pub struct TaskBarData{
    pub is_jumping: bool,
    pub idx:usize,
}

#[derive(Debug,PartialEq,Clone,Copy,Default)]
pub enum FileType{
    Directory,
    #[default]
    File,
}

impl Metadata{
    pub fn is_dir(&self) -> bool {
        &self.file_type == &FileType::Directory
    }

    pub fn is_file(&self) -> bool {
        &self.file_type == &FileType::File
    }
}
// Define the file system node
#[derive(Debug,Clone,PartialEq)]
pub struct FileSystemNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub file_id: Uuid,
    pub metadata: Metadata, // This struct provides metadata information about a file.
}

impl FileSystemNode{
    pub fn is_in_taskbar(&self) -> bool {
        self.metadata.task_bar.is_some()
    }

}
// Define the filesystem as a B-tree map
#[derive(Debug,PartialEq,Clone)]
pub struct FileSystem {
    pub tree: BTreeMap<Uuid, FileSystemNode>,
}

impl FileSystem {
    // Creates a new, empty FileSystem
    pub fn new() -> FileSystem {
        FileSystem {
            tree: BTreeMap::new(),
        }
    }

    // Adds a file to the filesystem
    pub fn add_file(&mut self, 
        file_id:Uuid, 
        path: String,
        metadata: Metadata
    ) {
        let file_name = path.split('/').last().unwrap_or_default().to_string();
        let file_node = FileSystemNode {
            name: file_name,
            path:std::path::PathBuf::from_str(&path).unwrap(),
            file_id,
            metadata,
        };
        self.tree.insert(file_id, file_node);
    }

    // Retrieves a file's metadata from the filesystem
    pub fn get_file_metadata(&self, id: Uuid) -> Option<&Metadata> {
        self.tree.get(&id).map(|node| &node.metadata)
    }

    // Lists all the files in the filesystem
    pub fn list_files(&self) {
        for (path, file_node) in &self.tree {
            println!("{}: {:?}", path, file_node);
        }
    }

    pub fn swap_taskbar(&mut self,id_a:Uuid,id_b:Uuid) {
        let mut swap_files = self.tree.iter_mut().filter_map(|(_,node)|
            if node.file_id==id_a || node.file_id == id_b {
                Some(node)
            } else {None}
        ).collect::<Vec<&mut FileSystemNode>>();
        // 1 is if we drop it on itself..
        if swap_files.len() == 2 {
            let idx_a = swap_files[0].clone().metadata.task_bar.as_ref().unwrap().idx;
            let idx_b = swap_files[1].clone().metadata.task_bar.as_ref().unwrap().idx;
            swap_files[0].metadata.task_bar.as_mut().unwrap().idx = idx_b;
            swap_files[1].metadata.task_bar.as_mut().unwrap().idx = idx_a;
        }
    }

    pub fn task_bar_ids(&self) -> Vec<Uuid> {
        let mut unsorted_ids = self.tree.iter().filter_map(|(_,node)|if node.is_in_taskbar() {
            Some((node.file_id,node.metadata.task_bar.as_ref().map(|t|t.idx).unwrap()))
        } else {
            None
        }).collect::<Vec<(Uuid,usize)>>();
        unsorted_ids.sort_by(|a,b|a.1.cmp(&b.1));
        unsorted_ids.into_iter().map(|(id,_)|id).collect::<Vec<Uuid>>()
    }

    // Removes a file from the filesystem
    pub fn remove_file(&mut self, id:Uuid) {
        self.tree.remove(&id);
    }
}