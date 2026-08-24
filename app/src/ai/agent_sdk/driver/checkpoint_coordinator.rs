//! Periodic workspace-handoff checkpoint coordinator.
//!
//! Drives a five-state machine: `Idle -> Due -> InFlight -> Idle` on the periodic path,
//! with `Finalizing -> Stopped` reachable from `Idle`, `Due`, or `InFlight` via
//! [`CheckpointCoordinatorHandle::finalize`]. `Idle` waits out the jittered interval,
//! `Due` waits for a safe boundary, and `InFlight` runs exactly one attempt at a time.
//! The timer only ever moves `Idle` to `Due`; all gather/upload/commit work happens
//! through `super::snapshot::run_checkpoint_from_declarations_file`, reusing the same
//! declarations file and gather/upload pipeline as the legacy end-of-run snapshot.
//!
//! Safe-boundary gating ("only touch the filesystem/network when the conversation
//! isn't mid-turn") is implemented as a bounded poll of `AgentDriver`'s own state via
//! its `ModelSpawner`, rather than a push subscription: `AgentDriver` already reads
//! exactly the state needed (`run_conversation_id`, the terminal view's action model)
//! through this same read-only, spawner-based pattern used by `run_snapshot_upload`.
//! This trades a small amount of latency (up to [`SAFE_BOUNDARY_POLL_INTERVAL`]) for
//! avoiding new push-subscription wiring through the UI model graph.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::FutureExt as _;
use futures::future::BoxFuture;
use instant::Instant;
use rand::Rng as _;
use tokio::sync::{mpsc, oneshot};
use warpui::r#async::executor::Background;
use warpui::r#async::{FutureExt as _, Timer};
use warpui::{ModelSpawner, SingletonEntity};

use super::AgentDriver;
use super::snapshot::{self, CheckpointResult, DeclarationsWriterHandle};
use crate::ai::ambient_agents::AmbientAgentTaskId;
use crate::ai::blocklist::BlocklistAIHistoryModel;
use crate::server::server_api::harness_support::{CheckpointGeneration, HarnessSupportClient};

/// Whether the conversation can tolerate a checkpoint attempt right now.
///
/// `DriverGone` is deliberately distinct from `Busy`: a dropped `AgentDriver` used to be
/// reported as "safe", which left the coordinator gathering and uploading the whole
/// workspace every interval forever, since nothing else ever stops the loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Boundary {
    /// The run has not established its conversation yet; do not checkpoint setup work.
    Initializing,
    /// Not mid-turn: safe to touch the filesystem and network.
    Safe,
    /// Mid-turn or actions in flight; retry after [`SAFE_BOUNDARY_POLL_INTERVAL`].
    Busy,
    /// The `AgentDriver` this coordinator serves no longer exists, so there is nothing
    /// left to checkpoint for and the loop must stop.
    DriverGone,
}

/// A safe-boundary predicate, decoupled from `ModelSpawner<AgentDriver>` so
/// [`coordinator_loop`] can be exercised in isolation by tests. Production code builds
/// this from [`is_safe_boundary`]; tests supply a directly-controllable closure.
type BoundaryCheck = Arc<dyn Fn() -> BoxFuture<'static, Boundary> + Send + Sync>;

/// Default cadence between the end of one checkpoint attempt and the timer firing again,
/// absent an override on `AgentDriverOptions`. Deliberately much coarser than
/// `HARNESS_SAVE_INTERVAL` (30s): each attempt gathers and uploads the whole workspace,
/// so it is priced as minutes-scale background work rather than a lightweight save.
///
/// Note this is measured from attempt *completion*, not attempt start: the timer is only
/// restarted once the `InFlight` state resolves, so back-to-back attempts can never
/// overlap.
pub(super) const DEFAULT_CHECKPOINT_INTERVAL: Duration = Duration::from_secs(5 * 60);
/// Upper bound on additive jitter, so agents scheduled at the same time don't all
/// checkpoint in lockstep.
const CHECKPOINT_JITTER: Duration = Duration::from_secs(30);
/// How often the `Due` state re-checks whether the conversation is at a safe boundary.
const SAFE_BOUNDARY_POLL_INTERVAL: Duration = Duration::from_secs(2);
/// How long `Due` will wait for a safe boundary before checkpointing anyway.
///
/// Without a cap, a single long turn (or a conversation parked in a state the predicate
/// never calls safe) starves the feature entirely -- exactly the long-running case periodic
/// checkpoints exist for. A slightly-inconsistent checkpoint is strictly better than none,
/// since the previous committed generation is only replaced on success.
const MAX_BOUNDARY_DEFERRAL: Duration = Duration::from_secs(10 * 60);
/// Slack added on top of the per-attempt floor by [`finalize_budget`], so the coordinator has
/// time to start the final attempt before its deadline.
const FINALIZE_ACK_SLACK: Duration = Duration::from_secs(10);
/// Maximum time to wait for queued declaration writes before an attempt is abandoned.
const DECLARATIONS_WRITER_FLUSH_TIMEOUT: Duration = Duration::from_secs(10);

/// The shutdown budget a caller must grant [`CheckpointCoordinatorHandle::finalize`] for a
/// final attempt to be possible.
///
/// Exposed so `AgentDriver` cannot drift out of sync with the floor enforced in
/// [`finalize_with_new_attempt`]. The writer flush is part of the floor whenever a writer is
/// present, and is included unconditionally here because the caller does not need to know
/// whether the coordinator has one. Passing anything at or below the full floor silently skips
/// the final attempt -- and because the coordinator owns the whole end-of-run path, that means
/// no end-of-run snapshot at all.
pub(super) fn finalize_budget(script_timeout: Duration, upload_timeout: Duration) -> Duration {
    DECLARATIONS_WRITER_FLUSH_TIMEOUT + script_timeout + upload_timeout + FINALIZE_ACK_SLACK
}

/// A request to finalize, carrying the deadline used to decide whether a final attempt can start.
struct FinalizeRequest {
    deadline: Instant,
    ack: oneshot::Sender<()>,
}

/// Handle used by `AgentDriver` to request finalization of the periodic checkpoint
/// coordinator. Cloneable and fire-and-forget: dropping every handle without calling
/// [`finalize`](Self::finalize) simply leaves the coordinator running periodic
/// attempts until the process exits.
#[derive(Clone)]
pub(super) struct CheckpointCoordinatorHandle {
    finalize_tx: mpsc::UnboundedSender<FinalizeRequest>,
}

impl CheckpointCoordinatorHandle {
    /// Spawn the coordinator task on `background` and return a handle to it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        client: Arc<dyn HarnessSupportClient>,
        task_id: AmbientAgentTaskId,
        working_dir: PathBuf,
        declarations_writer: Option<DeclarationsWriterHandle>,
        spawner: ModelSpawner<AgentDriver>,
        interval: Duration,
        script_timeout: Duration,
        upload_timeout: Duration,
        background: Arc<Background>,
    ) -> Self {
        let boundary_check: BoundaryCheck = Arc::new(move || {
            let spawner = spawner.clone();
            Box::pin(async move { is_safe_boundary(&spawner).await })
        });
        Self::spawn_with_boundary_check(
            client,
            task_id,
            working_dir,
            declarations_writer,
            boundary_check,
            interval,
            CHECKPOINT_JITTER,
            script_timeout,
            upload_timeout,
            background,
        )
    }

    /// Test-facing constructor that bypasses `ModelSpawner<AgentDriver>` (and so the full
    /// UI framework) by taking the safe-boundary predicate directly, and disables jitter
    /// (production jitter is bounded by [`CHECKPOINT_JITTER`], up to 30s, which would
    /// otherwise make tests using a short `interval` flaky/slow).
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    fn new_for_test(
        client: Arc<dyn HarnessSupportClient>,
        task_id: AmbientAgentTaskId,
        working_dir: PathBuf,
        boundary_check: BoundaryCheck,
        interval: Duration,
        script_timeout: Duration,
        upload_timeout: Duration,
        background: Arc<Background>,
    ) -> Self {
        Self::spawn_with_boundary_check(
            client,
            task_id,
            working_dir,
            None,
            boundary_check,
            interval,
            Duration::ZERO,
            script_timeout,
            upload_timeout,
            background,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_with_boundary_check(
        client: Arc<dyn HarnessSupportClient>,
        task_id: AmbientAgentTaskId,
        working_dir: PathBuf,
        declarations_writer: Option<DeclarationsWriterHandle>,
        boundary_check: BoundaryCheck,
        interval: Duration,
        jitter: Duration,
        script_timeout: Duration,
        upload_timeout: Duration,
        background: Arc<Background>,
    ) -> Self {
        let (finalize_tx, finalize_rx) = mpsc::unbounded_channel();
        // Keep the executor alive for as long as the detached coordinator task is alive. The
        // coordinator owns its in-flight attempt inline, but its scheduler still runs on this
        // background executor after the constructor returns.
        let loop_background = background.clone();
        background
            .spawn(coordinator_loop(
                client,
                task_id,
                working_dir,
                declarations_writer,
                boundary_check,
                interval,
                jitter,
                script_timeout,
                upload_timeout,
                loop_background,
                finalize_rx,
            ))
            .detach();
        Self { finalize_tx }
    }

    /// Request finalization: run at most one more checkpoint attempt if none is
    /// already in flight (skipped if `budget` doesn't exceed the writer/gather/upload
    /// floor), or await an already-in-flight attempt instead -- never both -- then
    /// stop the coordinator. `budget` is a hard upper bound for waiting on persistence;
    /// an attempt that outlives it is cancelled before the ack is sent. Safe to call at most
    /// once; safe to never call.
    ///
    /// Callers should derive `budget` from [`finalize_budget`] rather than passing a
    /// per-attempt timeout directly: the floor in [`finalize_with_new_attempt`] includes the
    /// declaration-writer flush, script, and upload timeouts, so a smaller budget silently
    /// skips the final attempt.
    pub(super) async fn finalize(&self, budget: Duration) {
        let (ack_tx, ack_rx) = oneshot::channel();
        let request = FinalizeRequest {
            deadline: Instant::now() + budget,
            ack: ack_tx,
        };
        if self.finalize_tx.send(request).is_err() {
            // Coordinator task already exited; nothing to wait for.
            return;
        }
        // Keep the caller bounded even if the coordinator task encounters an unexpected stall.
        // A timed-out ack receiver is harmless: the coordinator owns no additional shutdown
        // resources once its own finalization deadline has elapsed.
        let ack_timeout = budget.saturating_add(FINALIZE_ACK_SLACK);
        if tokio::time::timeout(ack_timeout, ack_rx).await.is_err() {
            log::warn!(
                "Timed out waiting for checkpoint coordinator finalization ack after {ack_timeout:?}"
            );
        }
    }
}

/// Add up to `jitter` of additive random delay to `interval` so many agents scheduled at once
/// don't checkpoint in lockstep. Production always passes [`CHECKPOINT_JITTER`]; tests pass
/// `Duration::ZERO` for determinism.
fn jittered_interval(interval: Duration, jitter: Duration) -> Duration {
    let jitter_ms = u64::try_from(jitter.as_millis()).unwrap_or(u64::MAX);
    let extra = if jitter_ms == 0 {
        0
    } else {
        rand::thread_rng().gen_range(0..=jitter_ms)
    };
    interval + Duration::from_millis(extra)
}

/// Run one checkpoint attempt to completion: drain pending declarations writes,
/// regenerate declarations, then gather, upload, and commit. Each external step is bounded by
/// its corresponding timeout; the finalization path adds an outer hard deadline.
///
/// `generation` carries the previous attempt's generation when this attempt is a retry, so
/// the re-gathered payload overwrites that attempt's staged objects instead of adding a new
/// set. See [`snapshot::run_checkpoint_from_declarations_file`].
#[allow(clippy::too_many_arguments)]
async fn run_one_attempt(
    client: Arc<dyn HarnessSupportClient>,
    task_id: AmbientAgentTaskId,
    working_dir: PathBuf,
    declarations_writer: Option<DeclarationsWriterHandle>,
    script_timeout: Duration,
    upload_timeout: Duration,
    generation: Option<CheckpointGeneration>,
) -> CheckpointResult {
    // Allocate before any bounded step so a retry can overwrite objects staged by an attempt
    // that is cancelled during shutdown.
    let generation = generation.unwrap_or_else(snapshot::mint_generation);

    // Drain queued driver-side `file` appends before the bash script starts appending its
    // own `repo` entries to the same append-only JSONL. `AgentDriver::run_snapshot_upload`
    // does this on the legacy path for exactly this reason; without it a checkpoint can
    // both miss the agent's most recent edits and race the script on the shared file.
    if let Some(writer) = &declarations_writer
        && writer
            .flush()
            .with_timeout(DECLARATIONS_WRITER_FLUSH_TIMEOUT)
            .await
            .is_err()
    {
        let reason = format!(
            "checkpoint attempt exceeded {DECLARATIONS_WRITER_FLUSH_TIMEOUT:?} declarations writer flush timeout"
        );
        log::warn!("{reason}");
        return CheckpointResult::Failed {
            generation: Some(generation),
            reason,
        };
    }
    snapshot::run_declarations_script(&working_dir, &task_id, script_timeout).await;
    let path = snapshot::resolve_declarations_path(Some(&task_id));
    match snapshot::run_checkpoint_from_declarations_file(&path, client, Some(generation.clone()))
        .with_timeout(upload_timeout)
        .await
    {
        Ok(result) => result,
        Err(_) => CheckpointResult::Failed {
            generation: Some(generation),
            reason: format!("checkpoint attempt exceeded {upload_timeout:?} upload timeout"),
        },
    }
}

/// Query `AgentDriver` (via its spawner) for whether the conversation is currently at
/// a safe boundary.
///
/// [`Boundary::Initializing`] while a fresh run is still establishing its conversation. This
/// state is kept separate from [`Boundary::Busy`] so the maximum boundary deferral cannot force
/// a checkpoint while setup commands, environment cloning, or MCP startup are still mutating
/// the workspace.
///
/// [`Boundary::Safe`] when the conversation can no longer be found or is quiescent -- there is
/// nothing to interrupt.
/// [`Boundary::DriverGone`] when the driver itself has been dropped, which stops the loop
/// rather than being conflated with "safe" and checkpointing forever.
///
/// `InProgress`/`TransientError` do not immediately imply `Busy`: for most of a turn the
/// agent is waiting on the model's response rather than touching the filesystem, and only
/// actually executing an action (a pending or running entry in the terminal's action model)
/// risks a concurrent mutation. Treating the whole status as `Busy` would make the safe-
/// boundary check nearly useless for a continuously active agent, since it would almost
/// never see anything but `InProgress` before `MAX_BOUNDARY_DEFERRAL` forces a checkpoint
/// anyway.
async fn is_safe_boundary(spawner: &ModelSpawner<AgentDriver>) -> Boundary {
    let result = spawner
        .spawn(|driver, ctx| {
            if !driver.checkpoint_ready {
                return Boundary::Initializing;
            }
            let Some(conversation_id) = driver.run_conversation_id else {
                // A fresh run opens the checkpoint gate just before dispatching its first
                // prompt, but the history event carrying its conversation ID arrives later.
                // Keep that short hand-off window in setup so the initial turn cannot race a
                // checkpoint even though all other harness preparation has completed.
                return Boundary::Initializing;
            };
            let Some(status) = BlocklistAIHistoryModel::as_ref(ctx)
                .conversation(&conversation_id)
                .map(|conversation| conversation.status().clone())
            else {
                return Boundary::Safe;
            };
            // Quiescent states, checked before the pending-action sweep below.
            //
            // `Blocked` in particular *is* backed by a pending action, so letting it fall
            // through would report `Busy` forever: a run parked on user approval (often for
            // hours) would poll every couple of seconds and never checkpoint, even though
            // nothing is mutating the workspace. That is precisely when a checkpoint is
            // most valuable.
            if status.is_waiting_for_events() || status.is_blocked() || status.is_done() {
                return Boundary::Safe;
            }
            // `InProgress` (running a turn) and `TransientError` (a failed turn about to be
            // retried) both cover time spent waiting on the model as well as time spent
            // executing actions, so fall through to the action model rather than treating
            // either as unconditionally busy.
            let terminal_view = driver
                .terminal_driver
                .as_ref(ctx)
                .terminal_view()
                .as_ref(ctx);
            if terminal_view
                .ai_action_model()
                .as_ref(ctx)
                .has_unfinished_actions_for_conversation(conversation_id)
            {
                Boundary::Busy
            } else {
                Boundary::Safe
            }
        })
        .await;
    result.unwrap_or(Boundary::DriverGone)
}

/// How the `Due` state resolved.
enum DueOutcome {
    /// Proceed to `InFlight`.
    Safe,
    /// A finalize request arrived while waiting; the caller owns it.
    Finalize(FinalizeRequest),
    /// The coordinator should stop: the driver is gone, or every handle was dropped.
    Stop,
}

/// Poll for a safe boundary, staying responsive to finalize throughout.
///
/// The boundary check is itself awaited inside `select!` rather than before it: it is a
/// round trip through `AgentDriver`'s model task queue, and a stalled queue must not be able
/// to wedge shutdown behind an unbounded await.
async fn wait_for_safe_boundary(
    boundary_check: &BoundaryCheck,
    finalize_rx: &mut mpsc::UnboundedReceiver<FinalizeRequest>,
) -> DueOutcome {
    let mut due_since = Instant::now();
    loop {
        // Checked immediately on entry (not only after the first poll interval elapses) so
        // an already-safe conversation doesn't pay needless latency.
        let boundary = futures::select! {
            boundary = boundary_check().fuse() => boundary,
            request = finalize_rx.recv().fuse() => {
                return request.map_or(DueOutcome::Stop, DueOutcome::Finalize);
            }
        };
        match boundary {
            Boundary::Safe => return DueOutcome::Safe,
            Boundary::DriverGone => {
                log::info!("AgentDriver is gone; stopping the periodic checkpoint coordinator");
                return DueOutcome::Stop;
            }
            Boundary::Initializing => {
                // Do not carry setup time into the post-conversation busy window. Otherwise a
                // long environment clone could make the first active turn immediately exceed
                // MAX_BOUNDARY_DEFERRAL and force an unsafe checkpoint.
                due_since = Instant::now();
            }
            Boundary::Busy => {}
        }
        if matches!(boundary, Boundary::Busy) && due_since.elapsed() >= MAX_BOUNDARY_DEFERRAL {
            log::warn!(
                "Conversation has not reached a safe boundary in {MAX_BOUNDARY_DEFERRAL:?}; \
                 checkpointing anyway rather than skipping this cycle entirely"
            );
            return DueOutcome::Safe;
        }
        futures::select! {
            _ = Timer::after(SAFE_BOUNDARY_POLL_INTERVAL).fuse() => {}
            request = finalize_rx.recv().fuse() => {
                return request.map_or(DueOutcome::Stop, DueOutcome::Finalize);
            }
        }
    }
}

/// Handle a finalize request received while no attempt is currently in flight: start
/// exactly one best-effort attempt only if `budget` exceeds the full writer/gather/upload floor.
/// The deadline is a hard upper bound; an attempt that outlives it is cancelled and logged.
#[allow(clippy::too_many_arguments)]
async fn finalize_with_new_attempt(
    request: FinalizeRequest,
    client: Arc<dyn HarnessSupportClient>,
    task_id: AmbientAgentTaskId,
    working_dir: PathBuf,
    declarations_writer: Option<DeclarationsWriterHandle>,
    boundary_check: &BoundaryCheck,
    script_timeout: Duration,
    upload_timeout: Duration,
    generation: Option<CheckpointGeneration>,
) {
    let boundary_remaining = request.deadline.saturating_duration_since(Instant::now());
    let boundary = if boundary_remaining.is_zero() {
        None
    } else {
        tokio::time::timeout(boundary_remaining, boundary_check())
            .await
            .ok()
    };
    if matches!(boundary, Some(Boundary::DriverGone) | None) {
        log::info!(
            "Skipping final checkpoint attempt because the run did not reach a checkpoint-safe boundary"
        );
        let _ = request.ack.send(());
        return;
    }

    // This is teardown-only: `AgentDriver::run_snapshot_upload` calls `finalize` after
    // `run_internal` has resolved or been cancelled, so setup can no longer mutate the
    // workspace. Preserve the legacy end-of-run snapshot even when setup itself failed before
    // the normal checkpoint gate opened.

    let floor = script_timeout
        + upload_timeout
        + declarations_writer
            .as_ref()
            .map_or(Duration::ZERO, |_| DECLARATIONS_WRITER_FLUSH_TIMEOUT);
    let remaining = request.deadline.saturating_duration_since(Instant::now());
    if remaining > floor {
        log::info!(
            "Starting final checkpoint attempt at shutdown (remaining budget {remaining:?})"
        );
        let attempt = run_one_attempt(
            client,
            task_id,
            working_dir,
            declarations_writer,
            script_timeout,
            upload_timeout,
            generation,
        );
        match tokio::time::timeout(remaining, attempt).await {
            Ok(result) => log::info!("Final checkpoint attempt resolved: {result:?}"),
            Err(_) => {
                log::warn!(
                    "Final checkpoint attempt did not complete within {remaining:?}; \
                     cancelling it at the shutdown deadline"
                );
            }
        }
    } else {
        // Callers must size the budget with `finalize_budget`; anything at or below the
        // full floor lands here and produces no end-of-run checkpoint at all.
        log::warn!(
            "Skipping final checkpoint attempt: remaining shutdown budget {remaining:?} \
             is below the {floor:?} floor"
        );
    }
    let _ = request.ack.send(());
}

/// Wait for an in-flight attempt during finalization. The attempt is owned by the coordinator,
/// so dropping it after the deadline cancels pending gather/upload/commit work before the caller
/// is acknowledged.
async fn finalize_in_flight_attempt_owned<F>(
    request: FinalizeRequest,
    attempt: Pin<&mut F>,
) -> oneshot::Sender<()>
where
    F: std::future::Future<Output = CheckpointResult> + ?Sized,
{
    let remaining = request.deadline.saturating_duration_since(Instant::now());
    match tokio::time::timeout(remaining, attempt).await {
        Ok(result) => {
            log::info!("In-flight checkpoint attempt resolved during finalization: {result:?}");
        }
        Err(_) => {
            log::warn!(
                "In-flight checkpoint attempt did not resolve within the remaining \
                 {remaining:?} shutdown budget; cancelling it before finalization ack"
            );
        }
    }
    request.ack
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn start_attempt(
    client: Arc<dyn HarnessSupportClient>,
    task_id: AmbientAgentTaskId,
    working_dir: PathBuf,
    declarations_writer: Option<DeclarationsWriterHandle>,
    script_timeout: Duration,
    upload_timeout: Duration,
    generation: Option<CheckpointGeneration>,
    background: &Background,
) -> oneshot::Receiver<CheckpointResult> {
    let (tx, rx) = oneshot::channel();
    background
        .spawn(async move {
            let result = run_one_attempt(
                client,
                task_id,
                working_dir,
                declarations_writer,
                script_timeout,
                upload_timeout,
                generation,
            )
            .await;
            let _ = tx.send(result);
        })
        .detach();
    rx
}

#[cfg(test)]
async fn finalize_with_in_flight_attempt(
    request: FinalizeRequest,
    mut result_rx: oneshot::Receiver<CheckpointResult>,
) {
    let remaining = request.deadline.saturating_duration_since(Instant::now());
    match tokio::time::timeout(remaining, &mut result_rx).await {
        Ok(Ok(result)) => {
            log::info!("In-flight checkpoint attempt resolved during finalization: {result:?}");
        }
        Ok(Err(_)) => {
            log::warn!("In-flight checkpoint attempt's result channel dropped without a result");
        }
        Err(_) => {
            log::warn!(
                "In-flight checkpoint attempt did not resolve within the remaining \
                 {remaining:?} shutdown budget; releasing the coordinator without waiting"
            );
        }
    }
    let _ = request.ack.send(());
}

/// The coordinator's main loop. `Idle` and `Due` are collapsed into the top of the
/// loop body: the timer is the only thing that ever moves `Idle` to `Due`, and `Due`
/// then polls the safe-boundary predicate. `InFlight` owns the attempt future so finalization can
/// either await persistence or cancel it before acknowledging shutdown.
#[allow(clippy::too_many_arguments)]
async fn coordinator_loop(
    client: Arc<dyn HarnessSupportClient>,
    task_id: AmbientAgentTaskId,
    working_dir: PathBuf,
    declarations_writer: Option<DeclarationsWriterHandle>,
    boundary_check: BoundaryCheck,
    interval: Duration,
    jitter: Duration,
    script_timeout: Duration,
    upload_timeout: Duration,
    _background: Arc<Background>,
    mut finalize_rx: mpsc::UnboundedReceiver<FinalizeRequest>,
) {
    // Generation of the last attempt that failed, if any. Retrying under it overwrites that
    // attempt's staged objects; minting per attempt would instead pile up a new set each time
    // and can exhaust the server's per-execution staging budget. Cleared once an attempt
    // commits (its objects are the committed checkpoint) or skips (nothing was staged).
    let mut pending_generation: Option<CheckpointGeneration> = None;
    loop {
        // --- Idle: wait for the next (jittered) tick or a finalize request. ---
        futures::select! {
            _ = Timer::after(jittered_interval(interval, jitter)).fuse() => {}
            request = finalize_rx.recv().fuse() => {
                let Some(request) = request else { return };
                finalize_with_new_attempt(
                    request,
                    client.clone(),
                    task_id,
                    working_dir.clone(),
                    declarations_writer.clone(),
                    &boundary_check,
                    script_timeout,
                    upload_timeout,
                    pending_generation,
                )
                .await;
                return;
            }
        }

        // --- Due: poll for a safe boundary, staying responsive to finalize. ---
        match wait_for_safe_boundary(&boundary_check, &mut finalize_rx).await {
            DueOutcome::Safe => {}
            DueOutcome::Finalize(request) => {
                finalize_with_new_attempt(
                    request,
                    client.clone(),
                    task_id,
                    working_dir.clone(),
                    declarations_writer.clone(),
                    &boundary_check,
                    script_timeout,
                    upload_timeout,
                    pending_generation,
                )
                .await;
                return;
            }
            DueOutcome::Stop => return,
        }

        // --- InFlight: run exactly one attempt, never overlapping another. ---
        let mut attempt = Box::pin(run_one_attempt(
            client.clone(),
            task_id,
            working_dir.clone(),
            declarations_writer.clone(),
            script_timeout,
            upload_timeout,
            pending_generation.clone(),
        ));
        futures::select! {
            result = attempt.as_mut().fuse() => {
                match result {
                    CheckpointResult::Committed { generation } => {
                        log::info!(
                            "Periodic checkpoint committed: generation={}",
                            generation.as_str()
                        );
                        pending_generation = None;
                    }
                    CheckpointResult::Skipped => {
                        log::info!("Periodic checkpoint skipped: no usable declarations");
                        pending_generation = None;
                    }
                    CheckpointResult::Failed { generation, reason } => {
                        log::warn!(
                            "Periodic checkpoint attempt failed (generation={:?}): {reason}",
                            generation.as_ref().map(CheckpointGeneration::as_str)
                        );
                        // Retry under the same generation so the next attempt overwrites
                        // whatever this one staged instead of staging a second set.
                        pending_generation = generation;
                    }
                }
                // Success, skip, or failure: return to Idle and wait a full interval
                // before the next attempt either way. The periodic timer itself
                // (rather than a distinct short backoff) is the retry mechanism for
                // failures too: checkpoints are best-effort with no recovery-point or
                // recovery-time guarantee, so retrying sooner isn't worth the extra
                // whole-workspace gather and upload.
            }
            request = finalize_rx.recv().fuse() => {
                let Some(request) = request else { return };
                let ack = finalize_in_flight_attempt_owned(request, attempt.as_mut()).await;
                // `attempt` is an inline future, not a detached task. Drop it before sending the
                // ack so no gather/upload/commit can complete after finalization is acknowledged.
                drop(attempt);
                let _ = ack.send(());
                return;
            }
        }
    }
}

#[cfg(test)]
#[path = "checkpoint_coordinator_tests.rs"]
mod tests;
