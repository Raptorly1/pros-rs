//! FreeRTOS task creation and management.
//!
//! Any method of creating a task will return a [`TaskHandle`].
//! This handle can be used to control the task.
//! A handle to the current task can be obtained with [`current`].
//!
//! Tasks can be created with the [`spawn`] function or, for more control, with a task [`Builder`].
//! ## Example
//! ```rust
//! # use pros::prelude::println;
//! use pros::task::{spawn, TaskPriority};
//! spawn(|| {
//!    println!("Hello from a task!");
//! });
//! ```
//!
//! Task locals can be created with the [`os_task_local!`](crate::os_task_local!) macro.
//! See the [`local`] module for more info on the custom task local implementation used.

pub mod local;

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::{ffi::CStr, future::Future, hash::Hash, str::Utf8Error, task::Poll, time::Duration};

use snafu::Snafu;

use crate::{
    async_runtime::executor::EXECUTOR,
    error::{bail_on, map_errno},
};

/// Creates a task to be run 'asynchronously' (More information at the [FreeRTOS docs](https://www.freertos.org/taskandcr.html)).
/// Takes in a closure that can move variables if needed.
/// If your task has a loop it is advised to use [`sleep(duration)`](sleep) so that the task does not take up necessary system resources.
/// Tasks should be long-living; starting many tasks can be slow and is usually not necessary.
pub fn spawn<F>(f: F) -> TaskHandle
where
    F: FnOnce() + Send + 'static,
{
    Builder::new().spawn(f).expect("Failed to spawn task")
}

/// Low level task spawning functionality
fn spawn_inner<F: FnOnce() + Send + 'static>(
    function: F,
    priority: TaskPriority,
    stack_depth: TaskStackDepth,
    name: Option<&str>,
) -> Result<TaskHandle, SpawnError> {
    let entrypoint = Box::new(TaskEntrypoint { function });
    let name = alloc::ffi::CString::new(name.unwrap_or("<unnamed>"))
        .unwrap()
        .into_raw();
    unsafe {
        let task = bail_on!(
            core::ptr::null(),
            pros_sys::task_create(
                Some(TaskEntrypoint::<F>::cast_and_call_external),
                Box::into_raw(entrypoint).cast(),
                priority as _,
                stack_depth as _,
                name,
            )
        );

        _ = alloc::ffi::CString::from_raw(name);

        Ok(TaskHandle { task })
    }
}

/// An owned permission to perform actions on a task.
#[derive(Clone)]
pub struct TaskHandle {
    pub(crate) task: pros_sys::task_t,
}
unsafe impl Send for TaskHandle {}
impl Hash for TaskHandle {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.task.hash(state)
    }
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.task == other.task
    }
}
impl Eq for TaskHandle {}

impl TaskHandle {
    /// Pause execution of the task.
    /// This can have unintended consequences if you are not careful,
    /// for example, if this task is holding a mutex when paused, there is no way to retrieve it until the task is unpaused.
    pub fn pause(&self) {
        unsafe {
            pros_sys::task_suspend(self.task);
        }
    }

    /// Resumes execution of the task.
    pub fn unpause(&self) {
        unsafe {
            pros_sys::task_resume(self.task);
        }
    }

    /// Sets the task's priority, allowing you to control how much cpu time is allocated to it.
    pub fn set_priority(&self, priority: impl Into<u32>) {
        unsafe {
            pros_sys::task_set_priority(self.task, priority.into());
        }
    }

    /// Get the state of the task.
    pub fn state(&self) -> TaskState {
        unsafe { pros_sys::task_get_state(self.task).into() }
    }

    /// Send a notification to the task.
    pub fn notify(&self) {
        unsafe {
            pros_sys::task_notify(self.task);
        }
    }

    /// Waits for the task to finish, and then deletes it.
    pub fn join(self) {
        unsafe {
            pros_sys::task_join(self.task);
        }
    }

    /// Aborts the task and consumes it. Memory allocated by the task will not be freed.
    pub fn abort(self) {
        unsafe {
            pros_sys::task_delete(self.task);
        }
    }

    pub fn name(&self) -> Result<String, Utf8Error> {
        unsafe {
            let name = pros_sys::task_get_name(self.task);
            let name_str = CStr::from_ptr(name);
            Ok(name_str.to_str()?.to_string())
        }
    }
}

/// An ergonomic builder for tasks. Alternatively you can use [`spawn`].
#[derive(Default)]
pub struct Builder<'a> {
    name: Option<&'a str>,
    priority: Option<TaskPriority>,
    stack_depth: Option<TaskStackDepth>,
}

impl<'a> Builder<'a> {
    /// Creates a task builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of the task, this is useful for debugging.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the priority of the task (how much time the scheduler gives to it.).
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets how large the stack for the task is.
    /// This can usually be set to default
    pub fn stack_depth(mut self, stack_depth: TaskStackDepth) -> Self {
        self.stack_depth = Some(stack_depth);
        self
    }

    /// Builds and spawns the task
    pub fn spawn<F>(self, function: F) -> Result<TaskHandle, SpawnError>
    where
        F: FnOnce() + Send + 'static,
    {
        spawn_inner(
            function,
            self.priority.unwrap_or_default(),
            self.stack_depth.unwrap_or_default(),
            self.name,
        )
    }
}

/// Represents the current state of a task.
pub enum TaskState {
    /// The task is currently utilizing the processor
    Running,
    /// The task is currently yielding but may run in the future
    Ready,
    /// The task is blocked. For example, it may be [`sleep`]ing or waiting on a mutex.
    /// Tasks that are in this state will usually return to the task queue after a set timeout.
    Blocked,
    /// The task is suspended. For example, it may be waiting on a mutex or semaphore.
    Suspended,
    /// The task has been deleted using [`TaskHandle::abort`].
    Deleted,
    /// The task's state is invalid somehow
    Invalid,
}

impl From<u32> for TaskState {
    fn from(value: u32) -> Self {
        match value {
            pros_sys::E_TASK_STATE_RUNNING => Self::Running,
            pros_sys::E_TASK_STATE_READY => Self::Ready,
            pros_sys::E_TASK_STATE_BLOCKED => Self::Blocked,
            pros_sys::E_TASK_STATE_SUSPENDED => Self::Suspended,
            pros_sys::E_TASK_STATE_DELETED => Self::Deleted,
            pros_sys::E_TASK_STATE_INVALID => Self::Invalid,
            _ => Self::Invalid,
        }
    }
}

/// Represents how much time the cpu should spend on this task.
/// (Otherwise known as the priority)
#[repr(u32)]
pub enum TaskPriority {
    High = 16,
    Default = 8,
    Low = 1,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Default
    }
}

impl From<TaskPriority> for u32 {
    fn from(val: TaskPriority) -> Self {
        val as u32
    }
}

/// Represents how large of a stack the task should get.
/// Tasks that don't have any or many variables and/or don't need floats can use the low stack depth option.
#[repr(u32)]
pub enum TaskStackDepth {
    Default = 8192,
    Low = 512,
}

impl Default for TaskStackDepth {
    fn default() -> Self {
        Self::Default
    }
}

struct TaskEntrypoint<F> {
    function: F,
}

impl<F> TaskEntrypoint<F>
where
    F: FnOnce(),
{
    unsafe extern "C" fn cast_and_call_external(this: *mut core::ffi::c_void) {
        let this = Box::from_raw(this.cast::<Self>());

        (this.function)()
    }
}

#[derive(Debug, Snafu)]
pub enum SpawnError {
    #[snafu(display("The stack cannot be used as the TCB was not created."))]
    TCBNotCreated,
}

map_errno! {
    SpawnError {
        ENOMEM => SpawnError::TCBNotCreated,
    }
}

/// Blocks the current FreeRTOS task for the given amount of time.
///
/// ## Caveats
///
/// This function will block the entire task, preventing concurrent
/// execution of async code. When in an async context, it is recommended
/// to use [`sleep`] instead.
pub fn delay(duration: Duration) {
    unsafe { pros_sys::delay(duration.as_millis() as u32) }
}

/// An interval that can be used to repeatedly run code at a given rate.
pub struct Interval {
    last_unblock_time: u32,
}

impl Interval {
    /// Creates a new interval. As time passes, the interval's actual delay
    /// will become smaller so that the average rate is maintained.
    pub fn start() -> Self {
        Self {
            last_unblock_time: unsafe { pros_sys::millis() },
        }
    }

    /// Blocks the current FreeRTOS task until the interval has elapsed.
    ///
    /// ## Caveats
    ///
    /// This function will block the entire task, preventing concurrent
    /// execution of async code. When in an async context, it is recommended
    /// to an async-friendly equivalent instead.
    pub fn delay(&mut self, delta: Duration) {
        let delta = delta.as_millis() as u32;
        unsafe {
            // PROS handles loop overruns so there's no need to check for them here
            pros_sys::task_delay_until((&mut self.last_unblock_time) as *mut _, delta);
        }
    }
}

/// A future that will complete after the given duration.
/// Sleep futures that are closer to completion are prioritized to improve accuracy.
pub struct SleepFuture {
    target_millis: u32,
}
impl Future for SleepFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.target_millis < unsafe { pros_sys::millis() } {
            Poll::Ready(())
        } else {
            EXECUTOR.with(|e| {
                e.reactor
                    .borrow_mut()
                    .sleepers
                    .push(cx.waker().clone(), self.target_millis)
            });
            Poll::Pending
        }
    }
}

/// Returns a future that will complete after the given duration.
pub fn sleep(duration: core::time::Duration) -> SleepFuture {
    SleepFuture {
        target_millis: unsafe { pros_sys::millis() + duration.as_millis() as u32 },
    }
}

/// Returns the task the function was called from.
pub fn current() -> TaskHandle {
    unsafe {
        let task = pros_sys::task_get_current();
        TaskHandle { task }
    }
}

/// Gets the first notification in the queue.
/// If there is none, blocks until a notification is received.
/// I am unsure what happens if the thread is unblocked while waiting.
/// returns the value of the notification
pub fn get_notification() -> u32 {
    unsafe { pros_sys::task_notify_take(false, pros_sys::TIMEOUT_MAX) }
}

pub struct SchedulerSuspendGuard {
    _private: (),
}

impl Drop for SchedulerSuspendGuard {
    fn drop(&mut self) {
        unsafe {
            pros_sys::rtos_resume_all();
        }
    }
}

/// Suspends the scheduler, preventing context switches.
/// No other tasks will be run until the returned guard is dropped.
///
/// # Safety
///
/// API functions that have the potential to cause a context switch (e.g. [`delay`], [`get_notification`])
/// must not be called while the scheduler is suspended.
#[must_use = "The scheduler will only remain suspended for the lifetime of the returned guard"]
pub unsafe fn suspend_all() -> SchedulerSuspendGuard {
    pros_sys::rtos_suspend_all();
    SchedulerSuspendGuard { _private: () }
}

#[doc(hidden)]
pub fn __init_entrypoint() {
    unsafe {
        // pros_sys::lcd_initialize();
    }
}
