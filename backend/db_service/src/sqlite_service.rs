//! This Module contains the [SQLiteService], a implementation of the [PersistentService] trait for
//! SQLite databases.

use std::collections::HashMap;
use num_bigint::BigInt;
use sqlite::{Connection, Value, State, Type, Statement};
use crate::{DatabaseError, PersistentService};

/// Implementation of [PersistentService]
pub struct SQLiteService {
    db: Connection
}

impl PersistentService for SQLiteService {
    fn new(db: &str) -> Self {
        SQLiteService {
            db: Connection::open(db).unwrap() //TODO: Error handling
        }
    }

    fn create_db(&self, query: String) -> Result<(), DatabaseError> {
        self.db.execute(query)
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))
    }

    fn close_connection(&self) -> Result<(), DatabaseError> {
        Ok(()) // the sqlite crate doesn't have to close connections
    }

    fn get_all(&self, table: &str)
        -> Result<Vec<HashMap<String, Option<String>>>, DatabaseError> {
        // inject unbindable table name into the query string
        let mut stmt = self.db.prepare(format!("SELECT * FROM {};", table))
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        let mut results = Vec::new();

        //iterate over the results and parse them into a hashmap
        while let Ok(State::Row) = stmt.next() {
            results.push(SQLiteService::read_row(&stmt));
        }
        Ok(results)
    }

    fn get_by_id(&self, table: &str, id: &BigInt)
        -> Result<HashMap<String, Option<String>>, DatabaseError> {
        // query the column name for the primary key of the given table
        let primary_key = self.primary_key(table)?;

        // inject unbindable table and column name into the query string
        let mut stmt = self.db
            .prepare(format!("SELECT * FROM {} WHERE {} == ?", table, primary_key))
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        // bind the id
        stmt.bind::<(_,Value)>((1, id.to_string().into()))
            .map_err(|err| DatabaseError::InvalidParameter(err.message.unwrap()))?;

        // check if at least one row is returned, parse and return only the first result
        if let Ok(State::Row) = stmt.next() {
            Ok(Self::read_row(&stmt))
        } else {
            Err(DatabaseError::NotFound(String::from("Couldn't find entry for given id.")))
        }
    }

    //TODO: parse boolean
    //TODO: how to save blob
    fn save(&self, table: &str, columns: Vec<String>, values: Vec<String>)
        -> Result<BigInt, DatabaseError> {

        // check if a valid number of columns and values was provided
        if (columns.len() != values.len() && columns.is_empty()) || values.is_empty() {
            return Err(DatabaseError::InvalidParameter(
                String::from("Number of columns doesn't match number of values.")
            ));
        }

        // for any valid number of values at least one value placeholder is needed
        let mut placeholder = String::from("?");

        // extend the placeholders for each value
        if values.len() > 1 {
            placeholder.push_str(&", ?".repeat(columns.len()-1));
        }

        let query;

        // inject unbindable table name and column names, as well as the matching number of
        // placeholders into the query string, depending on whether column names are provided or not
        if columns.is_empty() {
            query = format!("INSERT INTO {} VALUES ({});", table, placeholder);
        } else {
            // merge column names into a correctly formatted comma seperated string
            let mut col_str = String::from(&columns[0]);
            for i in 1..columns.len() {
                col_str.push_str(", ");
                col_str.push_str(&columns[i]);
            }

            query = format!("INSERT INTO {} ({}) VALUES ({});", table, col_str, placeholder);
        }

        // Bind with string format, since parameters of prepared statements have to match the number
        // of columns. You can't bind a string to :values. This will result in the InvalidSQL Error:
        // "Table x has y columns but 1 value was provided"
        let mut stmt = self.db.prepare(query)
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        // bind the values to the statement
        Self::bind_values(values, &mut stmt)?;

        // execute the statement until it's done
        while let Ok(State::Row) = stmt.next() {}

        // return the id of the inserted row
        self.get_last_row_id()
    }

    fn update(&self, table: &str, id: &BigInt, columns: Vec<String>, mut values: Vec<String>)
              -> Result<usize, DatabaseError> {

        // check if a valid number of columns and values was provided
        if (columns.len() != values.len()) || columns.is_empty() || values.is_empty() {
            return Err(
                DatabaseError::InvalidParameter(
                    String::from("Wrong number of column names or values was passed.")
                )
            );
        }

        // query the column name for the primary key of the given table
        let primary_key = self.primary_key(table)?;

        // for any valid number of values at least one value placeholder with injected column name
        // is needed
        let mut placeholder = format!("{} = ?", columns[0]);

        // extend the placeholders for each value
        for i in 1..columns.len() {
            placeholder.push_str(", ");
            placeholder.push_str(format!("{} = ?", columns[i]).as_str());
        }

        // inject unbindable table name and the matching number of placeholders into the query string
        let query = format!("UPDATE {} SET {} WHERE {} = ?", table, placeholder, primary_key);

        let mut stmt = self.db.prepare(query)
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        // add the id to the value vector in order to bind it together with the other values
        values.push(id.to_string());

        // bind the values to the statement
        Self::bind_values(values, &mut stmt)?;

        // execute the statement until it's done
        while let Ok(State::Row) = stmt.next() {}

        // return the number of affected rows
        Ok(self.db.change_count())

    }

    fn delete(&self, table: &str, id: &BigInt) -> Result<usize, DatabaseError> {
        // query the column name for the primary key of the given table
        let primary_key = self.primary_key(table)?;

        // inject unbindable table name and column name into the query string
        let mut stmt = self.db
            .prepare(format!("DELETE FROM {} WHERE {} == ?;", table, primary_key))
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        // bind the id
        stmt.bind::<(_,Value)>((1, id.to_string().into()))
            .map_err(|err| DatabaseError::InvalidParameter(err.message.unwrap()))?;

        // execute the statement until it's done
        while let Ok(State::Row) = stmt.next() {}

        // return the number of affected rows
        Ok(self.db.change_count())
    }
}

impl SQLiteService {
    //TODO: this function may also be public/ part of the persistent service interface but maybe
    //      the differences between Databases are to large

    /// This function returns the column name of the primary key column of a given table, using
    /// the `pragma_table_info()` function of SQLite
    /// # Arguments
    /// * `table` - the table name
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL query can be generated for the given table
    ///                                     name
    /// * [`DatabaseError::AtQuery`] - on failure
    /// # Return
    /// The column name as [String]
    fn primary_key(&self, table: &str)-> Result<String, DatabaseError> {
        // inject unbindable table name into the query string
        let mut stmt = self.db
            .prepare(format!("SELECT name from pragma_table_info('{}') WHERE pk == 1;", table))
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        // check if at least one row is returned, parse and return only the first result
        if let Ok(State::Row) = stmt.next() {
            Ok(stmt.read("name").unwrap())
        } else {
            Err(DatabaseError::AtQuery(String::from("Can't query primary Key.")))
        }
    }

    /// Converts a given array of [String]s, that represent values for a SQL query, to index -
    /// [Value] tuples that can be used to bind values to a prepared SQL [Statement].
    /// # Arguments
    /// * `v` - the array of values as [String]s
    /// # Panics
    /// If a [String] value can't be parsed to a [Value]
    /// # Returns
    /// A [Vector](Vec) of index - [Value] tuples
    /// # Example
    /// ```ignore
    /// // prepare a SQL statement that contains placeholders and bind an array of values to it
    /// // by converting the array in an iterable index-value tuple and bind it to the statement
    /// // using bind_iter()
    /// let mut stmt = connection.prepare(/* ... */)?;
    /// let value_pairs = SQLiteService::index_value_pair(&values);
    /// stmt.bind_iter::<_, (_, Value)>(value_pairs)
    ///     .map_err(|err| DatabaseError::InvalidParameter(err.message.unwrap()))?;
    /// ```
    fn index_value_pair(v: &[String]) -> Vec<(usize, Value)> {
        let mut pairs: Vec<(usize, Value)> = Vec::new();
        let iter = v.iter().enumerate();
        for i in iter {
            // indizes for values starting with 1, cast Strings to Values
            pairs.push((i.0 + 1, i.1.clone().into()))
        }
        pairs
    }

    /// Wrapper function to bind a string of values to a prepared SQL statement.
    /// # Arguments
    /// * `values` - a [Vector](vec) of values as [String]s
    /// * `stmt` - the prepared SQL [Statement]
    /// # Errors
    /// * [`DatabaseError::InvalidParameter`] - if the provided values can't be bind to the
    ///                                         SQL [Statement]
    /// # Panics
    /// If a [String] value in `values` can't be parsed to a [Value]
    /// # Example
    /// ``` ignore
    /// // bind an array of values to a prepared statement using bind_valeus()
    /// let mut stmt = connection.prepare(query)?;
    /// Self::bind_values(values, &mut stmt)?;
    /// ```
    fn bind_values(values: Vec<String>, stmt: &mut Statement) -> Result<(), DatabaseError> {
        let value_pairs = SQLiteService::index_value_pair(&values);
        stmt.bind_iter::<_, (_, Value)>(value_pairs)
            .map_err(|err| DatabaseError::InvalidParameter(err.message.unwrap()))?;
        Ok(())
    }

    /// Returns the row id for the last inserted row into the database, using the
    /// `last_insert_rowid()` function of SQLite.
    /// # Errors
    /// * [`DatabaseError::InvalidSQL`] - if no valid SQL statement to query the row id can be
    ///                                     generated
    /// * [`DatabaseError::NotFound`] - if the last row id can't be queried
    /// * [`DatabaseError::AtQuery`] - on failure
    /// # Return
    /// The row id as [BigInt]
    fn get_last_row_id(&self) -> Result<BigInt, DatabaseError> {
        let mut stmt = self.db
            .prepare("SELECT last_insert_rowid() as ID;")
            .map_err(|err| DatabaseError::InvalidSQL(err.message.unwrap()))?;

        // check if at least one row is returned, parse and return only the first result
        if let Ok(State::Row) = stmt.next() {
            let id = stmt.read::<i64, _>(0)
                .map_err(|err| DatabaseError::AtQuery(err.message.unwrap()))?;
            Ok(BigInt::from(id))
        } else {
            Err(DatabaseError::NotFound(String::from("Can't query ID for last insert.")))
        }
    }

    /// Auxiliary function to parse a query result into a [HashMap].
    /// # Arguments
    /// * `stmt` - a prepared SQL [Statement] with bound values.
    /// # Panics
    /// If a column value can't be read or parsed to a [String].
    /// # Return
    /// A [HashMap] consisting of the column names as [String] keys, and the read values as
    /// [Option]<[String]> values, that might be [None] for NULL values in the Database.
    /// # Examples
    /// ```ignore
    /// // parse the first result from a query
    /// if let Ok(State::Row) = stmt.next() {
    ///    Ok(Self::read_row(&stmt))
    /// }
    /// ```
    /// ```ignore
    /// // iterate over all results of a query and parse all of them
    /// let mut results = Vec::new();
    ///
    /// while let Ok(State::Row) = stmt.next() {
    ///    results.push(SQLiteService::read_row(&stmt));
    /// }
    /// ```
    fn read_row(stmt: &Statement) -> HashMap<String, Option<String>> {
        let mut result = HashMap::new();
        let columns = stmt.column_count();
        for i in 0..columns {
            result.insert(
                stmt.column_name(i).unwrap().to_string(),
                match stmt.column_type(i).unwrap() {
                    Type::Binary => Some(stmt.read::<String, _>(i).unwrap()),
                    Type::Float => Some(stmt.read::<f64, _>(i).unwrap().to_string()),
                    Type::Integer => Some(stmt.read::<i64, _>(i).unwrap().to_string()),
                    Type::String => Some(stmt.read::<String, _>(i).unwrap()),
                    Type::Null => None
                }
            );
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::fs::remove_file;
    use ctor::dtor;
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
    fn create_db() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);

        let query = String::from(include_str!("create_database.sql"));
        service.create_db(query).unwrap();
        let current_path =  current_dir().unwrap()
            .with_file_name("db_service/test.db");
        assert!(current_path.as_path().exists());
    }

    #[test]
    fn insert() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let _id = service
            .save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec!["0".to_string(), "first".to_string()])
            .unwrap();
    }

    #[test]
    fn insert_and_get() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let table = TABLE_NAME;
        let id = service
            .save(table, vec!["some_int".to_string(), "some_string".to_string(),
                              "some_float".to_string()],
                  vec![42.to_string(), "hello world".to_string(), 1.23.to_string()])
            .unwrap();

        let res = service.get_by_id(table, &id).unwrap();
        let mut expected = HashMap::new();
        expected.insert("ID".to_string(), Some(id.to_string()));
        expected.insert("some_int".to_string(), Some(42.to_string()));
        expected.insert("some_string".to_string(), Some("hello world".to_string()));
        expected.insert("some_float".to_string(), Some(1.23.to_string()));
        expected.insert("some_bool".to_string(), None);
        expected.insert("some_blob".to_string(), None);

        assert_eq!(res, expected);
    }

    #[test]
    fn get_all() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);

        let id1= service
            .save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec!["0".to_string(), "first".to_string()])
            .unwrap();

        let id2 = service
            .save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec![42.to_string(), "hello world".to_string()])
            .unwrap();

        let entries = service
            .get_all(TABLE_NAME).unwrap();

        let mut first = HashMap::new();
        first.insert("ID".to_string(), Some(id1.to_string()));
        first.insert("some_int".to_string(), Some(0.to_string()));
        first.insert("some_string".to_string(), Some("first".to_string()));
        first.insert("some_float".to_string(), None);
        first.insert("some_bool".to_string(), None);
        first.insert("some_blob".to_string(), None);

        let mut second = HashMap::new();
        second.insert("ID".to_string(), Some(id2.to_string()));
        second.insert("some_int".to_string(), Some(42.to_string()));
        second.insert("some_string".to_string(), Some("hello world".to_string()));
        second.insert("some_float".to_string(), None);
        second.insert("some_bool".to_string(), None);
        second.insert("some_blob".to_string(), None);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], first);
        assert_eq!(entries[1], second);
    }

    #[test]
    fn delete() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);

        let id1= service
            .save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec!["0".to_string(), "first".to_string()])
            .unwrap();

        let id2 = service
            .save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec![42.to_string(), "hello world".to_string()])
            .unwrap();

        let id3 = service.save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec!["123".to_string(), "last".to_string()]).unwrap();

        let count = service
            .delete(TABLE_NAME, &id2)
            .unwrap();

        assert_eq!(count, 1);

        let entries = service
            .get_all(TABLE_NAME).unwrap();

        let mut first = HashMap::new();
        first.insert("ID".to_string(), Some(id1.to_string()));
        first.insert("some_int".to_string(), Some(0.to_string()));
        first.insert("some_string".to_string(), Some("first".to_string()));
        first.insert("some_float".to_string(), None);
        first.insert("some_bool".to_string(), None);
        first.insert("some_blob".to_string(), None);

        let mut last = HashMap::new();
        last.insert("ID".to_string(), Some(id3.to_string()));
        last.insert("some_int".to_string(), Some(123.to_string()));
        last.insert("some_string".to_string(), Some("last".to_string()));
        last.insert("some_float".to_string(), None);
        last.insert("some_bool".to_string(), None);
        last.insert("some_blob".to_string(), None);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], first);
        assert_eq!(entries[1], last);
    }

    #[test]
    fn update() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let table = TABLE_NAME;
        let id = service
            .save(table, vec!["some_int".to_string(), "some_string".to_string()],
                  vec![42.to_string(), "hello world".to_string()])
            .unwrap();

        let count = service.update(table, &id,
                                   vec!["some_string".to_string(), "some_float".to_string()],
                                   vec!["new_value".to_string(), 1.23.to_string()])
            .unwrap();

        assert_eq!(count, 1);

        let res = service.get_by_id(table, &id).unwrap();
        let mut expected = HashMap::new();
        expected.insert("ID".to_string(), Some(id.to_string()));
        expected.insert("some_int".to_string(), Some(42.to_string()));
        expected.insert("some_string".to_string(), Some("new_value".to_string()));
        expected.insert("some_float".to_string(), Some(1.23.to_string()));
        expected.insert("some_bool".to_string(), None);
        expected.insert("some_blob".to_string(), None);

        assert_eq!(res, expected);
    }

    #[test]
    #[should_panic]
    fn get_deleted() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let table = TABLE_NAME;

        let id = service
            .save(table, vec!["some_int".to_string(), "some_string".to_string(),
                              "some_float".to_string()],
                  vec![42.to_string(), "hello world".to_string(), 1.23.to_string()])
            .unwrap();

        let count = service.delete(table, &id).unwrap();
        assert_eq!(count, 1);

        service.get_by_id(table, &id).unwrap();
    }

    #[test]
    fn delete_non_exist() {
        clean_up();

        let service = SQLiteService::new(DB_NAME);
        let table = TABLE_NAME;

        service
            .save(table, vec!["some_int".to_string(), "some_string".to_string(),
                              "some_float".to_string()],
                  vec![42.to_string(), "hello world".to_string(), 1.23.to_string()])
            .unwrap();

        let count = service.delete(table, &BigInt::from(4)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    #[should_panic]
    fn get_non_exist() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let table = TABLE_NAME;

        let res = service.get_by_id(table,&BigInt::from(4)).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    #[should_panic]
    fn invalid_insert_col() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let _id = service
            .save(TABLE_NAME,
                  vec!["some_int".to_string()],
                  vec!["0".to_string(), "first".to_string()])
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_insert_val() {
        clean_up();
        let service = SQLiteService::new(DB_NAME);
        let _id = service
            .save(TABLE_NAME,
                  vec!["some_int".to_string(), "some_string".to_string()],
                  vec!["0".to_string()])
            .unwrap();
    }

    #[test]
    fn index_value_pair() {
        let v = vec!["hello".to_string(), "world".to_string()];
        let pairs = SQLiteService::index_value_pair(&v);

        assert_eq!(pairs,
                   vec![(1, Value::from(v[0].clone())),
                        (2 , Value::from(v[1].clone()))]
        );
    }

    #[test]
    fn empty_value_pair() {
        let v = Vec::new();
        let pairs = SQLiteService::index_value_pair(&v);
        assert_eq!(pairs, vec![]);
    }
}
