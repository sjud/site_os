use super::*;

use std::collections::BTreeMap;
#[derive(Debug)]
pub struct Metadata{
    pub accessed:i64,
    pub created:i64,
    pub modified:i64,
    pub file_type:FileType,
}
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum FileType{
    Directory,
    File,
    SymbolicLink,
}
impl MetaData{
    pub fn is_dir(&self) -> bool {
        &self.file_type == Self::Directory
    }
    pub fn is_file(&self) -> bool {
        &self.file_type == Self::File
    }
    pub fn is_sym(&self) -> bool {
        &sel.file_typef == Self::SymbolicLink
    }
}
// Define the file system node
#[derive(Debug)]
struct FileSystemNode {
    pub name: String,
    pub metadata: Metadata, // This struct provides metadata information about a file.
}

// Define the filesystem as a B-tree map
struct FileSystem {
    tree: BTreeMap<String, FileSystemNode>,
}

impl FileSystem {
    // Creates a new, empty FileSystem
    fn new() -> FileSystem {
        FileSystem {
            tree: BTreeMap::new(),
        }
    }

    // Adds a file to the filesystem
    fn add_file(&mut self, path: String, metadata: Metadata) {
        let file_name = path.split('/').last().unwrap_or_default().to_string();
        let file_node = FileSystemNode {
            name: file_name,
            metadata,
        };
        self.tree.insert(path, file_node);
    }

    // Retrieves a file's metadata from the filesystem
    fn get_file_metadata(&self, path: &str) -> Option<&Metadata> {
        self.tree.get(path).map(|node| &node.metadata)
    }

    // Lists all the files in the filesystem
    fn list_files(&self) {
        for (path, file_node) in &self.tree {
            println!("{}: {:?}", path, file_node);
        }
    }

    // Removes a file from the filesystem
    fn remove_file(&mut self, path: &str) {
        self.tree.remove(path);
    }
}