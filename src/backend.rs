use std::collections::BTreeMap;

pub use noria::DataType;
use noria::{SyncControllerHandle, SyncTable, SyncView, ZookeeperAuthority};

pub struct NoriaBackend {
    pub handle: SyncControllerHandle<ZookeeperAuthority, tokio::runtime::TaskExecutor>,
    _rt: tokio::runtime::Runtime,
    _log: slog::Logger,

    _recipe: String,
    pub tables: BTreeMap<String, SyncTable>,
    pub views: BTreeMap<String, SyncView>,
}

impl NoriaBackend {
    pub fn new(zk_addr: &str, log: Option<slog::Logger>) -> Self {
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        let recipe = "CREATE TABLE answers (user varchar(255), lec int, q int, answer text, PRIMARY KEY (user, lec, q));
                      QUERY answers_by_lec: SELECT * FROM answers WHERE lec = ?;";

        debug!(log, "Finding Noria via Zookeeper...");

        let zk_auth = ZookeeperAuthority::new(&format!("{}", zk_addr))
            .expect("failed to connect to Zookeeper");

        debug!(log, "Connecting to Noria...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let executor = rt.executor();
        let mut ch = SyncControllerHandle::new(zk_auth, executor)
            .expect("failed to connect to Noria controller");

        debug!(log, "Installing recipe in Noria...");
        ch.install_recipe(&recipe.to_owned()).unwrap();

        let inputs = ch
            .inputs()
            .expect("couldn't get inputs from Noria")
            .into_iter()
            .map(|(n, _)| (n.clone(), ch.table(&n).unwrap().into_sync()))
            .collect::<BTreeMap<String, SyncTable>>();
        let outputs = ch
            .outputs()
            .expect("couldn't get outputs from Noria")
            .into_iter()
            .map(|(n, _)| (n.clone(), ch.view(&n).unwrap().into_sync()))
            .collect::<BTreeMap<String, SyncView>>();

        NoriaBackend {
            handle: ch,
            _rt: rt,
            _log: log,

            _recipe: recipe.to_owned(),
            tables: inputs,
            views: outputs,
        }
    }
}
