use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tower_service::Service;

norpc::include_code!("hello_world");

#[derive(Clone)]
struct HelloWorldApp {
    state: Arc<RwLock<HashMap<u64, String>>>,
}
impl HelloWorldApp {
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
#[norpc::async_trait]
impl HelloWorld for HelloWorldApp {
    type Error = ();
    async fn read(self, id: u64) -> Result<Option<String>, Self::Error> {
        Ok(self.state.read().await.get(&id).cloned())
    }
    async fn write(self, id: u64, v: String) -> Result<(), Self::Error> {
        self.state.write().await.insert(id, v);
        Ok(())
    }
    async fn write_many(self, kv: HashSet<(u64, String)>) -> Result<(), Self::Error> {
        for (k, v) in kv {
            self.state.write().await.insert(k, v);
        }
        Ok(())
    }
    async fn noop(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_hello_world() {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = HelloWorldApp::new();
        let service = HelloWorldService::new(app);
        let service = tower::service_fn(move |x| service.clone().call(x));
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let mut cli = HelloWorldClient::new(chan);

    assert_eq!(cli.read(1).await.unwrap(), None);

    cli.write(1, "one".to_owned()).await.unwrap();
    assert_eq!(cli.read(1).await.unwrap(), Some("one".to_owned()));
    assert_eq!(cli.read(2).await.unwrap(), None);
    assert_eq!(cli.read(3).await.unwrap(), None);

    let mut cli2 = cli.clone();
    let mut h = HashSet::new();
    h.insert((2, "two".to_owned()));
    h.insert((3, "three".to_owned()));
    cli2.write_many(h).await.unwrap();
    assert_eq!(cli2.read(3).await.unwrap(), Some("three".to_owned()));

    assert!(cli2.noop().await.is_ok());
}
