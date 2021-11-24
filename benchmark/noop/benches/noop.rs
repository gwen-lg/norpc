use tokio::sync::mpsc;
use tower::Service;

#[norpc::service]
trait Noop {
    fn noop();
}

#[derive(Clone)]
struct NoopApp;
#[norpc::async_trait]
impl Noop for NoopApp {
    async fn noop(self) {
        // tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_noop(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    rt.spawn(async move {
        let app = NoopApp;
        let service = NoopService::new(app);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let cli = NoopClient::new(chan);
    c.bench_with_input(BenchmarkId::new("noop request", 1), &cli, |b, cli| {
        b.to_async(&rt).iter(|| async {
            let mut cli = cli.clone();
            cli.noop().await.unwrap();
        });
    });
}

criterion_group!(noop, bench_noop);
criterion_main!(noop);
