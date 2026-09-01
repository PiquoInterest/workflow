use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, ThreadId};

pub const MAX_MEMOIZED_PRIMITIVE_UTF16_LENGTH: usize = 4096;
pub const DEFAULT_PREWARM_CONCURRENCY: usize = 8;
pub const PAYLOAD_CONFLICT_MESSAGE: &str =
    "replay payload cache key was reused with different binary data";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayValue {
    Undefined,
    Null,
    Boolean(bool),
    Integer(i64),
    String(String),
    Bytes(Vec<u8>),
    Object(Vec<(String, ReplayValue)>),
}

impl ReplayValue {
    fn is_memoizable_primitive(&self) -> bool {
        match self {
            Self::Undefined | Self::Null | Self::Boolean(_) | Self::Integer(_) => true,
            Self::String(value) => {
                value.encode_utf16().count() <= MAX_MEMOIZED_PRIMITIVE_UTF16_LENGTH
            }
            Self::Bytes(_) | Self::Object(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayPayload {
    Binary(Arc<[u8]>),
    Legacy(ReplayValue),
}

impl ReplayPayload {
    pub fn binary(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Binary(Arc::<[u8]>::from(bytes.into()))
    }

    pub fn legacy(value: ReplayValue) -> Self {
        Self::Legacy(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedReplayPayload {
    pub value: ReplayValue,
}

impl PreparedReplayPayload {
    pub fn from_input(input: &ReplayPayload) -> Self {
        let value = match input {
            ReplayPayload::Binary(bytes) => ReplayValue::Bytes(bytes.to_vec()),
            ReplayPayload::Legacy(value) => value.clone(),
        };
        Self { value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayCacheErrorKind {
    Preparation,
    PreparationPanicked,
    ReentrantPreparation,
    PayloadConflict,
    Hydration,
    HydrationPanicked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCacheError {
    pub kind: ReplayCacheErrorKind,
    pub message: String,
}

impl ReplayCacheError {
    pub fn preparation(message: impl Into<String>) -> Self {
        Self {
            kind: ReplayCacheErrorKind::Preparation,
            message: message.into(),
        }
    }

    pub fn hydration(message: impl Into<String>) -> Self {
        Self {
            kind: ReplayCacheErrorKind::Hydration,
            message: message.into(),
        }
    }

    fn preparation_panicked() -> Self {
        Self {
            kind: ReplayCacheErrorKind::PreparationPanicked,
            message: "replay payload preparation panicked".to_owned(),
        }
    }

    fn reentrant_preparation() -> Self {
        Self {
            kind: ReplayCacheErrorKind::ReentrantPreparation,
            message: "replay payload preparation re-entered the same cache key".to_owned(),
        }
    }

    fn hydration_panicked() -> Self {
        Self {
            kind: ReplayCacheErrorKind::HydrationPanicked,
            message: "replay payload hydration panicked".to_owned(),
        }
    }

    fn payload_conflict() -> Self {
        Self {
            kind: ReplayCacheErrorKind::PayloadConflict,
            message: PAYLOAD_CONFLICT_MESSAGE.to_owned(),
        }
    }
}

impl Display for ReplayCacheError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ReplayCacheError {}

pub trait ReplayPayloadPreparer: Send + Sync + 'static {
    fn prepare(&self, input: &ReplayPayload) -> Result<PreparedReplayPayload, ReplayCacheError>;
}

impl<F> ReplayPayloadPreparer for F
where
    F: Fn(&ReplayPayload) -> Result<PreparedReplayPayload, ReplayCacheError>
        + Send
        + Sync
        + 'static,
{
    fn prepare(&self, input: &ReplayPayload) -> Result<PreparedReplayPayload, ReplayCacheError> {
        self(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayPayloadField {
    Result,
    Error,
    Payload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEventType {
    StepCompleted,
    StepFailed,
    HookReceived,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayEvent {
    pub event_id: String,
    pub event_type: ReplayEventType,
    pub payload: Option<ReplayPayload>,
}

impl ReplayEvent {
    pub fn new(
        event_id: impl Into<String>,
        event_type: ReplayEventType,
        payload: Option<ReplayPayload>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type,
            payload,
        }
    }

    fn payload_field(&self) -> Option<ReplayPayloadField> {
        match self.event_type {
            ReplayEventType::StepCompleted => Some(ReplayPayloadField::Result),
            ReplayEventType::StepFailed => Some(ReplayPayloadField::Error),
            ReplayEventType::HookReceived => Some(ReplayPayloadField::Payload),
            ReplayEventType::Other => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunPayload {
    pub run_id: String,
    pub input: ReplayPayload,
}

impl WorkflowRunPayload {
    pub fn new(run_id: impl Into<String>, input: ReplayPayload) -> Self {
        Self {
            run_id: run_id.into(),
            input,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrewarmReport {
    pub discovered: usize,
    pub completed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CacheKey {
    WorkflowInput(String),
    Event {
        event_id: String,
        field: ReplayPayloadField,
    },
}

#[derive(Debug, Clone)]
enum PreparationState {
    Pending,
    Running(ThreadId),
    Ready(Result<Arc<PreparedReplayPayload>, ReplayCacheError>),
    Conflict(ReplayCacheError),
}

struct PreparationCell {
    original: Arc<[u8]>,
    state: Mutex<PreparationState>,
    changed: Condvar,
}

impl PreparationCell {
    fn new(original: Arc<[u8]>) -> Self {
        Self {
            original,
            state: Mutex::new(PreparationState::Pending),
            changed: Condvar::new(),
        }
    }

    fn mark_conflict(&self) -> ReplayCacheError {
        let mut state = lock_recover(&self.state);
        if let PreparationState::Conflict(error) = &*state {
            return error.clone();
        }
        let error = ReplayCacheError::payload_conflict();
        *state = PreparationState::Conflict(error.clone());
        self.changed.notify_all();
        error
    }

    fn prepare(
        &self,
        preparer: &dyn ReplayPayloadPreparer,
    ) -> Result<Arc<PreparedReplayPayload>, ReplayCacheError> {
        let current_thread = thread::current().id();
        loop {
            let mut state = lock_recover(&self.state);
            match &*state {
                PreparationState::Pending => {
                    *state = PreparationState::Running(current_thread);
                    drop(state);

                    let input = ReplayPayload::Binary(Arc::clone(&self.original));
                    let prepared = match catch_unwind(AssertUnwindSafe(|| preparer.prepare(&input))) {
                        Ok(result) => result.map(Arc::new),
                        Err(_) => Err(ReplayCacheError::preparation_panicked()),
                    };

                    let mut state = lock_recover(&self.state);
                    if let PreparationState::Conflict(error) = &*state {
                        let error = error.clone();
                        self.changed.notify_all();
                        return Err(error);
                    }
                    *state = PreparationState::Ready(prepared.clone());
                    self.changed.notify_all();
                    return prepared;
                }
                PreparationState::Running(owner) if owner == &current_thread => {
                    return Err(ReplayCacheError::reentrant_preparation());
                }
                PreparationState::Running(_) => {
                    state = wait_recover(&self.changed, state);
                    drop(state);
                }
                PreparationState::Ready(result) => return result.clone(),
                PreparationState::Conflict(error) => return Err(error.clone()),
            }
        }
    }
}

struct ReplayPayloadCacheInner {
    preparer: Arc<dyn ReplayPayloadPreparer>,
    prepared_payloads: Mutex<HashMap<CacheKey, Arc<PreparationCell>>>,
    primitive_step_results: Mutex<HashMap<String, ReplayValue>>,
    next_unscanned_event_index: Mutex<usize>,
    max_prewarm_concurrency: usize,
}

#[derive(Clone)]
pub struct ReplayPayloadCache {
    inner: Arc<ReplayPayloadCacheInner>,
}

impl ReplayPayloadCache {
    pub fn new<P>(preparer: P) -> Self
    where
        P: ReplayPayloadPreparer,
    {
        Self::with_max_prewarm_concurrency(preparer, DEFAULT_PREWARM_CONCURRENCY)
    }

    pub fn identity() -> Self {
        Self::new(|input: &ReplayPayload| Ok(PreparedReplayPayload::from_input(input)))
    }

    pub fn with_max_prewarm_concurrency<P>(preparer: P, max_prewarm_concurrency: usize) -> Self
    where
        P: ReplayPayloadPreparer,
    {
        Self {
            inner: Arc::new(ReplayPayloadCacheInner {
                preparer: Arc::new(preparer),
                prepared_payloads: Mutex::new(HashMap::new()),
                primitive_step_results: Mutex::new(HashMap::new()),
                next_unscanned_event_index: Mutex::new(0),
                max_prewarm_concurrency: max_prewarm_concurrency.max(1),
            }),
        }
    }

    pub fn prepare_workflow_input(
        &self,
        workflow_run: &WorkflowRunPayload,
    ) -> Result<Arc<PreparedReplayPayload>, ReplayCacheError> {
        self.consume(
            CacheKey::WorkflowInput(workflow_run.run_id.clone()),
            &workflow_run.input,
        )
    }

    pub fn prepare_event_payload(
        &self,
        event_id: &str,
        field: ReplayPayloadField,
        payload: &ReplayPayload,
    ) -> Result<Arc<PreparedReplayPayload>, ReplayCacheError> {
        self.consume(
            CacheKey::Event {
                event_id: event_id.to_owned(),
                field,
            },
            payload,
        )
    }

    pub fn prewarm(
        &self,
        workflow_run: &WorkflowRunPayload,
        events: &[ReplayEvent],
    ) -> PrewarmReport {
        let start_index = {
            let mut next = lock_recover(&self.inner.next_unscanned_event_index);
            let start = *next;
            *next = events.len();
            start
        };

        let mut tasks = Vec::new();
        let mut failed_discovery = 0;
        self.discover_binary(
            CacheKey::WorkflowInput(workflow_run.run_id.clone()),
            &workflow_run.input,
            &mut tasks,
            &mut failed_discovery,
        );

        for event in events.iter().skip(start_index) {
            let (Some(field), Some(payload)) = (event.payload_field(), event.payload.as_ref()) else {
                continue;
            };
            self.discover_binary(
                CacheKey::Event {
                    event_id: event.event_id.clone(),
                    field,
                },
                payload,
                &mut tasks,
                &mut failed_discovery,
            );
        }

        if tasks.is_empty() {
            return PrewarmReport {
                failed: failed_discovery,
                ..PrewarmReport::default()
            };
        }

        let next_task = AtomicUsize::new(0);
        let completed = AtomicUsize::new(0);
        let failed = AtomicUsize::new(0);
        let worker_count = self.inner.max_prewarm_concurrency.min(tasks.len());

        thread::scope(|scope| {
            for _ in 0..worker_count {
                let tasks = &tasks;
                let next_task = &next_task;
                let completed = &completed;
                let failed = &failed;
                let preparer = Arc::clone(&self.inner.preparer);
                scope.spawn(move || {
                    loop {
                        let index = next_task.fetch_add(1, Ordering::Relaxed);
                        let Some(cell) = tasks.get(index) else {
                            break;
                        };
                        if cell.prepare(preparer.as_ref()).is_ok() {
                            completed.fetch_add(1, Ordering::Relaxed);
                        } else {
                            failed.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });
            }
        });

        PrewarmReport {
            discovered: tasks.len(),
            completed: completed.load(Ordering::Relaxed),
            failed: failed_discovery + failed.load(Ordering::Relaxed),
        }
    }

    pub fn reset_scan(&self) {
        *lock_recover(&self.inner.next_unscanned_event_index) = 0;
    }

    pub fn get_step_result<F>(
        &self,
        event_id: &str,
        hydrate: F,
    ) -> Result<ReplayValue, ReplayCacheError>
    where
        F: FnOnce() -> Result<ReplayValue, ReplayCacheError>,
    {
        if let Some(value) = lock_recover(&self.inner.primitive_step_results)
            .get(event_id)
            .cloned()
        {
            return Ok(value);
        }

        let value = match catch_unwind(AssertUnwindSafe(hydrate)) {
            Ok(result) => result?,
            Err(_) => return Err(ReplayCacheError::hydration_panicked()),
        };
        if value.is_memoizable_primitive() {
            lock_recover(&self.inner.primitive_step_results)
                .insert(event_id.to_owned(), value.clone());
        }
        Ok(value)
    }

    fn consume(
        &self,
        key: CacheKey,
        payload: &ReplayPayload,
    ) -> Result<Arc<PreparedReplayPayload>, ReplayCacheError> {
        let ReplayPayload::Binary(bytes) = payload else {
            return self.run_preparer(payload).map(Arc::new);
        };

        let (cell, _) = self.ensure_binary_cell(key.clone(), Arc::clone(bytes))?;
        let result = cell.prepare(self.inner.preparer.as_ref());
        if let Err(error) = &result {
            if error.kind != ReplayCacheErrorKind::PayloadConflict {
                self.evict_observed_failure(&key, &cell, error);
            }
        }
        result
    }

    fn run_preparer(
        &self,
        payload: &ReplayPayload,
    ) -> Result<PreparedReplayPayload, ReplayCacheError> {
        match catch_unwind(AssertUnwindSafe(|| self.inner.preparer.prepare(payload))) {
            Ok(result) => result,
            Err(_) => Err(ReplayCacheError::preparation_panicked()),
        }
    }

    fn discover_binary(
        &self,
        key: CacheKey,
        payload: &ReplayPayload,
        tasks: &mut Vec<Arc<PreparationCell>>,
        failed: &mut usize,
    ) {
        let ReplayPayload::Binary(bytes) = payload else {
            return;
        };
        match self.ensure_binary_cell(key, Arc::clone(bytes)) {
            Ok((cell, true)) => tasks.push(cell),
            Ok((_, false)) => {}
            Err(_) => *failed += 1,
        }
    }

    fn ensure_binary_cell(
        &self,
        key: CacheKey,
        bytes: Arc<[u8]>,
    ) -> Result<(Arc<PreparationCell>, bool), ReplayCacheError> {
        let mut entries = lock_recover(&self.inner.prepared_payloads);
        if let Some(cell) = entries.get(&key).cloned() {
            let same_payload = cell.original.as_ref() == bytes.as_ref();
            drop(entries);
            if !same_payload {
                return Err(cell.mark_conflict());
            }
            return Ok((cell, false));
        }

        let cell = Arc::new(PreparationCell::new(bytes));
        entries.insert(key, Arc::clone(&cell));
        Ok((cell, true))
    }

    fn evict_observed_failure(
        &self,
        key: &CacheKey,
        cell: &Arc<PreparationCell>,
        expected_error: &ReplayCacheError,
    ) {
        let mut entries = lock_recover(&self.inner.prepared_payloads);
        let Some(cached) = entries.get(key) else {
            return;
        };
        if !Arc::ptr_eq(cached, cell) {
            return;
        }

        let state = lock_recover(&cell.state);
        let should_remove = matches!(
            &*state,
            PreparationState::Ready(Err(error)) if error == expected_error
        );
        drop(state);
        if should_remove {
            entries.remove(key);
        }
    }
}

fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn wait_recover<'a, T>(
    condition: &Condvar,
    guard: MutexGuard<'a, T>,
) -> MutexGuard<'a, T> {
    match condition.wait(guard) {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
