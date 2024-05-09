#[cfg(test)]
mod tests {
    use common::models::{
        acid_sync::AcidSync,
        db::{Column, Data, DataType, Row},
    };
    use persistence::table::table::Table;

    use crate::{errors::QueryError, process_query};

    use std::path::Path;

    fn sync_guard() -> AcidSync {
        AcidSync::default()
    }

    fn drop_table(table_name: &str) {
        assert!(
            process_query(format!("DROP TABLE {}", table_name).as_str(), sync_guard()).is_ok(),
            "Table not dropped"
        );
    }

    #[test]
    fn test_create_and_drop_table() {
        let table_name = "test_create_and_drop_table";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!(
                    "CREATE TABLE {} x int, y varchar, bool_column boolean",
                    table_name
                )
                .as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(Path::new(table_name).exists(), "File for table not created");
        assert!(
            process_query(
                format!("DROP TABLE {}", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not dropped"
        );
        assert!(
            !Path::new(table_name).exists(),
            "File for table still exists"
        );
    }

    #[test]
    fn test_table_already_exists() {
        let table_name = "test_table_already_exists";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(Path::new(table_name).exists(), "File for table not created");
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_err(),
            "Table should not be created because it already exists"
        );
        drop_table(table_name);
    }

    #[test]
    fn test_insert_row() {
        let table_name = "test_insert_row";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y int", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 24, 107", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row not inserted"
        );

        let result = process_query(
            format!("SELECT * FROM {}", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");
        let data = result.unwrap().data.unwrap().rows;
        assert!(data.len() == 1, "Table should have only one row");
        assert!(data[0].values.len() == 2, "Row should have two values");
        assert_eq!(data[0].values[0], Data::INT(24));
        assert_eq!(data[0].values[1], Data::INT(107));

        drop_table(table_name);
    }

    #[test]
    fn test_insert_row_only_subset_of_columns() {
        let table_name = "test_insert_row_only_subset_of_columns";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!(
                    "CREATE TABLE {} x int, y int, b boolean, f float",
                    table_name
                )
                .as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} (x, b) VALUES (24, true)", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row not inserted"
        );

        let result = process_query(
            format!("SELECT * FROM {}", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");
        let data = result.unwrap().data.unwrap().rows;
        assert!(data.len() == 1, "Table should have only one row");
        assert!(data[0].values.len() == 4, "Row should have four values");
        assert_eq!(data[0].values[0], Data::INT(24));
        assert_eq!(data[0].values[1], Data::NULL);
        assert_eq!(data[0].values[2], Data::BOOLEAN(true));
        assert_eq!(data[0].values[3], Data::NULL);

        drop_table(table_name);
    }

    #[test]
    fn test_insert_row_with_wrong_amount_of_values() {
        let table_name = "test_insert_row_with_wrong_amount_of_values";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y int", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 24, 107, 105", table_name).as_str(),
                sync_guard.clone()
            )
            .is_err(),
            "Inserting row should cause error"
        );
        drop_table(table_name);
    }

    #[test]
    fn test_insert_row_with_wrong_datatype_value() {
        let table_name = "test_insert_row_with_wrong_datatype_value";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 24, 107", table_name).as_str(),
                sync_guard.clone()
            )
            .is_err(),
            "Inserting row should cause error"
        );
        drop_table(table_name);
    }

    #[test]
    fn test_select_basic() {
        let table_name = "test_select_basic";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 24, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 25, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );

        let result = process_query(
            format!("SELECT * FROM {}", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");

        let expected = vec![
            Row {
                values: vec![Data::INT(24), Data::STRING("text".to_string())],
            },
            Row {
                values: vec![Data::INT(25), Data::STRING("text2".to_string())],
            },
        ];
        let data = result.unwrap().data.unwrap().rows;
        assert_eq!(expected, data);

        drop_table(table_name);
    }

    #[test]
    fn test_select_with_where() {
        let table_name = "test_select_with_where";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 24, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 25, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 25, 'text3'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 28, 'text4'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text5'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );

        let result = process_query(
            format!("SELECT * FROM {} WHERE x = 25", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");

        let expected = vec![
            Row {
                values: vec![Data::INT(25), Data::STRING("text2".to_string())],
            },
            Row {
                values: vec![Data::INT(25), Data::STRING("text3".to_string())],
            },
        ];
        let data = result.unwrap().data.unwrap().rows;
        assert_eq!(expected, data);

        drop_table(table_name);
    }

    #[test]
    fn test_select_with_unknown_column() {
        let table_name = "test_select_with_unknown_column";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        let result = process_query(
            format!("SELECT * FROM {} WHERE unknown = 25", table_name).as_str(),
            sync_guard.clone(),
        );

        assert!(result.is_err(), "Select failed");
        if let Err(e) = result {
            match e {
                QueryError::ColumnNotExists(_, _) => (),
                _ => assert!(false, "The error should be ColumnNotExists"),
            }
        }

        drop_table(table_name);
    }

    #[test]
    fn test_select_projection_with_star() {
        let table_name = "test_select_projection_with_star";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 24, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row should be successfully inserted"
        );

        let result = process_query(
            format!("SELECT *, x, x FROM {}", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");

        let expected = vec![Row {
            values: vec![
                Data::INT(24),
                Data::STRING("text".to_string()),
                Data::INT(24),
                Data::INT(24),
            ],
        }];
        let data = result.unwrap().data.unwrap().rows;
        assert_eq!(expected, data);

        drop_table(table_name);
    }

    #[test]
    fn test_select_with_index() {
        let table_name = "test_select_with_index";
        let sync_guard = sync_guard();

        let column1 = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: true,
        };
        let column2 = Column {
            name: String::from("Name"),
            data_type: DataType::STRING { size: 256 },
            is_indexed: false,
        };
        let table = Table {
            name: String::from(table_name),
            columns: vec![column1, column2],
        };
        assert!(table.create().is_ok());

        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 1, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row1 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 3, 'text3'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row3 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 4, 'text4'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row4 should be successfully inserted"
        );

        let result = process_query(
            format!("SELECT * FROM {} WHERE Id = 2", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");

        let expected = vec![Row {
            values: vec![Data::INT(2), Data::STRING("text2".to_string())],
        }];
        let data = result.unwrap().data.unwrap().rows;
        assert_eq!(expected, data);

        drop_table(table_name);
    }

    #[test]
    fn test_delete_basic() {
        let table_name = "test_delete_basic";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 1, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row1 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 3, 'text3'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row3 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 4, 'text4'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row4 should be successfully inserted"
        );

        let result = process_query(
            format!("DELETE FROM {} WHERE x >= 3", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");
        assert_eq!(
            result.unwrap().message.unwrap(),
            format!("Deleted 2 rows from table {}.", table_name)
        );

        drop_table(table_name);
    }

    #[test]
    fn test_delete_all() {
        let table_name = "test_delete_all";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 1, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row1 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 3, 'text3'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row3 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 4, 'text4'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row4 should be successfully inserted"
        );

        let result = process_query(
            format!("DELETE FROM {}", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");
        assert_eq!(
            result.unwrap().message.unwrap(),
            format!("Deleted 4 rows from table {}.", table_name)
        );

        drop_table(table_name);
    }

    #[test]
    fn test_delete_based_on_index() {
        let table_name = "test_delete_based_on_index";
        let sync_guard = sync_guard();

        let column1 = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: true,
        };
        let column2 = Column {
            name: String::from("Name"),
            data_type: DataType::STRING { size: 256 },
            is_indexed: false,
        };
        let table = Table {
            name: String::from(table_name),
            columns: vec![column1, column2],
        };
        assert!(table.create().is_ok());

        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 1, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row1 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text21'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 3, 'text3'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row3 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 4, 'text4'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row4 should be successfully inserted"
        );

        let result = process_query(
            format!("DELETE FROM {} WHERE Id = 2", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");
        assert_eq!(
            result.unwrap().message.unwrap(),
            format!("Deleted 2 rows from table {}.", table_name)
        );

        drop_table(table_name);
    }

    #[test]
    fn test_select_float_with_index() {
        let table_name = "test_select_float_with_index";
        let sync_guard = sync_guard.clone();
        let column1 = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: false,
        };
        let column2 = Column {
            name: String::from("float_column"),
            data_type: DataType::FLOAT,
            is_indexed: true,
        };
        let table = Table {
            name: String::from(table_name),
            columns: vec![column1, column2],
        };
        assert!(table.create().is_ok());

        let sync = sync_guard();

        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 1, 1.11", table_name).as_str(),
                sync.clone()
            )
            .is_ok(),
            "Row1 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 2.22", table_name).as_str(),
                sync.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 3, 3.33", table_name).as_str(),
                sync.clone()
            )
            .is_ok(),
            "Row3 should be successfully inserted"
        );
        let result = process_query(
            format!("SELECT * FROM {} WHERE float_column = 2.22", table_name).as_str(),
            sync.clone(),
        );
        assert!(result.is_ok(), "Select failed");

        let expected = vec![Row {
            values: vec![Data::INT(2), Data::FLOAT(2.22f64)],
        }];
        let data = result.unwrap().data.unwrap().rows;
        assert_eq!(expected, data);

        drop_table(table_name);
    }

    #[test]
    fn test_create_drop_index() {
        let table_name = "test_create_drop_index";
        let sync_guard = sync_guard();
        assert!(
            process_query(
                format!("CREATE TABLE {} x int, y varchar", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Table not created"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 1, 'text'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row1 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text2'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );
        assert!(
            process_query(
                format!("INSERT INTO {} VALUES 2, 'text21'", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Row2 should be successfully inserted"
        );

        assert!(
            process_query(
                format!("CREATE INDEX x ON {}", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Index on column x should be created"
        );
        let table_after_index_creation = Table::load(table_name.to_string()).unwrap();
        assert_eq!(
            table_after_index_creation
                .columns
                .get(0)
                .unwrap()
                .is_indexed,
            true
        );

        let result = process_query(
            format!("SELECT * FROM {} WHERE x = 1", table_name).as_str(),
            sync_guard.clone(),
        );
        assert!(result.is_ok(), "Select failed");

        let expected = vec![Row {
            values: vec![Data::INT(1), Data::STRING("text".to_string())],
        }];
        let data = result.unwrap().data.unwrap().rows;
        assert_eq!(expected, data);

        assert!(
            process_query(
                format!("DROP INDEX x ON {}", table_name).as_str(),
                sync_guard.clone()
            )
            .is_ok(),
            "Index on column x should be dropped"
        );

        let table_after_index_drop = Table::load(table_name.to_string()).unwrap();
        assert_eq!(
            table_after_index_drop.columns.get(0).unwrap().is_indexed,
            false
        );

        drop_table(table_name);
    }
}
