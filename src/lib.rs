pub mod types;

mod query_source;
mod connection;
mod result;
#[macro_use]
mod macros;

pub use result::*;
pub use query_source::{QuerySource, Queriable};
pub use connection::Connection;

#[cfg(test)]
mod test_usage_without_compiler_plugins {
    use super::{types, QuerySource, Queriable, Connection};
    use types::NativeSqlType;
    use std::env;

    #[derive(PartialEq, Eq, Debug)]
    struct User {
        id: i32,
        name: String,
    }

    table! {
        users {
            id -> Serial,
            name -> VarChar,
        }
    }

    queriable! {
        User {
            id -> i32,
            name -> String,
        }
    }

    fn connection() -> Connection {
        let connection_url = env::var("DATABASE_URL").ok()
            .expect("DATABASE_URL must be set in order to run tests");
        let result = Connection::establish(&connection_url).unwrap();
        result.execute("BEGIN").unwrap();
        result
    }

    fn setup_users_table(connection: &Connection) {
        connection.execute("CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR NOT NULL)")
            .unwrap();
    }

    #[test]
    fn it_can_perform_a_basic_query() {
        let connection = connection();
        setup_users_table(&connection);
        connection.execute("INSERT INTO users (name) VALUES ('Sean'), ('Tess')")
            .unwrap();

        let expected_users = vec![
            User { id: 1, name: "Sean".to_string() },
            User { id: 2, name: "Tess".to_string() },
         ];
        let actual_users = connection.query_all(&users::table).unwrap();
        assert_eq!(expected_users, actual_users);
    }

    #[test]
    fn with_select_sql() {
        let connection = connection();
        setup_users_table(&connection);
        connection.execute("INSERT INTO users (name) VALUES ('Sean'), ('Tess')")
            .unwrap();

        let select_count = unsafe { users::table.select_sql::<types::BigInt>("COUNT(*)") };
        let get_count = || connection.query_one::<_, i64>(&select_count).unwrap();
        // fails to compile (we should test this)
        // let actual_count = connection.query_one::<_, String>(&select_count).unwrap();

        assert_eq!(Some(2), get_count());

        connection.execute("INSERT INTO users (name) VALUES ('Jim')")
            .unwrap();

        assert_eq!(Some(3), get_count());
    }
}
