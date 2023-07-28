//! This module contains the [TestEntity] struct that can be used to test [Table](super::Table)s or
//! [PersistentService](super::PersistentService) implementations (provided that a matching
//! `test_entity` table exists in the database).

use std::collections::HashMap;
use num_bigint::BigInt;
use crate::Entity;

/// This struct was implemented for testing reasons only.
/// It also gives an example on how to implement the [Entity] trait.
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[allow(dead_code)]
pub struct TestEntity {
    pub id: Option<BigInt>,
    some_int: Option<isize>,
    pub some_string: Option<String>,
    some_bool: Option<bool>,
    some_blob: Option<Vec<u8>>,
    some_float: Option<f64>
}

impl TestEntity {
    pub(crate) fn new(some_int: Option<isize>, some_string: Option<String>, some_bool: Option<bool>,
           some_blob: Option<Vec<u8>>, some_float: Option<f64>) -> Self {
        TestEntity { id: None, some_int, some_string, some_bool, some_blob, some_float }
    }
}

impl From<HashMap<String, Option<String>>> for TestEntity {
    fn from(value: HashMap<String, Option<String>>) -> Self {
        let mut id = None;
        let mut some_int = None;
        let mut some_string = None;
        let mut some_bool = None;
        let mut some_blob = None;
        let mut some_float = None;

        for v in value {
            if let Some(val) = v.1 {
                match v.0.as_str() {
                    "ID" => id = Some(BigInt::from(val.parse::<isize>().unwrap())),
                    "some_int" => some_int = Some(val.parse::<isize>().unwrap()),
                    "some_string" => some_string = Some(val),
                    "some_bool" => match val.as_str() {
                        "1" => some_bool = Some(true),
                        "0" => some_bool= Some(false),
                        _ => panic!()
                    }
                    "some_blob" => some_blob = Some(val.as_bytes().to_vec()),
                    "some_float" => some_float = Some(val.parse::<f64>().unwrap_or(-1.0)),
                    _ => panic!()
                }
            }
        }

        TestEntity {id, some_int, some_string, some_bool, some_blob, some_float}
    }
}

impl Into<HashMap<String, String>> for TestEntity {
    fn into(self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(val) = self.some_int {
            map.insert("some_int".to_string(), val.to_string());
        }
        if let Some(val) = self.some_string {
            map.insert("some_string".to_string(), val);
        }
        if let Some(val) = self.some_bool {
            map.insert("some_bool".to_string(), val.to_string());
        }
        if let Some(val) = self.some_blob {
            map.insert("some_blob".to_string(), String::from_utf8(val).unwrap());
        }
        if let Some(val) = self.some_float {
            map.insert("some_float".to_string(), val.to_string());
        }
        map
    }
}

impl Entity for TestEntity {
    fn id(&self) -> &Option<BigInt> {
        &self.id
    }

    fn id_mut(&mut self) -> &mut Option<BigInt> {
        &mut self.id
    }
}