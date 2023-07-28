//! # db_service
//! This crate implements a basic ORM and a Repository-Pattern.
//! Using `db_service`, all structs implementing the [Entity] trait can be stored in any database
//! architecture for which a [PersistentService] is implemented.

use std::collections::HashMap;
pub use std::marker::PhantomData;
pub use num_bigint::BigInt;

#[derive(Debug)]
/// Enum of possible errors that may occur on interaction with the Database.
/// The Errors contain a [String] of the error message.
pub enum DatabaseError {
    Connection(String),
    NotFound(String),
    InvalidSQL(String),
    InvalidParameter(String),
    AtInsert(String),
    AtQuery(String)
}

//TODO: Entity has to know its id!
/// All structs that should be able to be saved in the DB should implement this trait.
/// The struct of type T must hold a field that stores the ID of the struct.
/// Storing the ID is left to the implementation of T.
/// The implementation of [Into] should map the struct fields to column names (keys).
/// The implementation of [From] should set the struct fields according to the given column values
/// in the [HashMap]. Be aware that there can be None values.
/// The [Clone] trait can be simply derived and is used to avoid moving entities on
/// save or update.
/// As an example you can lookup the implementation of the [test_entity::TestEntity].
pub trait Entity: From<HashMap<String, Option<String>>> + Into<HashMap<String, String>> + Clone {

    /// Getter for the ID of the Entity. 
    /// # Return
    /// A [Option]<[BigInt]> reference to the ID. 
    /// The value will be [None] if no id is set yet.
    fn id(&self) -> &Option<BigInt>;

    /// A "setter" for the ID of the Entity. This function should not be used in client code!
    /// # Return
    /// A mutable [Option]<[BigInt]> reference that can be used to set the ID.
    fn id_mut(&mut self) -> &mut Option<BigInt>;
}

//TODO: why bigint as id?
/// This trait defines all functions that should be able to execute directly on the database.
/// You may implement this trait for any DB you want to use.
pub trait PersistentService {

    /// New instances of a PersistentService should only be created by using this function.
    /// This should only create a connection. If no database with the given name exists the function
    /// should return an error or panic. The connection management is left to the implementation.
    /// # Arguments
    /// * `db` - the database name.
    /// # Return
    /// A struct that implements PersistentService
    /// # Panics
    /// If the no database with the given name exists
    fn new(db: &str) -> Self;

    /// This function creates a new database, for a given SQL-Query.
    /// # Arguments
    /// * `query` - A SQL query as [String]
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if the provided SQL-Query is invalid.
    /// # Example
    /// ```no_run
    /// //You can load a sql file and pass it to the function
    /// use db_service::sqlite_service::SQLiteService;
    /// use db_service::PersistentService;
    ///
    /// let service = SQLiteService::new("DB_NAME");
    ///
    /// let query = String::from(include_str!("create_database.sql"));
    /// service.create_db(query).unwrap();
    /// ```
    fn create_db(&self, query: String) -> Result<(), DatabaseError>;

    /// This function closes the connection created with [`new()`](PersistentService::new) and makes the PersistentService
    /// unusable.
    /// # Errors
    /// * [`DatabaseError::Connection`] - on failure
    fn close_connection(&self) -> Result<(), DatabaseError>;

    /// Returns all rows of a given table.
    /// # Arguments
    /// * `table` - the table name
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given table
    ///                                     name
    /// # Return
    /// A [Vector](Vec) of [HashMap] with [String] keys representing the column name and [Option]
    /// Values as [String]s. The Values are [None] for NULL values.
    fn get_all(&self, table: &str)
        -> Result<Vec<HashMap<String, Option<String>>>, DatabaseError>;

    /// Queries on row with the given id from a given table.
    /// # Arguments
    /// * `table` - the table name as &str
    /// * `id` - the id as [BigInt] reference
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given table
    ///                                     name or id
    /// * [`DatabaseError::NotFound`] - if no row with the given id exists
    /// # Return
    /// A [HashMap] with [String] keys representing the column name and [Option]
    /// Values as [String]s. The Values are [None] for NULL values.
    fn get_by_id(&self, table: &str, id: &BigInt)
        -> Result<HashMap<String, Option<String>>, DatabaseError>;

    /// Executes an INSERT INTO with the given values.
    /// # Arguments
    /// * `table` - the table name as &str
    /// * `columns` - a [Vector](vec) of column names, may be empty if values are provided for all
    ///                 columns in the correct order
    /// * `values` - a [Vector](vec) of values as [String]s, the order has to match the order of
    ///                 columns
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the number of given columns dosen't match the
    ///                                         number of values, or the values can't be bind to
    ///                                         the SQL statement
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given
    ///                                     parameters
    /// * [`DatabaseError::AtQuery`] - at failure
    /// # Return
    /// The id of the inserted row as [BigInt]
    fn save(&self, table: &str, columns: Vec<String>, values: Vec<String>)
        -> Result<BigInt, DatabaseError>;

    /// Updates a row with the given values.
    /// # Arguments
    /// * `table` - the table name as &str
    /// * `id` - the row id as [BigInt] reference
    /// * `columns` - a [Vector](vec) of column names, may be empty if values are provided for all
    ///                 columns in the correct order
    /// * `values` - a [Vector](vec) of values as [String]s, the order has to match the order of
    ///                 columns
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the number of given columns dosen't match the
    ///                                         number of values, or the values can't be bind to
    ///                                         the SQL statement
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given
    ///                                     parameters
    /// * [`DatabaseError::AtQuery`] - at failure
    ///
    /// # Side effects
    /// This function will ignore the given id and execute an INSERT INTO using
    /// [`save()`](PersistentService::save), if no entry for the given id exists.
    /// # Return
    /// The number of the affected rows
    fn update(&self, table: &str, id: &BigInt, columns: Vec<String>, values: Vec<String>)
        -> Result<usize, DatabaseError>;

    /// Deletes a row with from a given table.
    /// # Arguments
    /// * `table` - the table name as &str
    /// * `id` - the row id as [BigInt] reference
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the number of given columns dosen't match the
    ///                                         number of values
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given
    ///                                     parameters, or id can't be bind to the SQL statement
    /// * [`DatabaseError::AtQuery`] - at failure
    /// # Return
    /// The number of the affected rows
    fn delete(&self, table: &str, id: &BigInt) -> Result<usize, DatabaseError>;
}

/// This struct corresponds with (function) Repos from the Repository Pattern.
/// It implements generically all functions that can be executed on a table.
///
/// # Example
///
/// ```no_run, compile_fail
/// //To create a new table and save entities set up a persistent service first.
/// //Database name and table names must fit your local database.
/// //After this you can use table.save() etc.
/// use db_service::sqlite_service::SQLiteService;
/// use db_service::{PersistentService, Table};
///
/// let service = SQLiteService::new("DB_NAME");
/// let table = Table::new("TABLE_NAME", &service);
/// ```
#[allow(dead_code)]
pub struct Table<'a, T: Entity, S: PersistentService> {
    table_name: String,
    service: &'a S,
    phantom: PhantomData<T>
}

#[allow(dead_code)]
impl<'a, T: Entity, S: PersistentService> Table<'a, T, S> {

    /// New instances of a [Table] should only be created by using this function.
    /// # Arguments
    /// * `table_name` - the table name, matching an existing table in the database
    /// * `persistent_service` - a reference to a struct that implements [PersistentService]
    /// # Return
    /// A [Table] that can interact with structs of type `T` using the given [PersistentService]
    pub fn new(table_name: &str, persistent_service: &'a S) -> Self {
        Table {
            table_name: table_name.to_string(),
            service: persistent_service,
            phantom: PhantomData::default()
        }
    }

    //TODO: exist()

    /// Queries on row with the given id from the table, using  [`PersistentService::get_by_id()`].
    /// # Arguments
    /// * `id` - the id as [BigInt] reference
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given table
    ///                                     name or id
    /// * [`DatabaseError::NotFound`] - if no row with the given id exists
    /// # Return
    /// An instance of `T` that that may contain unset fields, due to NULL values in the Database.
    pub fn get_by_id(&self, id: &BigInt) -> Result<T, DatabaseError> {
        let result = self.service.get_by_id(&self.table_name, id)?;
        Ok(result.into())
    }

    /// Returns all rows of this table, using [`PersistentService::get_all()`].
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given table
    ///                                     name
    /// # Return
    /// A [Vector](Vec) of instances of `T` that may contain unset fields, due to NULL values in the
    /// Database.
    pub fn get_all(&self) -> Result<Vec<T>, DatabaseError> {
        let query_result =  self.service.get_all(&self.table_name)?;
        Ok(query_result.iter().map(|result| T::from(result.clone())).collect())
    }

    /// Inserts a new entry into the table, using [`PersistentService::save()`].
    /// The function will ignore and override the current `id` inside of the given entity.
    /// # Arguments
    /// * `entity` - a mutable reference to the struct that should be saved
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the number of given columns dosen't match the
    ///                                         number of values, or the values can't be bind to
    ///                                         the SQL statement
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given
    ///                                     parameters
    /// * [`DatabaseError::AtQuery`] - at failure
    /// # Side effects
    /// This function will set the `id` of `entity`.
    /// # Return
    /// The id of the inserted entry as [BigInt]
    pub fn save(&self, entity: &mut T) -> Result<BigInt, DatabaseError> {
        let e: HashMap<String, String> = T::into(entity.clone());
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for (k, v) in e {
            columns.push(k);
            values.push(v);
        }

        let id = self.service.save(&self.table_name, columns, values)?;
        *entity.id_mut() = Some(id.clone());
        Ok(id)
    }

    /// Updates a row with the given values, using [`PersistentService::update()`]
    /// # Arguments
    /// * `entity` - a mutable reference to the struct that should be updated
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the number of given columns dosen't match the
    ///                                         number of values, or the values can't be bind to
    ///                                         the SQL statement
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given
    ///                                     parameters
    /// * [`DatabaseError::AtQuery`] - at failure
    ///
    /// # Side effects
    /// This function calls [`save()`](Table::save), if no `id` is set for the given
    /// `entity`.
    /// # Return
    /// The number of the affected rows
    pub fn update(&self, entity: &mut T) -> Result<usize, DatabaseError> {
        let e: HashMap<String, String> = T::into(entity.clone());
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for (k, v) in e {
            columns.push(k);
            values.push(v)
        }

        if let Some(id) = entity.id() {
            self.service.update(&self.table_name, id, columns, values)
        } else {
            self.save(entity)?;
            Ok(1)
        }

    }

    /// Deletes a row with from the table, using [`PersistentService::delete()`]
    /// # Arguments
    /// * `id` - the row id as [BigInt] reference
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the number of given columns dosen't match the
    ///                                         number of values
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL Query can generated with the given
    ///                                     parameters, or id can't be bind to the SQL statement
    /// * [`DatabaseError::AtQuery`] - at failure
    /// # Return
    /// The number of the affected rows
    pub fn delete(&self, id: &BigInt) -> Result<usize, DatabaseError> {
        self.service.delete(&self.table_name, id)
    }
}

pub mod sqlite_service;
pub mod test_entity;

// TODO: move to integration tests
#[cfg(test)]
mod tests {
    use crate::sqlite_service::SQLiteService;
    use crate::test_entity::TestEntity;
    use ctor::dtor;
    use std::env::current_dir;
    use std::fs::remove_file;
    use super::*;

    const DB_NAME: &str = "test.db";
    const TABLE_NAME: &str = "test_entity";

    #[dtor]
    fn clean_up() {
        let current_path =  current_dir().unwrap()
            .with_file_name("db_service/test.db");

        if current_path.as_path().exists(){
            remove_file(current_path.as_path()).unwrap();
        }

        let service = SQLiteService::new(DB_NAME);
        let query = String::from(include_str!("create_database.sql"));
        service.create_db(query).unwrap();
    }

    #[test]
    fn save_struct() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(4.into(), "Hello world".to_string().into(),
                                None, None, 3.14.into());
        let id = t.save(&mut e).unwrap();
        assert_eq!(e.id, Some(id));
    }

    #[test]
    fn get_struct() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(4.into(), "query me!".to_string().into(),
                                None, None, 1.23.into());
        let id = t.save(&mut e).unwrap();

        let res = t.get_by_id(&id).unwrap();
        assert_eq!(res, e);
    }

    #[test]
    fn get_all_structs() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e1 = TestEntity::new(4.into(), "Hello world".to_string().into(),
                                None, None, 3.14.into());
        let mut e2 = TestEntity::new(42.into(), "query me!".to_string().into(),
                                None, None, 1.23.into());
        t.save(&mut e1).unwrap();
        t.save(&mut e2).unwrap();

        let results = t.get_all().unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results, vec![e1, e2]);
    }

    #[test]
    fn update_struct() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(42.into(), "update me!".to_string().into(),
                                None, None, 1.23.into());
        let id = t.save(&mut e).unwrap();

        e.some_string = "updated".to_string().into();

        let count = t.update(&mut e).unwrap();
        assert_eq!(count, 1);

        let res = t.get_by_id(&id).unwrap();
        assert_eq!(res, e);
    }

    #[test]
    fn delete_struct() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(42.into(), "update me!".to_string().into(),
                                None, None, 1.23.into());
        let id = t.save(&mut e).unwrap();

        let count = t.delete(&id).unwrap();
        assert_eq!(count, 1);

        let results = t.get_all().unwrap();
        assert_eq!(results.len(), 0)
    }

    #[test]
    #[should_panic]
    fn get_deleted_struct() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(42.into(), "update me!".to_string().into(),
                                None, None, 1.23.into());
        let id = t.save(&mut e).unwrap();

        let count = t.delete(&id).unwrap();
        assert_eq!(count, 1);

        t.get_by_id(&id).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_non_existent_struct() {
        clean_up();
        let s = SQLiteService::new(DB_NAME);
        let t: Table<TestEntity, SQLiteService> = Table::new(TABLE_NAME, &s);

        t.get_by_id(&BigInt::from(4)).unwrap();
    }

    #[test]
    fn delete_non_existent_struct() {
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(42.into(), "update me!".to_string().into(),
                                None, None, 1.23.into());
        t.save(&mut e).unwrap();

        let count = t.delete(&BigInt::from(42)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn update_unsaved() {
        let s = SQLiteService::new(DB_NAME);
        let t = Table::new(TABLE_NAME, &s);

        let mut e = TestEntity::new(42.into(), "update me!".to_string().into(),
                                    None, None, 1.23.into());
        let count = t.update(&mut e).unwrap();

        assert_eq!(count, 1);
        assert_ne!(e.id, None);
    }
}
