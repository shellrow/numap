use crate::json_models::{
    JsonDomainScanResult, JsonHostScanResult, JsonPingStat, JsonPortScanResult, JsonTracerouteStat,
};
use crate::result::{PingStat, TraceResult};
use crate::{db, define, option, output, result, scan, sys};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub async fn handle_port_scan(opt: option::ScanOption) {
    let probe_opt: option::ScanOption = opt.clone();
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = channel();
    let handle = thread::spawn(move || {
        async_io::block_on(async { scan::run_service_scan(probe_opt, &msg_tx).await })
    });
    let mut pb = output::get_spinner();
    while let Ok(msg) = msg_rx.recv() {
        if msg.contains("START_") || msg.contains("END_") {
            match msg.as_str() {
                define::MESSAGE_START_PORTSCAN => {
                    pb.set_message("Scanning ports ...");
                }
                define::MESSAGE_END_PORTSCAN => {
                    pb.finish_with_message("Port scan");
                    pb = output::get_spinner();
                }
                define::MESSAGE_START_SERVICEDETECTION => {
                    pb.set_message("Detecting services ...");
                }
                define::MESSAGE_END_SERVICEDETECTION => {
                    pb.finish_with_message("Service detection");
                    pb = output::get_spinner();
                }
                define::MESSAGE_START_OSDETECTION => {
                    pb.set_message("Detecting OS ...");
                }
                define::MESSAGE_END_OSDETECTION => {
                    pb.finish_with_message("OS detection");
                    pb = output::get_spinner();
                }
                _ => {}
            }
        }
    }
    pb.finish_and_clear();
    let result: result::PortScanResult = handle.join().unwrap();
    let json_result: JsonPortScanResult =
        JsonPortScanResult::from_result(sys::get_probe_id(), result.clone());
    if opt.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error"))
        );
    } else {
        output::show_portscan_result(result.clone());
    }

    if !opt.save_file_path.is_empty() {
        output::save_json(
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error")),
            opt.save_file_path.clone(),
        );
        println!("Probe result saved to: {}", opt.save_file_path);
    }
}

pub async fn handle_host_scan(opt: option::ScanOption) {
    let mut probe_opt: option::ScanOption = opt.clone();
    probe_opt.oui_map = db::get_oui_detail_map();
    probe_opt.ttl_map = db::get_os_ttl_map();
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = channel();
    let handle = thread::spawn(move || {
        async_io::block_on(async { scan::run_node_scan(probe_opt, &msg_tx).await })
    });
    let mut pb = output::get_spinner();
    while let Ok(msg) = msg_rx.recv() {
        if msg.contains("START_") || msg.contains("END_") {
            match msg.as_str() {
                define::MESSAGE_START_HOSTSCAN => {
                    pb.set_message("Scanning hosts ...");
                }
                define::MESSAGE_END_HOSTSCAN => {
                    pb.finish_with_message("Host scan");
                    pb = output::get_spinner();
                }
                define::MESSAGE_START_LOOKUP => {
                    pb.set_message("Lookup ...");
                }
                define::MESSAGE_END_LOOKUP => {
                    pb.finish_with_message("Lookup");
                    pb = output::get_spinner();
                }
                _ => {}
            }
        }
    }
    pb.finish_and_clear();
    let result: result::HostScanResult = handle.join().unwrap();
    let json_result: JsonHostScanResult =
        JsonHostScanResult::from_result(sys::get_probe_id(), result.clone());
    if opt.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error"))
        );
    } else {
        output::show_hostscan_result(result.clone());
    }

    if !opt.save_file_path.is_empty() {
        output::save_json(
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error")),
            opt.save_file_path.clone(),
        );
        println!("Probe result saved to: {}", opt.save_file_path);
    }
}

pub fn handle_ping(opt: option::ScanOption) {
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = channel();
    let ping_opt: option::ScanOption = opt.clone();
    let handle = thread::spawn(move || scan::run_ping(ping_opt, &msg_tx));
    while let Ok(msg) = msg_rx.recv() {
        println!("{}", msg);
    }
    let result: PingStat = handle.join().unwrap();
    let json_result: JsonPingStat = JsonPingStat::from_result(sys::get_probe_id(), result.clone());
    if opt.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error"))
        );
    } else {
        output::show_ping_result(result.clone());
    }

    if !opt.save_file_path.is_empty() {
        output::save_json(
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error")),
            opt.save_file_path.clone(),
        );
        println!("Probe result saved to: {}", opt.save_file_path);
    }
}

pub fn handle_trace(opt: option::ScanOption) {
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = channel();
    let trace_opt: option::ScanOption = opt.clone();
    let handle = thread::spawn(move || scan::run_traceroute(trace_opt, &msg_tx));
    while let Ok(msg) = msg_rx.recv() {
        println!("{}", msg);
    }
    let result: TraceResult = handle.join().unwrap();
    let json_result: JsonTracerouteStat =
        JsonTracerouteStat::from_result(sys::get_probe_id(), result.clone());
    if opt.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error"))
        );
    } else {
        output::show_trace_result(result.clone());
    }

    if !opt.save_file_path.is_empty() {
        output::save_json(
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error")),
            opt.save_file_path.clone(),
        );
        println!("Probe result saved to: {}", opt.save_file_path);
    }
}

pub fn handle_domain_scan(opt: option::ScanOption) {
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = channel();
    let probe_opt: option::ScanOption = opt.clone();
    let handle = thread::spawn(move || scan::run_domain_scan(probe_opt, &msg_tx));
    let mut pb = output::get_spinner();
    while let Ok(msg) = msg_rx.recv() {
        if msg.contains("START_") || msg.contains("END_") {
            match msg.as_str() {
                define::MESSAGE_START_DOMAINSCAN => {
                    pb.set_message("Scanning domains ...");
                }
                define::MESSAGE_END_DOMAINSCAN => {
                    pb.finish_with_message("Domain scan");
                    pb = output::get_spinner();
                }
                _ => {}
            }
        }
    }
    pb.finish_and_clear();
    let result: result::DomainScanResult = handle.join().unwrap();
    let json_result: JsonDomainScanResult =
        JsonDomainScanResult::from_result(sys::get_probe_id(), result.clone());
    if opt.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error"))
        );
    } else {
        output::show_domainscan_result(result.clone());
    }

    if !opt.save_file_path.is_empty() {
        output::save_json(
            serde_json::to_string_pretty(&json_result).unwrap_or(String::from("Serialize Error")),
            opt.save_file_path.clone(),
        );
        println!("Probe result saved to: {}", opt.save_file_path);
    }
}
