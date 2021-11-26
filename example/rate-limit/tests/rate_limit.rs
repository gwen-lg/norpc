use tokio::sync::mpsc;
use tower::Service;
use tower::ServiceBuilder;

const N: usize = 10000;

#[norpc::service]
trait RateLimit {
    fn noop();
}

#[derive(Clone)]
struct RateLimitApp;
#[norpc::async_trait]
impl RateLimit for RateLimitApp {
    async fn noop(self) {}
}
#[tokio::test(flavor = "multi_thread")]
async fn test_rate_limit() {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = RateLimitApp;
        let service = RateLimitService::new(app);
        let service = ServiceBuilder::new()
            .rate_limit(5000, std::time::Duration::from_secs(1))
            .service(service);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let chan = ServiceBuilder::new()
        .buffer(1)
        .rate_limit(1000, std::time::Duration::from_secs(1))
        .service(chan);
    let cli = RateLimitClient::new(chan);
    for _ in 0..N {
        // This can be commented out but to make sure thet the client is cloneable.
        let mut cli = cli.clone();
        cli.noop().await.unwrap();
    }
}
