use std::path::{Path, PathBuf};
use std::{fs, io};
use std::ops::Deref;

pub struct FileStorageConfig {
    data_path: PathBuf,
}

impl FileStorageConfig {
    pub fn new() -> Self {
        Self {
            data_path: PathBuf::from("./data"),
        }
    }

    pub fn data_path(mut self, data_path: String) -> Self {
        self.data_path = PathBuf::from(data_path);
        self
    }
}

#[derive(Debug)]
pub struct FileStorage {
    data_path: PathBuf,
}

impl FileStorage {
    pub fn new(FileStorageConfig { data_path }: FileStorageConfig) -> Result<Self, io::Error> {
        let path = Path::new(&data_path);
        if !path.exists() {
            Self::create_dir(&path)?
        }
        Ok(Self { data_path: PathBuf::from(path) })
    }


    pub fn create_bucket(&self, name: &Path) -> io::Result<()> {
        Self::create_dir(&self.data_path.join(name).deref())
    }

    pub fn bucket_exists(&self, path: &Path) -> bool {
        self.data_path.join(path).exists()
    }

    pub fn delete_bucket(&self, name: &Path) -> io::Result<()> {
        fs::remove_dir_all(self.data_path.join(name).deref())
    }

    fn create_dir(path: &Path) -> io::Result<()> {
        if !&path.exists() {
            println!("creating data folder {:?}", path);
            fs::create_dir(&path)?;
        }
        Ok(())
    }
}
