use crate::taskbar::{ TaskBarDataList};
use crate::topbar::{TopBarField, ProgramTopBarData};

use super::*;
use std::collections::{HashSet,HashMap};
use std::str::FromStr;
#[derive(Debug,Clone,PartialEq)]
pub struct SystemRuntime{
    pub active_proccesses:ActiveProccesses,
    pub file_system:FileSystem,
    pub selected_file_id:Option<Uuid>,
    pub settings:SystemSettings,
    pub program_top_bar:Option<ProgramTopBarData>,
    pub task_bar_items:TaskBarDataList,
}

#[derive(Debug,PartialEq,Clone,Default)]
pub struct SystemSettings{
    pub desktop:DesktopSettings,
    pub taskbar:TaskBarSettings,
    // maps file id to folder settings, (if file is a fodler)
    pub folder:HashMap<Uuid,FolderSettings>
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum FileSortBy{
    None,
    SnapToGrid,
    Name,
    Kind,
    DateLastOpened,
    DateModified,
    DateCreated,
}
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum FolderView{
    AsIcons{
        size:f32,
        spacing:f32,
        text_size:f32,
    },
    AsList{
        line_height:f32,
    },
    AsColumns{
        line_height:f32,
    },
    AsGallery{
        icon_size:f32,
    }
}
#[derive(Debug,PartialEq,Clone,Copy)]
pub struct DesktopSettings{
    pub icon_size:f32,
    pub sort_by:FileSortBy,
    pub use_stacks:bool,
}
impl Default for DesktopSettings {
    fn default() -> Self {
        Self { icon_size: 3., sort_by: FileSortBy::SnapToGrid, use_stacks: false }
    }
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub struct FolderSettings{
    pub view:FolderView,
    pub sort_by:FileSortBy,
}
impl Default for FolderSettings {
    fn default() -> Self {
        Self { view: FolderView::AsIcons { size: 3., spacing: 1., text_size: 1. }, sort_by: FileSortBy::SnapToGrid }
    }
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub struct TaskBarSettings{
    pub icon_size:f32,
    pub magnification:f32,
}
impl Default for TaskBarSettings {
    fn default() -> Self {
        Self { icon_size: 3., magnification: 1.5 }
    }
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
impl SystemRuntime {
    pub fn new(file_system:FileSystem,task_bar_ids:Vec<Uuid>) -> Self {
        Self {
            active_proccesses:ActiveProccesses::new(),
            file_system,
            selected_file_id:None,
            settings:SystemSettings::default(),
            program_top_bar:None,
            task_bar_items: TaskBarDataList::new(task_bar_ids)
        }
    }

    pub fn select_file(&mut self,file_id:Uuid)  {
        self.selected_file_id=Some(file_id);
    }   

    pub fn file_type(&self,file_id:Uuid) -> FileType {
        self.file_system.tree.get(&file_id).unwrap().metadata.file_type
    }

 

    pub fn is_running(&self,id:Uuid) -> bool {
        self.active_proccesses.0.contains_key(&id)
    }
    
  

  

    pub fn task_bar_ids(&self) -> Vec<Uuid> {
        self.task_bar_items.list()
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
        self.task_bar_items.list().contains(&id)
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

 



    // Removes a file from the filesystem
    pub fn remove_file(&mut self, id:Uuid) {
        self.tree.remove(&id);
    }
}