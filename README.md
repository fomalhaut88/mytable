Mytable implements an approach to store struct instances into a binary file.
Generally, the file represents a database with one table with some limitations
that are made for simplicity, minimization of the space and high performance
on add a new record, search, update or iterate. So it can be successfully used
for the purposes like logging. Notice: the library does not provide a way to
delete records from the table, so if you need it you should implement it by
your own.

## Basic example

### Define the structure

First, you need to define a structure. Once you did it
and started to insert the data you cannot modify the fields, otherwise the
stored data will be broken.

```rust
use mytable::{Table, TableTrait, Varchar};

#[derive(Debug, Copy, Clone)]
struct Person {
    id: usize,
    name: Varchar<20>,
    age: u32,
}
```

Second, you should implement **TableTrait** for the structure specifying
the work with **id** field:

```rust
impl TableTrait for Person {
    fn id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}
```

After that, it is a good idea to implement the method **new**
to create instances:

```rust
impl Person {
    fn new(name: &str, age: u32) -> Self {
        Self { id: 0, name: Varchar::<20>::new(name), age }
    }
}
```

### Work with the data

Create a table object:

```rust
let table = Table::new::<Person>("person.tbl");
```

Insert a record:

```rust
// Create a person
let mut alex = Person::new("alex", 32);
println!("Person before instert: {:?}", alex);

// Insert
alex.insert(&table).unwrap();
println!("Person after insert: {:?}", alex);
```

Update a record:

```rust
alex.age = 33;
alex.update(&table).unwrap();
```

Get record by id:

```rust
let person = Person::get(&table, 1).unwrap();
println!("Person extracted: {:?}", person);
```

Iterate all records:

```rust
for person in Person::all(&table) {
    println!("Person iterated: {:?}", person);
}
```

Iterate the records between two values of a sorted field.

```rust
for person in Person::iter_between(&table, 5, 10, &|person| person.id) {
    println!("Person iterated: {:?}", person);
}
```

### Work with index

First, it is a good idea to implement an *insert* method for the structire that
includes add to index. Also on update age it is necessary to make some changes
in the index tree, so we need to implement a special method for it.

```rust
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
```

After that we should define the inde table.

```rust
let age_index = Table::new::<TableIndex::<u32>>("person-age-index.tbl");
```

To insert a record with index:

```rust
let mut alex = Person::new("alex", 32);
alex.insert_with_index(&table, &age_index).unwrap();
```

To update the record with index:

```rust
alex.update_age(33, &age_index).unwrap();
alex.update(&table).unwrap();
```

To iterate records ordered by the index:

```rust
for id in TableIndex::<u32>::iter(&age_index) {
    println!("{:?}", Person::get(&table, id).unwrap());
}
```

To iterate records ordered by the index between two values:

```rust
for id in TableIndex::<u32>::iter_between(&age_index, &30, &35) {
    println!("{:?}", Person::get(&table, id).unwrap());
}
```

To search for a first record by the value:

```rust
let id = TableIndex::<u32>::search_one(&age_index, &30).unwrap();
println!("{:?}", Person::get(&table, id).unwrap());
```

To search for all records with the value:

```rust
for id in TableIndex::<u32>::search_many(&age_index, &30) {
    println!("{:?}", Person::get(&table, id).unwrap());
}
```
