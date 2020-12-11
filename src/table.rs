use std::{fs, io, iter};
use std::os::unix::prelude::FileExt;

use crate::table_trait::TableTrait;


/// Table is represented as a struct with the information about the path,
/// block size and the file object.
#[derive(Debug)]
pub struct Table {
    path: String,
    block_size: usize,
    file: fs::File
}


impl Table {
    /// Creates or opens a file to work. **block_size** is the size of record
    /// in bytes.
    pub fn new<T: TableTrait>(path: &str) -> Self {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path).unwrap();
        Self {
            path: path.to_string(),
            block_size: T::block_size(),
            file
        }
    }

    /// The number of records inserted.
    pub fn size(&self) -> usize {
        self.file.metadata().unwrap().len() as usize / self.block_size
    }

    /// Table is empty
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    /// Gets bytes of a record by its index.
    pub fn get(&self, idx: usize) -> Result<Vec<u8>, io::Error> {
        let mut block: Vec<u8> = vec![0; self.block_size];
        self.file.read_exact_at(&mut block, (idx * self.block_size) as u64)?;
        Ok(block)
    }

    /// Inserts data bytes to the end of file.
    pub fn append(&self, block: &[u8]) -> Result<usize, io::Error> {
        let idx = self.size();
        self.file.write_all_at(block, (idx * self.block_size) as u64)?;
        Ok(idx)
    }

    /// Updates data bytes located by the index.
    pub fn update(
                &self,
                block: &[u8],
                idx: usize
            ) -> Result<(), io::Error> {
        self.file.write_all_at(block, (idx * self.block_size) as u64)?;
        Ok(())
    }

    /// Iterates all records as data blocks.
    pub fn iter(&self) -> Box<dyn Iterator<Item = Vec<u8>> + '_> {
        self.iter_between(0, self.size()).unwrap()
    }

    /// Iterates records as data blocks between given indices
    /// (**>= idx_from** and **< idx_to**).
    pub fn iter_between(
                &self,
                idx_from: usize,
                idx_to: usize
            ) -> Result<
                Box<dyn Iterator<Item = Vec<u8>> + '_>,
                io::Error
            > {
        let mut idx = idx_from;

        Ok(Box::new(iter::from_fn(move || {
            let result;
            if idx < idx_to {
                let block = self.get(idx).unwrap();
                result = Some(block);
                idx += 1;
            } else {
                result = None;
            }
            result
        })))
    }

    /// Finds an index of a first block that has the given **value**.
    /// The function **get_value** extracts the value to compate from a block.
    pub fn find_sorted<T: PartialOrd>(
                &self,
                value: T,
                get_value: &dyn Fn(&[u8]
            ) -> T) -> usize {
        let mut idx = 0;
        let mut size = self.size();

        while size > 0 {
            let block = self.get(idx + size / 2).unwrap();

            if value > get_value(&block) {
                idx += size / 2 + 1;
                size = size / 2 + size % 2 - 1;
            } else {
                size = size / 2;
            }
        }

        idx
    }
}
