use std::{mem, slice, io};

use crate::table::Table;


/// There are methods to insert, update, extract, iterate (and some other)
/// a structure object in the table. It requires **id** and **set_id** to be
/// implemented.
pub trait TableTrait where Self: Sized + Copy {
    /// Gets a unique id of the record.
    fn id(&self) -> usize;

    /// Sets id to the record.
    fn set_id(&mut self, id: usize);

    /// Returns size of the record in bytes.
    fn block_size() -> usize {
        mem::size_of::<Self>()
    }

    /// Represents the record as bytes slice.
    fn as_bytes(&self) -> &[u8] {
        let pointer = (self as *const Self) as *const u8;
        unsafe {
            slice::from_raw_parts(pointer, Self::block_size())
        }
    }

    /// Constructs the record from bytes slice.
    fn from_bytes(block: &[u8]) -> Self {
        let pointer = (block as *const [u8]) as *const Self;
        unsafe {
            slice::from_raw_parts(pointer, Self::block_size())[0]
        }
    }

    /// Gets first (the earliest) record from the table.
    fn get_first(table: &Table) -> Result<Self, io::Error> {
        Self::get(table, 1)
    }

    /// Gets id of the first record. Returns 0 if there is no record.
    fn get_first_id(table: &Table) -> Result<usize, io::Error> {
        if table.empty() {
            Err(io::Error::new(io::ErrorKind::NotFound, "empty table"))
        } else {
            Ok(1)
        }
    }

    /// Gets index of the block in the table by given id.
    fn get_index_by_id(
                table: &Table,
                id: usize
            ) -> Result<usize, io::Error> {
        if (id > 0) && (id <= table.size()) {
            Ok(id - 1)
        } else {
            Err(
                io::Error::new(io::ErrorKind::NotFound, id.to_string())
            )
        }
    }

    /// Extracts the record from the table by id.
    fn get(table: &Table, id: usize) -> Result<Self, io::Error> {
        if id > table.size() {
            return Err(
                io::Error::new(io::ErrorKind::NotFound, id.to_string())
            );
        }

        let idx = Self::get_index_by_id(table, id)?;
        let block = table.get(idx)?;
        let obj = Self::from_bytes(&block);

        Ok(obj)
    }

    /// Inserts the record to the table.
    fn insert(&mut self, table: &Table) -> Result<usize, io::Error> {
        if self.id() != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "id"));
        }
        let idx = table.append(&self.as_bytes())?;
        self.set_id(idx + 1);
        table.update(&self.as_bytes(), idx)?;
        Ok(self.id())
    }

    /// Updates the record in the table.
    fn update(&self, table: &Table) -> Result<(), io::Error> {
        let idx = Self::get_index_by_id(table, self.id())?;
        table.update(&self.as_bytes(), idx)
    }

    /// Iterates all records from the table.
    fn all(table: &Table) -> Box<dyn Iterator<Item = Self> + '_> {
        Box::new(table.iter().map(
            |block| Self::from_bytes(&block)
        ))
    }

    /// Iterates the records from the table between two values
    /// that can be extracted from a record by the function
    /// **get_sorted_value**. The values must be sorted.
    fn iter_between<'a, T: PartialOrd>(
                table: &'a Table,
                sorted_value_from: T,
                sorted_value_to: T,
                get_sorted_value: &'a dyn Fn(&Self) -> T
            ) -> Box<dyn Iterator<Item = Self> + 'a> {
        let idx_from = table.find_sorted(
            sorted_value_from,
            &|block| get_sorted_value(&Self::from_bytes(&block))
        );
        let idx_to = table.find_sorted(
            sorted_value_to,
            &|block| get_sorted_value(&Self::from_bytes(&block))
        );

        Box::new(table.iter_between(idx_from, idx_to).unwrap().map(
            |block| Self::from_bytes(&block)
        ))
    }
}


#[cfg(test)]
mod tests {
    use std::fs;

    use crate::varchar::*;
    use super::*;

    const TABLE_PATH: &str = "test-trait-person.tbl";

    #[derive(Debug, Copy, Clone)]
    struct Person {
        id: usize,
        name: Varchar<20>,
        age: u32,
    }

    impl TableTrait for Person {
        fn id(&self) -> usize {
            self.id
        }

        fn set_id(&mut self, id: usize) {
            self.id = id;
        }
    }

    impl Person {
        fn new(name: &str, age: u32) -> Self {
            Self { id: 0, name: Varchar::<20>::new(name), age }
        }
    }

    #[test]
    fn test_basic() {
        _ensure_removed_table_file();

        let table = Table::new::<Person>(TABLE_PATH);

        let mut alex = Person::new("alex", 32);

        // Insert
        alex.insert(&table).unwrap();
        assert_eq!(alex.id, 1);
        assert_eq!(table.size(), 1);

        // Update
        alex.age = 33;
        alex.update(&table).unwrap();
        let alex2 = Person::get(&table, 1).unwrap();
        assert_eq!(alex2.id, 1);
        assert_eq!(alex2.age, 33);

        // All
        let persons: Vec<Person> = Person::all(&table).collect();
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].id, 1);

        _ensure_removed_table_file();
    }

    fn _ensure_removed_table_file() {
        if fs::metadata(TABLE_PATH).is_ok() {
            fs::remove_file(TABLE_PATH).unwrap();
        }
    }
}
