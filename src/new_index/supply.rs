use std::fs;
use std::process;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use error_chain::ChainedError;

use crate::daemon::Daemon;
use crate::util::bincode;

use super::db::{DB, DBRow};
use super::schema::Store;

const SUPPLY_KEY: &[u8] = b"Y";
const BOOTSTRAP_MARKER: &str = "/run/electrs-supply-boot";

/// Persisted total supply counter, written atomically together with the
/// history rows of the blocks that changed it.
#[derive(Serialize, Deserialize, Debug)]
struct SupplyValue {
    value: u64,  // total supply in base units (e.g. satoshis)
    height: u32, // blocks up to and including this height are covered by `value`
}

enum State {
    /// Baseline unknown yet: remember per-block deltas until bootstrap completes.
    Uninit { pending: Vec<(u32, i64)> },
    /// Counter is live: `total` covers all blocks <= `skip_below`.
    Ready { total: u64, skip_below: u32 },
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State::Uninit { pending: Vec::new() });
}

fn apply_net(total: u64, net: i64) -> u64 {
    if net >= 0 {
        total.saturating_add(net as u64)
    } else {
        total.saturating_sub(net.unsigned_abs())
    }
}

/// Load the persisted counter (if any) before indexing starts.
pub fn init(db: &DB) {
    let loaded = db
        .get(SUPPLY_KEY)
        .and_then(|bytes| bincode::deserialize_little::<SupplyValue>(&bytes).ok());
    if let Some(sv) = loaded {
        let mut st = STATE.lock().unwrap();
        *st = State::Ready { total: sv.value, skip_below: sv.height };
        info!(
            "total supply counter loaded: {} base units (height {})",
            sv.value, sv.height
        );
    } else {
        info!("total supply counter missing, one-shot daemon bootstrap required");
    }
}

/// Fold the per-block supply deltas of an indexing chunk into the counter.
/// The counter row is appended to `rows`, so it is persisted in the very same
/// WriteBatch as the history rows of the blocks it accounts for.
pub fn apply(block_nets: &[(u32, i64)], rows: &mut Vec<DBRow>) {
    if block_nets.is_empty() {
        return;
    }
    let mut st = STATE.lock().unwrap();
    match &mut *st {
        State::Ready { total, skip_below } => {
            for (height, net) in block_nets {
                if *height > *skip_below {
                    *total = apply_net(*total, *net);
                    *skip_below = *height;
                }
            }
            let value = SupplyValue { value: *total, height: *skip_below };
            let bytes = bincode::serialize_little(&value)
                .expect("failed to serialize supply counter");
            rows.push(DBRow { key: SUPPLY_KEY.to_vec(), value: bytes });
        }
        State::Uninit { pending } => {
            pending.extend_from_slice(block_nets);
        }
    }
}

/// Current total supply in base units, if the counter is ready.
pub fn get() -> Option<u64> {
    match &*STATE.lock().unwrap() {
        State::Ready { total, .. } => Some(*total),
        State::Uninit { .. } => None,
    }
}

/// One-shot baseline via the daemon's `gettxoutsetinfo`, run once in a
/// background thread. Afterwards the counter is maintained incrementally and
/// this expensive RPC is never called again.
pub fn spawn_bootstrap(store: Arc<Store>, daemon: Arc<Daemon>) {
    {
        let st = STATE.lock().unwrap();
        if let State::Ready { .. } = *st {
            return;
        }
    }
    let _ = fs::write(BOOTSTRAP_MARKER, format!("{}", process::id()));
    thread::spawn(move || {
        loop {
            match daemon.gettxoutsetinfo() {
                Ok(info) => {
                    // total_amount is reported in whole coins (f64)
                    let baseline = (info.total_amount * 1e8).round() as u64;
                    finalize(store.history_db(), baseline, info.height);
                    info!(
                        "total supply baseline bootstrapped from gettxoutsetinfo: \
                         {} base units at height {}",
                        baseline, info.height
                    );
                    break;
                }
                Err(e) => {
                    warn!("supply bootstrap failed, retrying in 30s: {}", e.display_chain())
                }
            }
            thread::sleep(Duration::from_secs(30));
        }
        let _ = fs::remove_file(BOOTSTRAP_MARKER);
    });
}

fn finalize(db: &DB, baseline: u64, height: u32) {
    let mut st = STATE.lock().unwrap();
    let pending = match ::std::mem::replace(
        &mut *st,
        State::Uninit { pending: Vec::new() },
    ) {
        State::Ready { total, skip_below } => {
            *st = State::Ready { total, skip_below };
            return;
        }
        State::Uninit { pending } => pending,
    };

    // Baseline covers all blocks <= `height`. Deltas from blocks indexed
    // while the bootstrap ran that are *above* that height must be added;
    // everything at or below it is already part of the baseline.
    let mut total = baseline;
    let mut covered_height = height;
    for (block_height, net) in &pending {
        if *block_height > height {
            total = apply_net(total, *net);
            if *block_height > covered_height {
                covered_height = *block_height;
            }
        }
    }

    let value = SupplyValue { value: total, height: covered_height };
    let bytes = bincode::serialize_little(&value).expect("failed to serialize supply counter");
    db.put_sync(SUPPLY_KEY, &bytes);
    *st = State::Ready { total, skip_below: covered_height };
}
