use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use codec::Encode;
use parking_lot::Mutex;
use polymesh_worker_common::{
    BackendCodeHash, Protocol, WorkRequest, WorkResponseResult, WorkerError,
};

use crate::{blake2b256_hash, cache::modules::ProtocolModuleRef};

struct BackendOutcome {
    module: ProtocolModuleRef,
    result: Result<WorkResponseResult, WorkerError>,
}

lazy_static::lazy_static! {
    static ref LOG_FILE: Option<Mutex<BufWriter<File>>> = match std::env::var_os("WORKER_DIFF_LOG") {
        Some(path) => match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => Some(Mutex::new(BufWriter::new(file))),
            Err(err) => {
                log::error!("Failed to open WORKER_DIFF_LOG: {err}");
                None
            }
        },
        None => {
            log::warn!("worker-differential enabled without WORKER_DIFF_LOG; comparison records will not be written");
            None
        }
    };
}

pub(crate) fn initialize_log() {
    let _ = LOG_FILE.as_ref();
}

/// Execute one request on each loaded backend, log the outcomes, and return the primary result.
pub(crate) fn execute(
    protocol: Protocol,
    request: &WorkRequest,
    modules: &[ProtocolModuleRef],
) -> Result<WorkResponseResult, WorkerError> {
    let mut outcomes = Vec::with_capacity(modules.len());
    for module in modules {
        let result = module
            .get_instance()
            .ok_or(WorkerError::ModuleExecutionFailed)
            .and_then(|mut instance| instance.execute(request));
        outcomes.push(BackendOutcome {
            module: module.clone(),
            result,
        });
    }

    record(protocol, request, &outcomes);
    outcomes
        .into_iter()
        .next()
        .map(|outcome| outcome.result)
        .unwrap_or(Err(WorkerError::NoBackendAvailable))
}

pub(crate) fn record_unavailable(protocol: Protocol, request: &WorkRequest) {
    record(protocol, request, &[]);
}

fn record(protocol: Protocol, request: &WorkRequest, outcomes: &[BackendOutcome]) {
    let Some(log_file) = LOG_FILE.as_ref() else {
        return;
    };

    let fingerprints = outcomes
        .iter()
        .map(|outcome| outcome.result.encode())
        .collect::<Vec<_>>();
    let response_matched = fingerprints
        .first()
        .is_some_and(|first| fingerprints.iter().all(|result| result == first));
    let contexts = outcomes
        .iter()
        .map(|outcome| outcome.module.context_hash())
        .collect::<Vec<_>>();
    let context_matched = contexts
        .first()
        .is_some_and(|first| contexts.iter().all(|context| context == first));
    let expected = outcomes
        .first()
        .map(|outcome| outcome.module.expected_backends().to_vec())
        .unwrap_or_default();
    let loaded = outcomes
        .iter()
        .map(|outcome| outcome.module.kind())
        .collect::<Vec<_>>();
    let missing = expected
        .iter()
        .filter(|kind| !loaded.contains(kind))
        .copied()
        .collect::<Vec<_>>();
    let status = comparison_status(
        expected.len(),
        outcomes.len(),
        missing.len(),
        context_matched,
        response_matched,
    );
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let request_hash = blake2b256_hash(&request.encode());
    let config_hash = outcomes.first().map(|outcome| outcome.module.config_hash());

    let backends = outcomes
        .iter()
        .map(|outcome| {
            let result_hash = blake2b256_hash(&outcome.result.encode());
            let (outcome_kind, error) = match &outcome.result {
                Ok(Ok(_)) => ("response", None),
                Ok(Err(err)) => ("protocol_error", Some(format!("{err:?}"))),
                Err(err) => ("worker_error", Some(format!("{err:?}"))),
            };
            serde_json::json!({
                "backend": format!("{:?}", outcome.module.kind()),
                "code_hash": hex_hash(&outcome.module.code_hash()),
                "context_hash": outcome.module.context_hash().map(|hash| hex_hash(&hash)),
                "outcome": outcome_kind,
                "result_hash": hex_hash(&result_hash),
                "error": error,
            })
        })
        .collect::<Vec<_>>();

    let record = serde_json::json!({
        "timestamp_unix_ms": timestamp_ms,
        "status": status,
        "context_match": context_matched,
        "response_match": response_matched,
        "protocol": {
            "id": protocol.id.0,
            "version": {
                "major": protocol.version.major,
                "minor": protocol.version.minor,
                "patch": protocol.version.patch,
            },
        },
        "config_hash": config_hash.map(|hash| hex_hash(&hash)),
        "request_hash": hex_hash(&request_hash),
        "expected_backends": expected.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        "missing_backends": missing.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        "backends": backends,
    });

    let mut writer = log_file.lock();
    if let Err(err) = writeln!(writer, "{record}").and_then(|_| writer.flush()) {
        log::error!("Failed to write WORKER_DIFF_LOG record: {err}");
    }
}

fn comparison_status(
    expected_count: usize,
    loaded_count: usize,
    missing_count: usize,
    context_matched: bool,
    response_matched: bool,
) -> &'static str {
    if loaded_count == 0 {
        "no_backends"
    } else if expected_count < 2 {
        "insufficient_backends"
    } else if missing_count > 0 {
        "missing_backends"
    } else if !context_matched {
        "context_mismatch"
    } else if !response_matched {
        "mismatch"
    } else {
        "match"
    }
}

fn hex_hash(hash: &BackendCodeHash) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(hash.len() * 2);
    for byte in hash {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_encoded_outcomes_determine_match() {
        let first: Result<WorkResponseResult, WorkerError> =
            Ok(Ok(polymesh_worker_common::WorkResponse(vec![1, 2, 3])));
        let same: Result<WorkResponseResult, WorkerError> =
            Ok(Ok(polymesh_worker_common::WorkResponse(vec![1, 2, 3])));
        let different: Result<WorkResponseResult, WorkerError> =
            Ok(Ok(polymesh_worker_common::WorkResponse(vec![1, 2, 4])));

        assert_eq!(first.encode(), same.encode());
        assert_ne!(first.encode(), different.encode());
    }

    #[test]
    fn hashes_are_fixed_width_lowercase_hex() {
        assert_eq!(hex_hash(&[0x01; 32]), "01".repeat(32));
    }

    #[test]
    fn status_requires_complete_backend_coverage_and_matching_results() {
        assert_eq!(comparison_status(0, 0, 0, true, true), "no_backends");
        assert_eq!(
            comparison_status(1, 1, 0, true, true),
            "insufficient_backends"
        );
        assert_eq!(comparison_status(3, 2, 1, true, true), "missing_backends");
        assert_eq!(comparison_status(2, 2, 0, false, true), "context_mismatch");
        assert_eq!(comparison_status(2, 2, 0, true, false), "mismatch");
        assert_eq!(comparison_status(2, 2, 0, true, true), "match");
    }
}
