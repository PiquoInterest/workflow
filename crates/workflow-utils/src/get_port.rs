use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use crate::WORKFLOW_ROUTE_BASE;

const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_PROBE_ENDPOINT_BYTES: usize = 8 * 1024;
const MAX_STATUS_LINE_BYTES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPortOptions {
    pub endpoint: Option<String>,
    pub timeout: Duration,
}

impl Default for WorkflowPortOptions {
    fn default() -> Self {
        Self {
            endpoint: None,
            timeout: DEFAULT_PROBE_TIMEOUT,
        }
    }
}

#[must_use]
pub fn get_port() -> Option<u16> {
    get_all_ports().into_iter().next()
}

#[must_use]
pub fn get_all_ports() -> Vec<u16> {
    platform::get_ports(std::process::id())
}

#[must_use]
pub fn get_workflow_port(options: WorkflowPortOptions) -> Option<u16> {
    let ports = get_all_ports();
    match ports.as_slice() {
        [] => None,
        [only] => Some(*only),
        _ => {
            let probes = ports
                .iter()
                .copied()
                .map(|port| {
                    let options = options.clone();
                    thread::spawn(move || (port, probe_port(port, &options)))
                })
                .collect::<Vec<_>>();

            for probe in probes {
                if let Ok((port, true)) = probe.join() {
                    return Some(port);
                }
            }

            ports.first().copied()
        }
    }
}

#[must_use]
pub fn parse_windows_netstat_ports_for_pid(output: &str, process_id: u32) -> Vec<u16> {
    output
        .lines()
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 5
                || !parts[0].eq_ignore_ascii_case("TCP")
                || !parts[3].eq_ignore_ascii_case("LISTENING")
                || parts[4].parse::<u32>().ok()? != process_id
            {
                return None;
            }

            let (_, port) = parts[1].rsplit_once(':')?;
            parse_decimal_port(port)
        })
        .collect()
}

fn probe_port(port: u16, options: &WorkflowPortOptions) -> bool {
    let default_endpoint = format!("{WORKFLOW_ROUTE_BASE}/flow?__health");
    let endpoint = options.endpoint.as_deref().unwrap_or(&default_endpoint);
    if !is_safe_probe_endpoint(endpoint) || options.timeout.is_zero() {
        return false;
    }

    let deadline = Instant::now().checked_add(options.timeout);
    let addresses = [
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port),
    ];

    addresses
        .into_iter()
        .any(|address| probe_address(address, endpoint, deadline, options.timeout))
}

fn probe_address(
    address: SocketAddr,
    endpoint: &str,
    deadline: Option<Instant>,
    fallback_timeout: Duration,
) -> bool {
    let Some(connect_timeout) = remaining_timeout(deadline, fallback_timeout) else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&address, connect_timeout) else {
        return false;
    };

    let Some(io_timeout) = remaining_timeout(deadline, fallback_timeout) else {
        return false;
    };
    if stream.set_read_timeout(Some(io_timeout)).is_err()
        || stream.set_write_timeout(Some(io_timeout)).is_err()
    {
        return false;
    }

    let request = format!(
        "HEAD {endpoint} HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        address.port()
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    read_status_code(&mut stream) == Some(200)
}

fn remaining_timeout(deadline: Option<Instant>, fallback: Duration) -> Option<Duration> {
    match deadline {
        Some(deadline) => deadline.checked_duration_since(Instant::now()).filter(|value| !value.is_zero()),
        None => Some(fallback),
    }
}

fn read_status_code(stream: &mut TcpStream) -> Option<u16> {
    let mut response = Vec::with_capacity(128);
    let mut buffer = [0_u8; 256];

    while response.len() < MAX_STATUS_LINE_BYTES {
        let count = stream.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        let remaining = MAX_STATUS_LINE_BYTES - response.len();
        response.extend_from_slice(&buffer[..count.min(remaining)]);
        if response.windows(2).any(|window| window == b"\r\n") {
            break;
        }
    }

    let end = response.windows(2).position(|window| window == b"\r\n")?;
    let status_line = std::str::from_utf8(&response[..end]).ok()?;
    let mut parts = status_line.split_whitespace();
    let version = parts.next()?;
    let status = parts.next()?;
    if !version.starts_with("HTTP/") || parts.next().is_none() {
        return None;
    }
    status.parse::<u16>().ok()
}

fn is_safe_probe_endpoint(endpoint: &str) -> bool {
    !endpoint.is_empty()
        && endpoint.starts_with('/')
        && endpoint.len() <= MAX_PROBE_ENDPOINT_BYTES
        && endpoint
            .bytes()
            .all(|byte| !byte.is_ascii_control() && byte != b' ')
}

fn parse_decimal_port(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse::<u16>().ok()
}

fn parse_hex_port(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u16::from_str_radix(value, 16).ok()
}

#[cfg(target_os = "linux")]
mod platform {
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::PathBuf;

    use super::parse_hex_port;

    pub(super) fn get_ports(process_id: u32) -> Vec<u16> {
        let fd_root = PathBuf::from(format!("/proc/{process_id}/fd"));
        let Ok(entries) = fs::read_dir(fd_root) else {
            return Vec::new();
        };

        let mut descriptors = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let descriptor = entry.file_name().to_str()?.parse::<u64>().ok()?;
                Some((descriptor, entry.path()))
            })
            .collect::<Vec<_>>();
        descriptors.sort_unstable_by_key(|(descriptor, _)| *descriptor);

        let socket_inodes = descriptors
            .into_iter()
            .filter_map(|(_, path)| fs::read_link(path).ok())
            .filter_map(|target| {
                let target = target.to_string_lossy();
                let inode = target.strip_prefix("socket:[")?.strip_suffix(']')?;
                if inode.is_empty() || !inode.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                Some(inode.to_owned())
            })
            .collect::<Vec<_>>();
        if socket_inodes.is_empty() {
            return Vec::new();
        }

        let wanted = socket_inodes.iter().cloned().collect::<HashSet<_>>();
        let mut inode_to_port = HashMap::new();
        for table in ["/proc/net/tcp", "/proc/net/tcp6"] {
            let Ok(contents) = fs::read_to_string(table) else {
                continue;
            };
            for line in contents.lines().skip(1) {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                if fields.len() < 10 || fields[3] != "0A" || !wanted.contains(fields[9]) {
                    continue;
                }
                let Some((_, port_hex)) = fields[1].split_once(':') else {
                    continue;
                };
                if let Some(port) = parse_hex_port(port_hex) {
                    inode_to_port.insert(fields[9].to_owned(), port);
                }
            }
        }

        socket_inodes
            .into_iter()
            .filter_map(|inode| inode_to_port.get(&inode).copied())
            .collect()
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;

    use super::parse_decimal_port;

    pub(super) fn get_ports(process_id: u32) -> Vec<u16> {
        let process_id = process_id.to_string();
        let Ok(output) = Command::new("lsof")
            .args(["-a", "-i", "-P", "-n", "-p", process_id.as_str()])
            .output()
        else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.contains("LISTEN"))
            .filter_map(|line| line.split_whitespace().nth(8))
            .filter_map(|address| address.rsplit_once(':').map(|(_, port)| port))
            .filter_map(parse_decimal_port)
            .collect()
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::process::Command;

    use super::parse_windows_netstat_ports_for_pid;

    pub(super) fn get_ports(process_id: u32) -> Vec<u16> {
        let Ok(output) = Command::new("cmd")
            .args(["/c", "netstat -ano -p tcp | findstr LISTENING"])
            .output()
        else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        parse_windows_netstat_ports_for_pid(
            &String::from_utf8_lossy(&output.stdout),
            process_id,
        )
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod platform {
    pub(super) fn get_ports(_process_id: u32) -> Vec<u16> {
        Vec::new()
    }
}
