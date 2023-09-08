use mysql::prelude::*;
use mysql::Opts;
pub use mysql::Value;
use mysql::*;
use std::collections::HashMap;
use std::io::Write;

pub struct MySqlBackend {
    handle: mysql::Conn,
    pub log: slog::Logger,
    _schema: String,
    prep_stmts: HashMap<String, mysql::Statement>,
    db_user: String,
    db_password: String,
    db_addr: String,
    db_name: String,
    backup_file: std::fs::File,
}

impl MySqlBackend {
    pub fn new(
        user: &str,
        password: &str,
        dbname: &str,
        addr: &str,
        backup_file: &str,
        log: Option<slog::Logger>,
        prime: bool,
    ) -> Result<Self> {
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
            Opts::from_url(&format!("mysql://{}:{}@{}/", user, password, addr)).unwrap(),
        )
        .unwrap();
        assert_eq!(db.ping(), true);

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(backup_file)
            .unwrap();

        if prime {
            file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .write(true)
                .open(backup_file)
                .unwrap();

            let mut cmd = String::from("");
            for line in schema.lines() {
                let line = line.trim();
                if line.starts_with("--") || line.is_empty() {
                    continue;
                }
                cmd += line;
                cmd += " ";
                if line.ends_with(";") {
                    db.query_drop(cmd).unwrap();
                    cmd = String::from("");
                }
            }
        }

        Ok(MySqlBackend {
            handle: db,
            log: log,
            _schema: schema.to_owned(),
            prep_stmts: HashMap::new(),
            db_user: String::from(user),
            db_password: String::from(password),
            db_addr: String::from(addr),
            db_name: String::from(dbname),
            backup_file: file,
        })
    }

    fn reconnect(&mut self) {
        self.handle = mysql::Conn::new(
            Opts::from_url(&format!(
                "mysql://{}:{}@{}/",
                self.db_user, self.db_password, self.db_addr
            ))
            .unwrap(),
        )
        .unwrap();
    }

    // TODO(babman): log DELETE and UPDATE statements as well.
    pub fn prep_exec(&mut self, sql: &str, params: Vec<Value>) -> Vec<Vec<Value>> {
        if !self.prep_stmts.contains_key(sql) {
            let stmt = self
                .handle
                .prep(sql)
                .expect(&format!("failed to prepare statement \'{}\'", sql));
            self.prep_stmts.insert(sql.to_owned(), stmt);
        }
        loop {
            match self
                .handle
                .exec_iter(self.prep_stmts[sql].clone(), params.clone())
            {
                Err(e) => {
                    warn!(
                        self.log,
                        "query \'{}\' failed ({}), reconnecting to database", sql, e
                    );
                }
                Ok(res) => {
                    let mut rows = vec![];
                    for row in res {
                        let rowvals = row.unwrap().unwrap();
                        let vals: Vec<Value> = rowvals.iter().map(|v| v.clone().into()).collect();
                        rows.push(vals);
                    }
                    debug!(self.log, "executed query {}, got {} rows", sql, rows.len());
                    return rows;
                }
            }
            self.reconnect();
        }
    }

    fn do_insert(&mut self, table: &str, vals: Vec<Value>, replace: bool) {
        let op = if replace { "REPLACE" } else { "INSERT" };
        // Write SQL statement to back up log file.
        let log = format!(
            "{} INTO {} VALUES ({});",
            op,
            table,
            vals.iter()
                .map(|v| v.as_sql(true))
                .collect::<Vec<_>>()
                .join(",")
        );
        writeln!(self.backup_file, "{}", log).unwrap();

        // Create prepared statement and
        let q = format!(
            "{} INTO {} VALUES ({})",
            op,
            table,
            vals.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
        );
        debug!(self.log, "executed insert query {} for row {:?}", q, vals);

        while let Err(e) = self.handle.exec_drop(q.clone(), vals.clone()) {
            warn!(
                self.log,
                "failed to insert into {}, query {} ({}), reconnecting to database", table, q, e
            );
            self.reconnect();
        }
    }

    pub fn insert(&mut self, table: &str, vals: Vec<Value>) {
        self.do_insert(table, vals, false);
    }

    pub fn replace(&mut self, table: &str, vals: Vec<Value>) {
        self.do_insert(table, vals, true);
    }
}
