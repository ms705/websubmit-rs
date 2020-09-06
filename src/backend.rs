use noria::builder::Builder;
use noria::handle::SyncHandle;
pub use noria::manual::ops::filter::{Filter, FilterCondition, Value};
pub use noria::manual::ops::grouped::aggregate::Aggregation;
pub use noria::manual::ops::identity::Identity;
pub use noria::manual::ops::join::{Join, JoinSource, JoinType};
pub use noria::manual::ops::project::Project;
pub use noria::manual::Base;
pub use noria::manual::Operator;
pub use noria::NodeIndex;
use noria::ZookeeperAuthority;
pub use noria::{DataType, Modification};
use noria::{DurabilityMode, PersistenceParameters};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct NoriaBackend {
    pub handle: SyncHandle<ZookeeperAuthority>,
    _log: slog::Logger,
    pub unions: Option<(NodeIndex, NodeIndex)>,
    pub noria_index: HashMap<String, u32>,
}

impl NoriaBackend {
    pub fn new(
        zk_addr: &str,
        class: &str,
        log: Option<slog::Logger>,
    ) -> Result<Self, std::io::Error> {
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        let mut b = Builder::default();
        b.set_sharding(None);
        b.log_with(log.clone());
        b.set_persistence(PersistenceParameters::new(
            DurabilityMode::DeleteOnExit,
            Duration::from_millis(1),
            Some(String::from(class)),
            1,
        ));
        let authority: Arc<ZookeeperAuthority> =
            Arc::new(ZookeeperAuthority::new(zk_addr).unwrap());
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let fut = b.start(authority);
        let wh = rt.block_on(fut).unwrap();
        thread::sleep(Duration::from_millis(200));
        let mut sh = SyncHandle::from_existing(rt, wh);
        thread::sleep(Duration::from_millis(200));
        b.create_global_table(&mut sh, "shards", &["name", "node_index"], vec![1])
            .expect("failed to create a global table");

        let _ = sh.migrate(move |mig| {
            let exports = mig.add_base(
                "exports",
                &["apikey", "hash"],
                Base::default().with_key(vec![0]),
            );
            let exports_by_apikey = mig.add_ingredient(
                "exports_by_apikey",
                &["apikey", "hash"],
                Project::new(exports, &[0, 1], None, None),
            );
            mig.maintain_anonymous(exports_by_apikey, &[0]);

            let lectures = mig.add_base(
                "lectures",
                &["id", "label"],
                Base::new(vec![]).with_key(vec![0]),
            );
            let questions = mig.add_base(
                "questions",
                &["lec", "q", "question"],
                Base::new(vec![]).with_key(vec![0, 1]),
            );
            let qcount = mig.add_ingredient(
                "qcount",
                &["lec", "qcount"],
                Aggregation::COUNT.over(questions, 1, &[0]),
            );
            let qc = mig.add_ingredient(
                "qc",
                &["lec", "qcount"],
                Project::new(qcount, &[0, 1], None, None),
            );
            let j = Join::new(
                qc,
                lectures,
                JoinType::Left,
                vec![JoinSource::B(0, 0), JoinSource::R(1), JoinSource::L(1)],
            );
            let ll = mig.add_ingredient("ll", &["id", "label", "qc"], j);
            let leclist = mig.add_ingredient(
                "leclist",
                &["id", "label", "qc"],
                Project::new(ll, &[0, 1, 2], None, None),
            );
            mig.maintain_anonymous(leclist, &[0]);

            let lecture = mig.add_ingredient(
                "lecture",
                &["id", "label", "bogokey"],
                Project::new(lectures, &[0, 1], Some(vec![0.into()]), None),
            );
            mig.maintain_anonymous(lecture, &[2]);

            let qs_by_lec = mig.add_ingredient(
                "qs_by_lec",
                &["lec", "q", "question", "bogokey"],
                Project::new(questions, &[0, 1, 2], Some(vec![0.into()]), None),
            );
            mig.maintain_anonymous(qs_by_lec, &[0]);
        });

        Ok(NoriaBackend {
            handle: sh,
            _log: log,
            unions: None,
            noria_index: HashMap::default(),
        })
    }
}
