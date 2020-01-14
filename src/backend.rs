use std::collections::BTreeMap;

pub use noria::DataType;
use noria::{ControllerHandle, Table, View, ZookeeperAuthority};

pub struct NoriaBackend {
    pub handle: ControllerHandle<ZookeeperAuthority>,
    _rt: tokio::runtime::Runtime,
    _log: slog::Logger,

    _recipe: String,
    pub tables: BTreeMap<String, Table>,
    pub views: BTreeMap<String, View>,
}

impl NoriaBackend {
    pub async fn new(zk_addr: &str, log: Option<slog::Logger>) -> Result<Self, std::io::Error> {
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        let recipe = std::fs::read_to_string("src/schema.sql")?;

        debug!(log, "Finding Noria via Zookeeper...");

        let zk_auth = ZookeeperAuthority::new(&format!("{}", zk_addr))
            .expect("failed to connect to Zookeeper");

        debug!(log, "Connecting to Noria...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let executor = rt.executor();
        let mut ch = ControllerHandle::new(zk_auth).await.unwrap();

        debug!(log, "Installing recipe in Noria...");
        ch.install_recipe(&recipe.to_owned()).await.unwrap();
        let init_inputs = ch
            .inputs().await.unwrap()
            .into_iter();
        let mut inputs = BTreeMap::new();
        for (n, _) in init_inputs {
          inputs.insert(n.clone(), ch.table(&n).await.unwrap());
        };

        let init_outputs = ch
            .outputs().await.unwrap()
            .into_iter();

        let mut outputs = BTreeMap::new();
        for (n, _) in init_outputs {
          outputs.insert(n.clone(), ch.view(&n).await.unwrap());
        };

        Ok(NoriaBackend {
            handle: ch,
            _rt: rt,
            _log: log,

            _recipe: recipe.to_owned(),
            tables: inputs,
            views: outputs,
        })
    }
}
