use futures::AsyncReadExt;

mod interface_capnp {
    include!(concat!(env!("OUT_DIR"), "/interface_capnp.rs"));
}

/// Bind the current thread to `core`.
#[cfg(target_os = "linux")]
fn bind_to_core(core: usize) {
    let mut cpu_set = nix::sched::CpuSet::new();
    cpu_set.set(core).unwrap();
    nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();
}

struct Impl;

impl interface_capnp::ping_pong::Server for Impl {
    fn ping(
        &mut self,
        capnp_request: interface_capnp::ping_pong::PingParams,
        mut capnp_response: interface_capnp::ping_pong::PingResults,
    ) -> capnp::capability::Promise<(), ::capnp::Error> {
        let _request = capnp_request
            .get()
            .unwrap()
            .get_ping()
            .unwrap()
            .to_string()
            .unwrap();
        capnp_response.get().set_pong("pong");
        capnp::capability::Promise::ok(())
    }
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let address = args.next().unwrap();
    let core = 0;

    #[cfg(target_os = "linux")]
    bind_to_core(core);

    println!("INFO: Listening on {}", address);
    #[cfg(target_os = "linux")]
    println!("INFO: Bound to core {}", core);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async move {
                let listener = tokio::net::TcpListener::bind(address).await.unwrap();
                let client: interface_capnp::ping_pong::Client = capnp_rpc::new_client(Impl);
                while let Ok((stream, _)) = listener.accept().await {
                    stream.set_nodelay(true).unwrap();
                    let client_clone = client.clone();
                    let fut = async move {
                        // let stream_poll = stream.try_into_poll_io().unwrap();
                        let (reader, writer) =
                            tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();

                        let network = capnp_rpc::twoparty::VatNetwork::new(
                            reader,
                            writer,
                            capnp_rpc::rpc_twoparty_capnp::Side::Server,
                            Default::default(),
                        );

                        let rpc_system =
                            capnp_rpc::RpcSystem::new(Box::new(network), Some(client_clone.client));

                        tokio::task::spawn_local(rpc_system);
                    };

                    tokio::task::spawn_local(fut);
                }
            })
            .await;
    })
}
