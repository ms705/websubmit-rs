pub use noria::DataType;

use noria::{DurabilityMode, PersistenceParameters};
use noria::consensus::LocalAuthority;
use std::time::Duration;
use noria::builder::Builder;
use noria::handle::{SyncHandle};
use noria::manual::ops::filter::{Filter, FilterCondition, Value};
use noria::manual::ops::project::Project;
use noria::manual::Operator;
use noria::manual::Base;
use std::{thread};


pub struct NoriaBackend {
    pub handle: SyncHandle<LocalAuthority>,
    _log: slog::Logger,
}

impl NoriaBackend {
    pub fn new(zk_addr: &str, log: Option<slog::Logger>) -> Result<Self, std::io::Error> {
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        let mut b = Builder::default();
        b.set_sharding(None);
        b.disable_partial();
        b.log_with(log.clone());
        b.set_persistence(PersistenceParameters::new(
            DurabilityMode::MemoryOnly,
            Duration::from_millis(1),
            Some(String::from("websubmit")),
            1,
        ));

        let mut sh = b.start_simple().unwrap();
        thread::sleep( Duration::from_millis(200));

        let _ = sh.migrate(move |mig| {
            let users = mig.add_base("users", &["email_key", "apikey"], Base::new(vec![]).with_key(vec![1]));
            let lectures = mig.add_base("lectures", &["id", "label"], Base::new(vec![]).with_key(vec![0]));
            let questions = mig.add_base("questions", &["lec", "q", "question"], Base::new(vec![]).with_key(vec![0, 1]));
            println!("Created questions");
            // figure out the aggregation
            // let leclist = mig.add_ingredient("leclist", &["id", "label"], )
            let lecture_filters = Some(&[
                Some(FilterCondition::Comparison(
                    Operator::Equal,
                    Value::Column(0),
                )),
                None,
            ]);
            let lecture = mig.add_ingredient("lecture", &["id", "label"], Filter::new(lectures, lecture_filters.unwrap()));
            let question_filters = Some(&[
                Some(FilterCondition::Comparison(
                    Operator::Equal,
                    Value::Column(0),
                )),
                None,
                None,
            ]);
            let qs_by_lec = mig.add_ingredient("qs_by_lec", &["lec", "q", "question"], Filter::new(questions, question_filters.unwrap()));
            let user_filters = Some(&[
                None,
                Some(FilterCondition::Comparison(
                    Operator::Equal,
                    Value::Constant(DataType::None),
                )),
            ]);
            let users_by_apikey = mig.add_ingredient("users_by_apikey", &["email_key", "apikey"],
                                                     Filter::new(users, user_filters.unwrap()));
            let all_users = mig.add_ingredient("all_users", &["email_key"],
                                               Project::new(users, &[0], None, None));
            mig.maintain_anonymous(all_users, &[0]);
            mig.maintain_anonymous(users_by_apikey, &[0]);
            mig.maintain_anonymous(qs_by_lec, &[0]);
            mig.maintain_anonymous(lecture, &[0]);
        });

        Ok(NoriaBackend {
            handle: sh,
            _log: log,
        })
    }
}
