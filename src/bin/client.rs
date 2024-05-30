use futures::AsyncReadExt;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

mod interface_capnp {
    include!(concat!(env!("OUT_DIR"), "/interface_capnp.rs"));
}

struct Counter {
    count: AtomicU64,
}

impl Counter {
    fn print(&self, duration: u64) {
        println!("QPS: {}", self.count.load(Ordering::Relaxed) / duration);
    }
}

/// Bind the current thread to `core`.
#[cfg(target_os = "linux")]
fn bind_to_core(core: usize) {
    let mut cpu_set = nix::sched::CpuSet::new();
    cpu_set.set(core).unwrap();
    nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let address = std::rc::Rc::new(args.next().unwrap());
    let n_conn: usize = args.next().unwrap().parse().unwrap();
    let duration: u64 = args.next().unwrap().parse().unwrap();
    let core = 1;

    #[cfg(target_os = "linux")]
    bind_to_core(core);

    println!("INFO: Benchmark against: {}", address);
    println!("INFO: The # of connections: {}", n_conn);
    println!("INFO: Duration: {}s", duration);
    #[cfg(target_os = "linux")]
    println!("INFO: Bound to core {}", core);

    let counter = std::rc::Rc::new(Counter {
        count: AtomicU64::new(0),
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                for _ in 0..n_conn {
                    let address = std::rc::Rc::clone(&address);
                    let counter = std::rc::Rc::clone(&counter);
                    let fut = async move {
                        let stream = tokio::net::TcpStream::connect(address.deref())
                            .await
                            .unwrap();
                        stream.set_nodelay(true).unwrap();
                        // let stream_poll = stream.try_into_poll_io().unwrap();
                        let (reader, writer) =
                            tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                        let rpc_network = Box::new(capnp_rpc::twoparty::VatNetwork::new(
                            reader,
                            writer,
                            capnp_rpc::rpc_twoparty_capnp::Side::Client,
                            Default::default(),
                        ));
                        let mut rpc_system = capnp_rpc::RpcSystem::new(rpc_network, None);
                        let client: crate::interface_capnp::ping_pong::Client =
                            rpc_system.bootstrap(capnp_rpc::rpc_twoparty_capnp::Side::Server);
                        tokio::task::spawn_local(rpc_system);

                        loop {
                            let mut request = client.ping_request();
                            request.get().set_ping("ping");
                            let _response = request.send().promise.await.unwrap();
                            counter.count.fetch_add(1, Ordering::Relaxed);
                        }
                    };

                    tokio::task::spawn_local(fut);
                }

                tokio::time::sleep(Duration::from_secs(duration)).await;
                counter.print(duration);
            })
            .await;
    });
}
