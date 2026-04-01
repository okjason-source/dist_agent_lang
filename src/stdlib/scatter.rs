//! **Scatter** — process-local “time to do work” for DAL programs.
//!
//! Internally Scatter keeps a **schedule** (per-id next fire time), a **min-heap**
//! of next-fire `(Instant, id)` (with lazy stale removal), and a **due queue**
//! (FIFO ids ready for [`pending`]). The worker thread sleeps until the next due
//! time or until a **notify** from [`after_ms`], [`every_ms`], or [`cancel`].
//!
//! ## Fleets
//!
//! Fleets are **named sets of agent ids** (see [`crate::fleet`]); Scatter does
//! **not** push timers across processes. The useful pattern is composition in
//! one long-running host (e.g. `dal serve`): Scatter says *when*, your handler
//! says *what* — e.g. on job `"fleet:tick"` call `fleet::run("my-fleet")` or
//! `fleet::deploy`. Multi-process fleet coordination stays out of band.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::runtime::values::Value;

/// Default `SCATTER_TICK_MS` when the env var is unset (minimum sleep to avoid
/// hot spinning when many jobs fire at the same instant).
const DEFAULT_SCATTER_TICK_MS: u64 = 50;

struct ScatterState {
    /// Per-id next fire time and repeat rule.
    schedule: HashMap<String, Job>,
    /// FIFO queue of job ids that are due and await [`pending`] (drain).
    due_queue: VecDeque<String>,
    /// Min-heap by next fire time; stale entries possible until popped (see [`process_due`]).
    heap: BinaryHeap<(Reverse<Instant>, String)>,
}

impl Default for ScatterState {
    fn default() -> Self {
        Self {
            schedule: HashMap::new(),
            due_queue: VecDeque::new(),
            heap: BinaryHeap::new(),
        }
    }
}

struct Job {
    kind: JobKind,
    next: Instant,
}

enum JobKind {
    Once,
    Every { interval_ms: u64 },
}

struct ScatterSync {
    state: Mutex<ScatterState>,
    cv: Condvar,
}

impl ScatterSync {
    fn notify(&self) {
        self.cv.notify_one();
    }
}

static SYNC: OnceLock<Arc<ScatterSync>> = OnceLock::new();

fn tick_ms() -> u64 {
    static T: OnceLock<u64> = OnceLock::new();
    *T.get_or_init(|| {
        std::env::var("SCATTER_TICK_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_SCATTER_TICK_MS)
            .max(1)
            .min(3_600_000)
    })
}

fn sync() -> &'static Arc<ScatterSync> {
    SYNC.get_or_init(|| {
        let s = Arc::new(ScatterSync {
            state: Mutex::new(ScatterState::default()),
            cv: Condvar::new(),
        });
        let worker = s.clone();
        std::thread::Builder::new()
            .name("dal-scatter-tick".into())
            .spawn(move || worker_loop(worker))
            .expect("dal-scatter-tick thread");
        s
    })
}

fn ensure_worker() {
    let _ = sync();
}

/// Pop from the heap while `next_fire <= now`, validate against `schedule`, move to `due_queue`.
fn process_due(st: &mut ScatterState, now: Instant) {
    while let Some((Reverse(t), id)) = st.heap.peek().cloned() {
        if t > now {
            break;
        }
        st.heap.pop();
        let Some(job) = st.schedule.get(&id) else {
            continue;
        };
        if job.next != t {
            continue;
        }
        match job.kind {
            JobKind::Once => {
                st.schedule.remove(&id);
                st.due_queue.push_back(id);
            }
            JobKind::Every { interval_ms } => {
                let next = now + Duration::from_millis(interval_ms);
                if let Some(j) = st.schedule.get_mut(&id) {
                    j.next = next;
                }
                st.due_queue.push_back(id.clone());
                st.heap.push((Reverse(next), id));
            }
        }
    }
}

fn worker_loop(sync: Arc<ScatterSync>) {
    loop {
        let mut guard = match sync.state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let now = Instant::now();
        process_due(&mut guard, now);
        let next_fire = guard.heap.peek().map(|(Reverse(t), _)| *t);
        match next_fire {
            None => {
                drop(sync.cv.wait(guard).unwrap_or_else(|e| e.into_inner()));
            }
            Some(t) => {
                let now = Instant::now();
                let dur = t.saturating_duration_since(now);
                if dur.is_zero() {
                    drop(guard);
                    std::thread::sleep(Duration::from_millis(tick_ms()));
                    continue;
                }
                drop(
                    sync.cv
                        .wait_timeout(guard, dur)
                        .unwrap_or_else(|e| e.into_inner())
                        .0,
                );
            }
        }
    }
}

fn ms_to_u64(ms: i64) -> Result<u64, String> {
    if ms < 0 {
        return Err("delay must be non-negative".into());
    }
    Ok(ms as u64)
}

/// One-shot at **wall-clock** Unix ms (same semantics as [`after_ms`] after computing delay).
/// For parsing user/LLM strings, use [`crate::stdlib::time::parse_rfc3339_unix_ms`] then this, or call [`crate::stdlib::time::delay_ms_until_unix_ms`] + [`after_ms`].
pub fn after_at_unix_ms(unix_ms: i64, id: &str) -> Result<(), String> {
    let delay = crate::stdlib::time::delay_ms_until_unix_ms(unix_ms);
    after_ms(delay, id)
}

/// Schedule a one-shot job. `id` must be non-empty; duplicate `id` replaces the
/// previous schedule.
pub fn after_ms(delay_ms: i64, id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("scatter job id must not be empty".into());
    }
    let ms = ms_to_u64(delay_ms)?;
    ensure_worker();
    let s = sync();
    let next = Instant::now() + Duration::from_millis(ms);
    let mut st = s
        .state
        .lock()
        .map_err(|_| "scatter lock poisoned".to_string())?;
    st.schedule.insert(
        id.to_string(),
        Job {
            kind: JobKind::Once,
            next,
        },
    );
    st.heap.push((Reverse(next), id.to_string()));
    drop(st);
    s.notify();
    Ok(())
}

/// Schedule a repeating job every `interval_ms`. Duplicate `id` replaces.
pub fn every_ms(interval_ms: i64, id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("scatter job id must not be empty".into());
    }
    let ms = ms_to_u64(interval_ms)?;
    if ms == 0 {
        return Err("every_ms interval must be > 0".into());
    }
    ensure_worker();
    let s = sync();
    let now = Instant::now();
    let next = now + Duration::from_millis(ms);
    let mut st = s
        .state
        .lock()
        .map_err(|_| "scatter lock poisoned".to_string())?;
    st.schedule.insert(
        id.to_string(),
        Job {
            kind: JobKind::Every { interval_ms: ms },
            next,
        },
    );
    st.heap.push((Reverse(next), id.to_string()));
    drop(st);
    s.notify();
    Ok(())
}

/// Remove a scheduled job if present, and drop any queued occurrences of `id` in the due queue.
pub fn cancel(id: &str) -> bool {
    let Some(s) = SYNC.get() else {
        return false;
    };
    let Ok(mut st) = s.state.lock() else {
        return false;
    };
    let removed_schedule = st.schedule.remove(id).is_some();
    let dq_before = st.due_queue.len();
    st.due_queue.retain(|x| x != id);
    let changed = removed_schedule || dq_before != st.due_queue.len();
    drop(st);
    if changed {
        s.notify();
    }
    changed
}

/// Number of jobs still in the schedule (not including ids waiting in the due queue).
pub fn scheduled_count() -> usize {
    SYNC.get()
        .and_then(|s| s.state.lock().ok())
        .map(|g| g.schedule.len())
        .unwrap_or(0)
}

/// Drain all due job ids from the due queue (FIFO).
pub fn pending() -> Vec<String> {
    let Some(s) = SYNC.get() else {
        return Vec::new();
    };
    let Ok(mut st) = s.state.lock() else {
        return Vec::new();
    };
    st.due_queue.drain(..).collect()
}

/// Copy due ids without draining (debug / introspection). Prefer [`pending`] for real dispatch.
pub fn peek_pending() -> Vec<String> {
    let Some(s) = SYNC.get() else {
        return Vec::new();
    };
    let Ok(st) = s.state.lock() else {
        return Vec::new();
    };
    st.due_queue.iter().cloned().collect()
}

/// Milliseconds until the next heap fire, if any (uses min-heap peek).
pub fn next_due_ms() -> Option<u64> {
    let s = SYNC.get()?;
    let st = s.state.lock().ok()?;
    let now = Instant::now();
    let t = st.heap.peek().map(|(Reverse(t), _)| *t)?;
    if t <= now {
        return Some(0);
    }
    Some(t.duration_since(now).as_millis() as u64)
}

#[cfg(test)]
static SCATTER_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    SCATTER_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
pub(crate) fn reset_for_test() {
    if let Some(s) = SYNC.get() {
        if let Ok(mut st) = s.state.lock() {
            st.schedule.clear();
            st.due_queue.clear();
            st.heap.clear();
        }
        s.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn uniq(prefix: &str) -> String {
        format!(
            "{}_{}",
            prefix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }

    #[test]
    fn after_ms_fires_once() {
        let _lock = test_lock();
        reset_for_test();
        let id = uniq("after");
        after_ms(80, &id).unwrap();
        thread::sleep(Duration::from_millis(250));
        let p = pending();
        assert!(p.contains(&id), "pending={p:?}");
        assert!(!cancel(&id));
    }

    #[test]
    fn every_ms_repeats() {
        let _lock = test_lock();
        reset_for_test();
        let id = uniq("every");
        every_ms(120, &id).unwrap();
        thread::sleep(Duration::from_millis(380));
        let p = pending();
        assert!(p.iter().filter(|x| *x == &id).count() >= 1, "pending={p:?}");
        cancel(&id);
    }

    #[test]
    fn peek_pending_does_not_drain() {
        let _lock = test_lock();
        reset_for_test();
        let id = uniq("peek");
        after_ms(30, &id).unwrap();
        thread::sleep(Duration::from_millis(200));
        let a = peek_pending();
        assert!(a.contains(&id), "peek={a:?}");
        let b = pending();
        assert!(b.contains(&id));
        assert!(peek_pending().is_empty());
    }

    #[test]
    fn after_at_unix_ms_one_shot() {
        let _lock = test_lock();
        reset_for_test();
        let id = uniq("abs");
        let t = crate::stdlib::time::unix_ms_now() + 150;
        after_at_unix_ms(t, &id).unwrap();
        thread::sleep(Duration::from_millis(400));
        let p = pending();
        assert!(p.contains(&id), "pending={p:?}");
    }
}

/// Convert pending list to DAL `Value::Array` of strings.
pub fn pending_value() -> Value {
    let v = pending();
    Value::Array(v.into_iter().map(Value::String).collect())
}

/// Non-draining pending snapshot for DAL (debug only).
pub fn peek_pending_value() -> Value {
    let v = peek_pending();
    Value::Array(v.into_iter().map(Value::String).collect())
}
