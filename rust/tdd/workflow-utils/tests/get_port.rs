use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use workflow_utils::{
    WorkflowPortOptions, get_all_ports, get_port, get_workflow_port,
    parse_windows_netstat_ports_for_pid,
};

fn port_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn listener() -> TcpListener {
    TcpListener::bind(("127.0.0.1", 0)).unwrap()
}

fn listener_port(listener: &TcpListener) -> u16 {
    listener.local_addr().unwrap().port()
}

#[derive(Debug, Clone, Copy)]
enum ServerMode {
    Workflow,
    NotWorkflow,
    Slow,
}

struct TestHttpServer {
    port: u16,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl TestHttpServer {
    fn start(mode: ServerMode) -> Self {
        let listener = listener();
        let port = listener_port(&listener);
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let join = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
                        let mut bytes = [0_u8; 4096];
                        let count = stream.read(&mut bytes).unwrap_or(0);
                        let request = String::from_utf8_lossy(&bytes[..count]);
                        match mode {
                            ServerMode::Workflow => {
                                if request.contains("?__health") {
                                    write_response(
                                        &mut stream,
                                        "200 OK",
                                        "Workflow SDK endpoint is healthy",
                                    );
                                } else if request.contains("/.well-known/workflow/v1/") {
                                    write_response(
                                        &mut stream,
                                        "400 Bad Request",
                                        "Missing required headers",
                                    );
                                } else {
                                    write_response(&mut stream, "404 Not Found", "");
                                }
                            }
                            ServerMode::NotWorkflow => {
                                write_response(&mut stream, "404 Not Found", "");
                            }
                            ServerMode::Slow => {
                                for _ in 0..100 {
                                    if thread_stop.load(Ordering::Relaxed) {
                                        break;
                                    }
                                    thread::sleep(Duration::from_millis(10));
                                }
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            port,
            stop,
            join: Some(join),
        }
    }
}

impl Drop for TestHttpServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

#[test]
fn get_port_returns_none_when_no_ports_are_in_use() {
    let _guard = port_test_lock();
    assert_eq!(get_port(), None);
}

#[test]
fn get_port_handles_a_server_listening_on_a_specific_port() {
    let _guard = port_test_lock();
    let server = TcpListener::bind(("127.0.0.1", 3000)).unwrap_or_else(|_| listener());
    assert_eq!(get_port(), Some(listener_port(&server)));
}

#[test]
fn get_port_returns_the_listening_server_port() {
    let _guard = port_test_lock();
    let server = listener();
    assert_eq!(get_port(), Some(listener_port(&server)));
}

#[test]
fn get_port_returns_the_first_server_port() {
    let _guard = port_test_lock();
    let first = listener();
    let _second = listener();
    assert_eq!(get_port(), Some(listener_port(&first)));
}

#[test]
fn get_port_is_consistent_across_repeated_calls() {
    let _guard = port_test_lock();
    let server = listener();
    let expected = Some(listener_port(&server));
    assert_eq!(get_port(), expected);
    assert_eq!(get_port(), expected);
    assert_eq!(get_port(), expected);
}

#[test]
fn get_port_handles_ipv6_listeners_when_ipv6_is_available() {
    let _guard = port_test_lock();
    let Ok(server) = TcpListener::bind(("::1", 0)) else {
        return;
    };
    assert_eq!(get_port(), Some(listener_port(&server)));
}

#[test]
fn get_port_handles_multiple_calls_in_sequence() {
    let _guard = port_test_lock();
    let server = listener();
    let expected = Some(listener_port(&server));
    assert_eq!(get_port(), expected);
    assert_eq!(get_port(), expected);
}

#[test]
fn get_port_ignores_closed_servers() {
    let _guard = port_test_lock();
    let server = listener();
    let closed_port = listener_port(&server);
    drop(server);
    assert_ne!(get_port(), Some(closed_port));
}

#[test]
fn get_port_handles_a_server_restart_on_the_same_port() {
    let _guard = port_test_lock();
    let first = listener();
    let port = listener_port(&first);
    assert_eq!(get_port(), Some(port));
    drop(first);
    thread::sleep(Duration::from_millis(100));
    let second = TcpListener::bind(("127.0.0.1", port)).unwrap();
    assert_eq!(get_port(), Some(listener_port(&second)));
}

#[test]
fn get_port_handles_concurrent_calls() {
    let _guard = port_test_lock();
    let server = listener();
    let expected = listener_port(&server);
    let joins: Vec<_> = (0..10).map(|_| thread::spawn(get_port)).collect();
    for join in joins {
        assert_eq!(join.join().unwrap(), Some(expected));
    }
}

#[test]
fn get_all_ports_returns_an_empty_collection_when_nothing_is_listening() {
    let _guard = port_test_lock();
    assert_eq!(get_all_ports(), Vec::<u16>::new());
}

#[test]
fn get_all_ports_returns_every_listening_port() {
    let _guard = port_test_lock();
    let first = listener();
    let second = listener();
    let ports = get_all_ports();
    assert!(ports.contains(&listener_port(&first)));
    assert!(ports.contains(&listener_port(&second)));
    assert!(ports.len() >= 2);
}

#[test]
fn get_all_ports_returns_a_deterministic_order() {
    let _guard = port_test_lock();
    let _first = listener();
    let _second = listener();
    let first = get_all_ports();
    let second = get_all_ports();
    let third = get_all_ports();
    assert_eq!(first, second);
    assert_eq!(second, third);
}

#[test]
fn get_workflow_port_returns_none_when_no_ports_are_listening() {
    let _guard = port_test_lock();
    assert_eq!(get_workflow_port(WorkflowPortOptions::default()), None);
}

#[test]
fn get_workflow_port_returns_a_single_port_without_probing() {
    let _guard = port_test_lock();
    let server = listener();
    assert_eq!(
        get_workflow_port(WorkflowPortOptions::default()),
        Some(listener_port(&server))
    );
}

#[test]
fn get_workflow_port_identifies_the_workflow_server_among_multiple_ports() {
    let _guard = port_test_lock();
    let _not_workflow = TestHttpServer::start(ServerMode::NotWorkflow);
    let workflow = TestHttpServer::start(ServerMode::Workflow);
    assert_eq!(
        get_workflow_port(WorkflowPortOptions::default()),
        Some(workflow.port)
    );
}

#[test]
fn get_workflow_port_falls_back_to_the_first_port_when_probing_fails() {
    let _guard = port_test_lock();
    let first = TestHttpServer::start(ServerMode::NotWorkflow);
    let _second = TestHttpServer::start(ServerMode::NotWorkflow);
    assert_eq!(
        get_workflow_port(WorkflowPortOptions::default()),
        Some(first.port)
    );
}

#[test]
fn get_workflow_port_respects_a_custom_timeout() {
    let _guard = port_test_lock();
    let _slow = TestHttpServer::start(ServerMode::Slow);
    let workflow = TestHttpServer::start(ServerMode::Workflow);
    let start = Instant::now();
    let result = get_workflow_port(WorkflowPortOptions {
        timeout: Duration::from_millis(100),
        ..WorkflowPortOptions::default()
    });
    assert_eq!(result, Some(workflow.port));
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn get_workflow_port_handles_concurrent_calls() {
    let _guard = port_test_lock();
    let workflow = TestHttpServer::start(ServerMode::Workflow);
    let expected = workflow.port;
    let joins: Vec<_> = (0..5)
        .map(|_| thread::spawn(|| get_workflow_port(WorkflowPortOptions::default())))
        .collect();
    for join in joins {
        assert_eq!(join.join().unwrap(), Some(expected));
    }
}

#[test]
fn windows_netstat_parser_only_returns_listeners_owned_by_the_requested_pid() {
    let output = [
        "  Proto  Local Address          Foreign Address        State           PID",
        "  TCP    0.0.0.0:22             0.0.0.0:0              LISTENING       4",
        "  TCP    127.0.0.1:55812        0.0.0.0:0              LISTENING       4",
        "  TCP    127.0.0.1:3000         0.0.0.0:0              LISTENING       55812",
        "  TCP    [::1]:3001             [::]:0                 LISTENING       55812",
        "  TCP    127.0.0.1:3002         0.0.0.0:0              ESTABLISHED     55812",
    ]
    .join("\n");
    assert_eq!(
        parse_windows_netstat_ports_for_pid(&output, 55_812),
        vec![3000, 3001]
    );
}

#[test]
fn windows_netstat_parser_rejects_partial_or_out_of_range_ports() {
    let output = [
        "TCP 127.0.0.1:3000junk 0.0.0.0:0 LISTENING 55812",
        "TCP 127.0.0.1:65536 0.0.0.0:0 LISTENING 55812",
        "TCP 127.0.0.1:3002 0.0.0.0:0 LISTENING 55812",
    ]
    .join("\n");
    assert_eq!(
        parse_windows_netstat_ports_for_pid(&output, 55_812),
        vec![3002]
    );
}

#[test]
fn unsafe_custom_probe_endpoints_are_rejected_before_network_use() {
    let _guard = port_test_lock();
    let first = TestHttpServer::start(ServerMode::NotWorkflow);
    let _workflow = TestHttpServer::start(ServerMode::Workflow);
    let result = get_workflow_port(WorkflowPortOptions {
        endpoint: Some("/.well-known/workflow/v1/flow?__health\r\nX-Test: injected".to_owned()),
        timeout: Duration::from_millis(100),
    });
    assert_eq!(result, Some(first.port));
}
