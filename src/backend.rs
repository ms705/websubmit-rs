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
            //note remember to fix this
            //db.query_drop(format!("DROP DATABASE IF EXISTS {};", dbname))
            //    .unwrap();
            //db.query_drop(format!("CREATE DATABASE {};", dbname))
            //    .unwrap();
            // reconnect
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
        // let mut vals2 = Vec::new();
        // //debug!(self.log, "HEREtmp {}", temp);
        // if table == "questions" {
        //     let vv = vals[0..2].to_vec();
        //     //let key = vv.iter().map(|_| "?").collect::<Vec<&str>>().join("-");
        //     //insert_vals += "test"; //for testing purposes  //&key.to_string();
        //     //insert_vals += ",";
        //     //insert_vals += &temp.to_string();
        //     insert_vals += "1, 1, 1, 1";
        //     vals2 = vec![1, 1, 1, 1];
        // } else if table == "answers" {
        //     let vv = vals[0..3].to_vec();
        //     let key = vv.iter().map(|_| "?").collect::<Vec<&str>>().join("-");
        //     insert_vals += &key;
        //     insert_vals += ",";
        //     insert_vals += &temp;
        // } else {
        insert_vals += &temp;
            //vals2 = vals;
        //}
        //debug!(self.log, "HERE {}", insert_vals);
        let q = format!(
            "{} INTO {} VALUES ({})",
            op,
            table,
            insert_vals
            //vals.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
        );
        debug!(self.log, "executed insert query {} for row {:?}", q, vals);
        self.handle
            .exec_drop(q.clone(), vals)
            .expect(&format!("failed to insert into {}, query {}!", table, q));
    }

    pub fn insert(&mut self, table: &str, vals: Vec<Value>) {
        // if table == "questions" {
        //     let vals2 = vals[0..2].to_vec();
        //     let vals2_unwrapped: Vec<i64> = vals2.iter().map(|x| from_value(x.clone())).collect::<Vec<i64>>();
        //     let key = vals2_unwrapped.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join("-");
        //     let key_as_bytes: Vec<u8> = key.as_bytes().to_vec();
        //     let mut vals_m = vals;
        //     let mut new_vals : Vec<Value> = Vec::new(); 
        //     new_vals.push(mysql::Value::Bytes(key_as_bytes)); 
        //     new_vals.append(&mut vals_m);
        //     self.do_insert(table, new_vals, false);
        // } else if table == "answers" {
        //     let unwrapped1:Vec<u8> = from_value(vals[0].clone());
        //     let unwrapped2:i64 = from_value(vals[1].clone());
        //     let unwrapped3:i64 = from_value(vals[2].clone());
            
        //     let email_string: String = String::from_utf8(unwrapped1).unwrap();
        //     let key = format!("{}-{}-{}", email_string, unwrapped2, unwrapped3);
        //     // let vals3 = vals[0..3].to_vec();
        //     // let key = vals3.iter().map(|_| "?").collect::<Vec<&str>>().join("-");
        //     let key_as_bytes: Vec<u8> = key.as_bytes().to_vec();
        //     let mut vals_m = vals;
        //     let mut new_vals : Vec<Value> = Vec::new(); 
        //     new_vals.push(mysql::Value::Bytes(key_as_bytes)); 
        //     new_vals.append(&mut vals_m);
        //     self.do_insert(table, new_vals, false);
        //} else {
            self.do_insert(table, vals, false);
        //}
    }

    pub fn replace(&mut self, table: &str, vals: Vec<Value>) {
        // if table == "questions" {
        //     let vals2 = vals[0..2].to_vec();
        //     let vals2_unwrapped: Vec<i64> = vals2.iter().map(|x| from_value(x.clone())).collect::<Vec<i64>>();
        //     let key = vals2_unwrapped.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join("-");
        //     let key_as_bytes: Vec<u8> = key.as_bytes().to_vec();
        //     let mut vals_m = vals;
        //     let mut new_vals : Vec<Value> = Vec::new(); 
        //     new_vals.push(mysql::Value::Bytes(key_as_bytes)); 
        //     new_vals.append(&mut vals_m);
        //     self.do_insert(table, new_vals, true);
        // } else if table == "answers" {
        //     let unwrapped1:Vec<u8> = from_value(vals[0].clone());
        //     let unwrapped2:i64 = from_value(vals[1].clone());
        //     let unwrapped3:i64 = from_value(vals[2].clone());
            
        //     let email_string: String = String::from_utf8(unwrapped1).unwrap();
        //     let key = format!("{}-{}-{}", email_string, unwrapped2, unwrapped3);
        //     // let vals3 = vals[0..3].to_vec();
        //     // let key = vals3.iter().map(|_| "?").collect::<Vec<&str>>().join("-");
        //     let key_as_bytes: Vec<u8> = key.as_bytes().to_vec();
        //     let mut vals_m = vals;
        //     let mut new_vals : Vec<Value> = Vec::new(); 
        //     new_vals.push(mysql::Value::Bytes(key_as_bytes)); 
        //     new_vals.append(&mut vals_m);
        //     self.do_insert(table, new_vals, true);
        // } else {
            self.do_insert(table, vals, true);
        //}
    }
}
