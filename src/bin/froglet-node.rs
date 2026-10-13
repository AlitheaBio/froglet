use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    // A builtin worker serves one request under its supervisor's RLIMIT_AS.
    // A multi-thread runtime starts one thread per CPU, each reserving a stack
    // and a malloc arena, so on a many-core host the worker exhausted its
    // address space before it could spawn its last runtime thread.
    let worker = std::env::args().nth(1).as_deref() == Some(froglet::builtin_worker::SUBCOMMAND);
    let runtime = if worker {
        tokio::runtime::Builder::new_current_thread()
    } else {
        tokio::runtime::Builder::new_multi_thread()
    }
    .enable_all()
    .build()
    .expect("Failed building the Runtime");
    runtime.block_on(run())
}

async fn run() -> ExitCode {
    // Reqwest is compiled without an implicit crypto provider so the release
    // carries one Ring-backed Rustls implementation instead of Ring plus
    // AWS-LC. Install it before any CLI or server path constructs a client.
    froglet::tls::ensure_rustls_crypto_provider();

    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(String::as_str);
    if subcommand == Some(froglet::builtin_worker::SUBCOMMAND) {
        let result = match froglet::builtin_worker::read_request() {
            Ok(request) => froglet::builtin_worker::dispatch_standard(request).await,
            Err(error) => Err(error),
        };
        froglet::builtin_worker::write_result(result);
        return ExitCode::SUCCESS;
    }

    // CLI subcommands (Phase 2 of agent-grade publish): cli::* modules
    // handle init / build / publish / invoke / whoami. They share a
    // CliError type with stable exit codes so wrapping scripts can
    // branch.
    if let Some(name) = subcommand
        && let Some(handler) = lookup_cli_handler(name)
    {
        let rest = args[2..].to_vec();
        let json_mode = rest.iter().any(|argument| argument == "--json");
        let result = handler.run(rest).await;
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                let exit_code = e.exit_code() as u8;
                if json_mode {
                    let mut report = froglet::cli::doctor::error_report(&e);
                    report["exit_code"] = serde_json::json!(exit_code);
                    println!("{report}");
                } else {
                    eprintln!("error: {e}");
                }
                ExitCode::from(exit_code)
            }
        };
    }

    // Legacy identity utilities (added before the cli/ module existed).
    if let Some(handler) = subcommand.and_then(legacy_identity_handler) {
        return match handler() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        };
    }

    if matches!(subcommand, Some("help" | "--help" | "-h")) {
        print_help();
        return ExitCode::SUCCESS;
    }

    // An unrecognized subcommand must fail loudly. Silently falling through
    // to server mode turned typos into a daemon start (and stale binaries
    // into surprise servers); only a bare `froglet-node` runs the server.
    if let Some(unknown) = subcommand {
        eprintln!("error: unknown subcommand {unknown:?}\n");
        print_help();
        return ExitCode::FAILURE;
    }

    // No subcommand → run the server (preserves backward compat with
    // existing docker-compose / installer / agent-bootstrap callers).
    let role = match std::env::var("FROGLET_NODE_ROLE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "provider" => froglet::server::ServiceRole::Provider,
        // "requester" accepted as alias for "runtime"
        "runtime" | "requester" => froglet::server::ServiceRole::Runtime,
        // Default: both provider and runtime on one node
        _ => froglet::server::ServiceRole::Dual,
    };
    match froglet::server::run_with_role(role).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("server error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Boxed future trait for async cli subcommand handlers. Returning
/// the trait object keeps the lookup table simple and avoids generic
/// fan-out across 5 handlers.
trait CliHandler {
    fn run<'a>(
        &'a self,
        args: Vec<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), froglet::cli::CliError>> + Send + 'a>,
    >;
}

macro_rules! make_async_handler {
    ($name:ident, $module:path) => {
        struct $name;
        impl CliHandler for $name {
            fn run<'a>(
                &'a self,
                args: Vec<String>,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = Result<(), froglet::cli::CliError>>
                        + Send
                        + 'a,
                >,
            > {
                Box::pin($module(args))
            }
        }
    };
}

macro_rules! make_sync_handler {
    ($name:ident, $module:path) => {
        struct $name;
        impl CliHandler for $name {
            fn run<'a>(
                &'a self,
                args: Vec<String>,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = Result<(), froglet::cli::CliError>>
                        + Send
                        + 'a,
                >,
            > {
                let result = $module(args);
                Box::pin(async move { result })
            }
        }
    };
}

make_sync_handler!(InitHandler, froglet::cli::init::run);
make_sync_handler!(ConfigureAgentHandler, froglet::cli::configure_agent::run);
make_async_handler!(BuildHandler, froglet::cli::build::run);
make_async_handler!(PublishHandler, froglet::cli::publish::run);
make_async_handler!(WhoamiHandler, froglet::cli::whoami::run);
make_async_handler!(SafeguardsHandler, froglet::cli::safeguards::run);
make_async_handler!(DownloadHandler, froglet::cli::download::run);
make_async_handler!(InvokeHandler, froglet::cli::invoke::run);
make_async_handler!(McpHandler, froglet::cli::mcp::run);
make_async_handler!(PrepareHandler, froglet::cli::prepare::run);
make_async_handler!(PrepareHttpHandler, froglet::cli::http_service::run);
make_sync_handler!(UpdatesHandler, froglet::cli::prepare::run_updates);
make_async_handler!(DoctorHandler, froglet::cli::doctor::run);
make_async_handler!(StatusHandler, froglet::local_status::run);
make_async_handler!(AttestHandler, froglet::cli::attest::run);
make_sync_handler!(IdentityHandler, froglet::cli::identity::run);

fn lookup_cli_handler(name: &str) -> Option<Box<dyn CliHandler>> {
    match name {
        "init" => Some(Box::new(InitHandler)),
        "configure-agent" => Some(Box::new(ConfigureAgentHandler)),
        "build" => Some(Box::new(BuildHandler)),
        "publish" => Some(Box::new(PublishHandler)),
        "whoami" => Some(Box::new(WhoamiHandler)),
        "safeguards" => Some(Box::new(SafeguardsHandler)),
        "download" => Some(Box::new(DownloadHandler)),
        "invoke" => Some(Box::new(InvokeHandler)),
        "mcp" => Some(Box::new(McpHandler)),
        "prepare-http-service" => Some(Box::new(PrepareHttpHandler)),
        "prepare-service" => Some(Box::new(PrepareHandler)),
        "check-updates" => Some(Box::new(UpdatesHandler)),
        "doctor" => Some(Box::new(DoctorHandler)),
        "status" => Some(Box::new(StatusHandler)),
        "attest-dns-record" => Some(Box::new(AttestHandler)),
        "identity" => Some(Box::new(IdentityHandler)),
        _ => None,
    }
}

type LegacyIdentityFn = fn() -> Result<(), Box<dyn std::error::Error>>;

fn legacy_identity_handler(name: &str) -> Option<LegacyIdentityFn> {
    match name {
        "sign-message" => Some(run_sign_message),
        "print-identity" => Some(run_print_identity),
        _ => None,
    }
}

/// Read a message from stdin and emit a hex BIP340 Schnorr signature over
/// the message using the node identity. Used by providers who need to
/// produce signed claims (e.g. marketplace provider-domain claims under
/// `<slug>.providers.<suffix>`) without standing up a separate signing
/// tool. Loads the same identity the server would, from the
/// `FROGLET_DATA_DIR`-rooted seed file.
///
/// Stderr gets a one-line `provider_id: <pubkey>` so a caller can see
/// which identity signed the message; stdout is just the signature hex
/// so the output is pipe-friendly.
fn run_sign_message() -> Result<(), Box<dyn std::error::Error>> {
    let identity = load_identity_for_cli()?;
    let mut message = Vec::new();
    std::io::stdin().read_to_end(&mut message)?;
    eprintln!("provider_id: {}", identity.node_id());
    println!("{}", identity.sign_message_hex(&message));
    Ok(())
}

/// Print the node's provider_id (uncompressed-secp256k1 pubkey hex) so a
/// provider can copy it into marketplace claims without parsing the seed
/// file by hand.
fn run_print_identity() -> Result<(), Box<dyn std::error::Error>> {
    let identity = load_identity_for_cli()?;
    println!("{}", identity.node_id());
    Ok(())
}

fn load_identity_for_cli() -> Result<froglet::identity::NodeIdentity, Box<dyn std::error::Error>> {
    let config = froglet::config::NodeConfig::from_env()
        .map_err(|e| format!("failed to load node config: {e}"))?;
    let identity = froglet::identity::NodeIdentity::load_or_create(&config)
        .map_err(|e| format!("failed to load node identity: {e}"))?;
    Ok(identity)
}

fn print_help() {
    println!(
        "froglet-node — Froglet provider/runtime daemon + author CLI\n\
         \n\
         Server mode (no subcommand):\n  \
           froglet-node                          run the server (FROGLET_NODE_ROLE selects role)\n\
         \n\
         Author commands (Phase 2 of agent-grade publish):\n  \
           froglet-node init <name>              scaffold a new Froglet service project\n  \
           froglet-node build                    validate manifests + build artifact (no publish)\n  \
           froglet-node publish [--host X]       publish service to marketplace via the local daemon\n  \
           froglet-node whoami                   print identity + daemon info\n  \
           froglet-node safeguards status       inspect effective limits, activity and pause state\n  \
           froglet-node safeguards pause        stop new admissions without clearing usage\n  \
           froglet-node safeguards resume       resume within remaining allowances\n  \
           froglet-node safeguards prune-cache  remove disposable indexes older than 24 hours\n  \
           froglet-node safeguards invite-create  issue an expiring invitation to a private file\n  \
           froglet-node safeguards invite-list    list invitation usage and expiry\n  \
           froglet-node safeguards invite-revoke  revoke an invitation immediately\n  \
           froglet-node invoke <id|share-url> [input]  invoke a service; share URLs require verified free terms\n\
           froglet-node mcp [--probe]            native agent MCP bridge / command-line probe\n\
         \n\
           froglet-node prepare-http-service --request FILE  prepare a fixed HTTP operation\n\
           froglet-node prepare-service --request FILE  prepare a selected catalog, file download, or Wasm service\n\
           froglet-node download --service-url URL --destination PATH  verify and save a shared file\n\
           froglet-node safeguards abort-files  stop active file responses without clearing usage\n\
           froglet-node check-updates --json     check registered source files\n\
           froglet-node doctor --json            diagnose installation, runtime, relay, and agent\n\
           froglet-node status --open --json     get a read-only local status page link\n\
           --provider-id ID --provider-url URL   select a remote HTTPS service (invoke)\n\
           --idempotency-key KEY                 reconcile an uncertain invocation\n\
         \n\
         Identity utilities:\n  \
           froglet-node identity <command>       generate, back up, restore, rotate, or verify identity custody\n  \
           froglet-node sign-message             read a message from stdin and emit a hex Schnorr signature\n  \
           froglet-node print-identity           print the node's provider_id (pubkey hex)\n  \
           froglet-node attest-dns-record <zone> print the signed _froglet.<zone> TXT record for DNS attestation\n\
         \n\
         Common flags:\n  \
           --json                                emit machine-readable output\n  \
           --host local|relay|tor|self           override the manifest's hosting choice (publish only)\n  \
           --marketplace URL                     override the marketplace URL (publish only)\n  \
           --no-wait | --timeout-secs N          skip or bound polling for the deal result (invoke only)\n  \
         \n  \
           froglet-node help                     show this message\n"
    );
}
