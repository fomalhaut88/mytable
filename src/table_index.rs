use std::{io, iter};

use crate::table::*;
use crate::table_trait::*;


/// TableIndex is a record that has TableTrait implemented, so it keeps its
/// own table file and work as a table with fixed fields. Inside the binary
/// tree algorithms are implemented to insert, search and iterate.
#[derive(Debug, Copy, Clone)]
pub struct TableIndex<T> {
    id: usize,
    value: T,
    table_id: usize,
    left: usize,
    right: usize,
}


impl<T: Copy> TableTrait for TableIndex<T> {
    fn id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}


impl<'a, T: 'a + Copy + Clone + PartialOrd> TableIndex<T> {
    fn new(value: &T, table_id: usize) -> Self {
        Self {
            id: 0,
            value: value.clone(),
            table_id: table_id,
            left: 0,
            right: 0,
        }
    }

    /// Adds an index value to the table.
    pub fn add(
                table: &Table,
                value: &T,
                table_id: usize
            ) -> Result<(), io::Error> {
        let mut record = Self::new(value, table_id);
        let record_id = record.insert(table)?;
        Self::_bind(table, value, record_id);
        Ok(())
    }

    /// Searches for a node by **value**. The **id** of original
    /// record is returned.
    pub fn search_one(
                table: &Table,
                value: &T
            ) -> Result<usize, io::Error> {
        for table_id in Self::search_many(table, value) {
            return Ok(table_id);
        }
        return Err(io::Error::new(io::ErrorKind::NotFound, "table index"));
    }

    /// Searches for all nodes with given **value**.
    /// It returns an iterator that yields **id** of original records.
    pub fn search_many(
                table: &'a Table,
                value: &'a T
            ) -> Box<dyn Iterator<Item = usize> + 'a> {
        Box::new(
            Self::_iter_by_value(table, value)
                .filter(|rec| rec.table_id > 0)
                .map(|rec| rec.table_id)
        )
    }

    /// Iterates all nodes in the order of its values.
    pub fn iter(table: &'a Table) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut stack = vec![(Self::get_first(table).unwrap(), 0u8)];

        Box::new(iter::from_fn(move || {
            let mut result = None;

            while !stack.is_empty() {
                let last = stack.last_mut().unwrap();

                if last.1 == 0 {
                    last.1 = 1;
                    if last.0.left > 0 {
                        let rec = Self::get(table, last.0.left).unwrap();
                        stack.push((rec, 0));
                    }
                    continue;
                }

                if last.1 == 1 {
                    last.1 = 2;
                    if last.0.table_id > 0 {
                        result = Some(last.0.table_id);
                        break;
                    }
                    continue;
                }

                if last.1 == 2 {
                    last.1 = 3;
                    if last.0.right > 0 {
                        let rec = Self::get(table, last.0.right).unwrap();
                        stack.push((rec, 0));
                    }
                    continue;
                }

                if last.1 == 3 {
                    stack.remove(stack.len() - 1);
                    continue;
                }
            }

            result
        }))
    }

    /// Iterates the nodes in the order of its values between the given values
    /// (**>= values_from** and **< values_to**).
    pub fn iter_between(
                table: &'a Table,
                value_from: &'a T,
                value_to: &'a T
            ) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut stack = Self::_build_stack_from(table, value_from);

        Box::new(iter::from_fn(move || {
            let mut result = None;

            while !stack.is_empty() {
                let last = stack.last_mut().unwrap();

                if last.1 == 0 {
                    last.1 = 1;
                    if last.0.left > 0 {
                        let rec = Self::get(table, last.0.left).unwrap();
                        stack.push((rec, 0));
                    }
                    continue;
                }

                if last.1 == 1 {
                    last.1 = 2;
                    if last.0.value < *value_to {
                        if last.0.table_id > 0 {
                            result = Some(last.0.table_id);
                            break;
                        } else {
                            continue;
                        }
                    } else {
                        break;
                    }
                }

                if last.1 == 2 {
                    last.1 = 3;
                    if last.0.right > 0 {
                        let rec = Self::get(table, last.0.right).unwrap();
                        stack.push((rec, 0));
                    }
                    continue;
                }

                if last.1 == 3 {
                    stack.remove(stack.len() - 1);
                    continue;
                }
            }

            result
        }))
    }

    /// Excludes the node by setting its **table_id** to **0**.
    pub fn exclude(
                table: &Table,
                value: &T,
                table_id: usize
            ) -> Result<(), io::Error> {
        let rec_option = {
            let mut result = None;
            for rec in Self::_iter_by_value(table, value) {
                if rec.table_id == table_id {
                    result = Some(rec);
                    break;
                }
            }
            result
        };

        match rec_option {
            Some(mut rec) => {
                rec.table_id = 0;
                rec.update(table)?;
                Ok(())
            },
            None => {
                Err(io::Error::new(
                    io::ErrorKind::NotFound, table_id.to_string()
                ))
            }
        }
    }

    fn _bind(table: &Table, value: &T, record_id: usize) {
        let mut id = Self::get_first_id(table).unwrap();

        if id != record_id {
            while id > 0 {
                let mut rec = Self::get(table, id).unwrap();

                if *value < rec.value {
                    id = rec.left;
                    if id == 0 {
                        rec.left = record_id;
                    }
                } else {
                    id = rec.right;
                    if id == 0 {
                        rec.right = record_id;
                    }
                }

                if id == 0 {
                    rec.update(table).unwrap();
                }
            }
        }
    }

    fn _build_stack_from(table: &Table, value: &T) -> Vec<(Self, u8)> {
        let mut stack = Vec::new();

        let mut id = Self::get_first_id(table).unwrap();

        while id > 0 {
            let rec = Self::get(table, id).unwrap();

            if *value < rec.value {
                stack.push((rec, 1u8));
                id = rec.left;
            } else if *value > rec.value {
                stack.push((rec, 3u8));
                id = rec.right;
            } else {
                stack.push((rec, 1u8));
                break;
            }
        }

        stack
    }

    fn _iter_by_value(
                table: &'a Table,
                value: &'a T
            ) -> Box<dyn Iterator<Item = Self> + 'a> {
        let mut id = Self::get_first_id(table).unwrap();

        Box::new(iter::from_fn(move || {
            while id > 0 {
                let rec = Self::get(table, id).unwrap();

                if *value < rec.value {
                    id = rec.left;
                } else {
                    id = rec.right;

                    if *value == rec.value {
                        return Some(rec);
                    }
                }
            }
            return None;
        }))
    }

    // fn _iter_stack(table: &'a Table, stack: &'a mut Vec<(Self, u8)>) -> Box<dyn Iterator<Item = usize> + 'a> {
    //     Box::new(iter::from_fn(move || {
    //         let mut result = None;

    //         while !stack.is_empty() {
    //             let last = stack.last_mut().unwrap();

    //             if last.1 == 0 {
    //                 last.1 = 1;
    //                 if last.0.left > 0 {
    //                     let rec = Self::get(table, last.0.left).unwrap();
    //                     stack.push((rec, 0));
    //                 }
    //                 continue;
    //             }

    //             if last.1 == 1 {
    //                 last.1 = 2;
    //                 result = Some(last.0.table_id);
    //                 break;
    //             }

    //             if last.1 == 2 {
    //                 last.1 = 3;
    //                 if last.0.right > 0 {
    //                     let rec = Self::get(table, last.0.right).unwrap();
    //                     stack.push((rec, 0));
    //                 }
    //                 continue;
    //             }

    //             if last.1 == 3 {
    //                 stack.remove(stack.len() - 1);
    //                 continue;
    //             }
    //         }

    //         result
    //     }))
    // }
}


#[cfg(test)]
mod tests {
    use std::fs;

    use crate::varchar::*;
    use super::*;

    const TABLE_PATH: &str = "test-index-person.tbl";
    const TABLE_AGE_INDEX_PATH: &str = "test-index-person-age-index.tbl";

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

        fn insert_with_index(
                    &mut self,
                    table: &Table,
                    age_index: &Table
                ) -> Result<usize, io::Error> {
            let id = self.insert(table)?;
            TableIndex::add(age_index, &self.age, id)?;
            Ok(id)
        }

        fn update_age(
                    &mut self,
                    age: u32,
                    age_index: &Table
                ) -> Result<(), io::Error> {
            TableIndex::exclude(age_index, &self.age, self.id)?;
            TableIndex::add(age_index, &age, self.id)?;
            self.age = age;
            Ok(())
        }
    }

    #[test]
    fn test_table_index() {
        _ensure_removed_tables();

        let table = Table::new::<Person>(TABLE_PATH);
        let age_index = Table::new::<TableIndex::<u32>>(TABLE_AGE_INDEX_PATH);

        // Insert a person with index
        let mut alex = Person::new("alex", 32);
        alex.insert_with_index(&table, &age_index).unwrap();

        // Update value with index
        alex.update_age(33, &age_index).unwrap();
        alex.update(&table).unwrap();

        _ensure_removed_tables();
    }

    fn _ensure_removed_tables() {
        if fs::metadata(TABLE_PATH).is_ok() {
            fs::remove_file(TABLE_PATH).unwrap();
        }
        if fs::metadata(TABLE_AGE_INDEX_PATH).is_ok() {
            fs::remove_file(TABLE_AGE_INDEX_PATH).unwrap();
        }
    }
}
