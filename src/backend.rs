use mysql::prelude::*;
use mysql::Opts;
pub use mysql::Value;
use mysql::*;
use std::collections::HashMap;


pub struct MySqlBackend {
    pub handle: mysql::Conn,
    pub log: slog::Logger,
    _schema: String,
    prep_stmts: HashMap<String, mysql::Statement>,
}

impl MySqlBackend {
    pub fn new(dbname: &str, log: Option<slog::Logger>, prime: bool) -> Result<Self> {
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        let schema = std::fs::read_to_string("src/schema.sql")?;

        debug!(
            log,
            "Connecting to MySql DB and initializing schema {}...", dbname
        );
        let mut db = mysql::Conn::new(
            Opts::from_url(&format!("mysql://pelton:password@127.0.0.1:10001/{}", dbname)).unwrap(),
        )
        .unwrap();
        assert_eq!(db.ping(), true);

        if prime {
            db = mysql::Conn::new(
                Opts::from_url(&format!("mysql://pelton:password@127.0.0.1:10001/{}", dbname)).unwrap(),
            )
            .unwrap();
            for line in schema.lines() {
                if line.starts_with("--") || line.is_empty() {
                    continue;
                }
                db.query_drop(line).unwrap();
            }
        }

        Ok(MySqlBackend {
            handle: db,
            log: log,
            _schema: schema.to_owned(),
            prep_stmts: HashMap::new(),
        })
    }

    pub fn prep_exec(&mut self, sql: &str, params: Vec<Value>) -> Vec<Vec<Value>> {
        if !self.prep_stmts.contains_key(sql) {
            let stmt = self
                .handle
                .prep(sql)
                .expect(&format!("failed to prepare statement \'{}\'", sql));
            self.prep_stmts.insert(sql.to_owned(), stmt);
        }
        let res = self
            .handle
            .exec_iter(self.prep_stmts[sql].clone(), params)
            .expect(&format!("query \'{}\' failed", sql));
        let mut rows = vec![];
        for row in res {
            let rowvals = row.unwrap().unwrap();
            let vals: Vec<Value> = rowvals.iter().map(|v| v.clone().into()).collect();
            rows.push(vals);
        }
        debug!(self.log, "executed query {}, got {} rows", sql, rows.len());
        return rows;
    }

    fn do_insert(&mut self, table: &str, vals: Vec<Value>, replace: bool) {
        let op = if replace { "REPLACE" } else { "INSERT" };
        //changes to be made here
        let mut insert_vals = String::new(); 
        let temp = vals.iter().map(|_| "?").collect::<Vec<&str>>().join(",");
        insert_vals += &temp;
        let q = format!(
            "{} INTO {} VALUES ({})",
            op,
            table,
            insert_vals
        );
        debug!(self.log, "executed insert query {} for row {:?}", q, vals);
        self.handle
            .exec_drop(q.clone(), vals)
            .expect(&format!("failed to insert into {}, query {}!", table, q));
    }

    pub fn insert(&mut self, table: &str, vals: Vec<Value>) {
        self.do_insert(table, vals, false);
    }

    pub fn replace(&mut self, table: &str, vals: Vec<Value>) {
            self.do_insert(table, vals, true);
    }
}
