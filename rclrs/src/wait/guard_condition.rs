use crate::rcl_bindings::*;
use crate::{Context, RclrsError, ToResult};

use std::sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard};

/// A struct for encapsulating a guard condition - a waitable trigger
pub struct GuardCondition {
    /// The rcl_guard_condition_t that this struct encapsulates.
    rcl_guard_condition: Arc<Mutex<rcl_guard_condition_t>>,
    /// An optional callback to call when this guard condition is triggered.
    callback: Option<Box<dyn Fn(usize)>>,
    /// A flag to indicate if this guard condition has already been assigned to a wait set.
    pub(crate) in_use_by_wait_set: Arc<AtomicBool>,
    /// A count for the number of times this guard condition was triggered, but no callback was assigned.
    unread_count: usize,
}

impl Drop for GuardCondition {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: No precondition for this function (besides passing in a valid guard condition)
            rcl_guard_condition_fini(&mut *self.rcl_guard_condition.lock().unwrap());
        }
    }
}

impl GuardCondition {
    /// Creates a new guard condition
    pub fn new(context: &Context) -> Self {
        let mut guard_condition = unsafe { rcl_get_zero_initialized_guard_condition() };
        unsafe {
            // SAFETY: The context must be valid. No other preconditions for this function.
            rcl_guard_condition_init(
                &mut guard_condition,
                &mut *context.rcl_context_mtx.lock().unwrap(),
                rcl_guard_condition_get_default_options(),
            );
        }

        Self {
            rcl_guard_condition: Arc::new(Mutex::new(guard_condition)),
            callback: None,
            in_use_by_wait_set: Arc::new(AtomicBool::new(false)),
            unread_count: 0,
        }
    }

    /// Locks the underlying guard condition and returns it.
    pub fn lock(&self) -> MutexGuard<rcl_guard_condition_t> {
        self.rcl_guard_condition.lock().unwrap()
    }

    /// Sets the callback to call when this guard condition is triggered.
    pub fn set_on_trigger_callback(&mut self, callback: Option<Box<dyn Fn(usize)>>) {
        match callback {
            Some(callback) => {
                if self.unread_count > 0 {
                    callback(self.unread_count);
                    self.unread_count = 0;
                }
                self.callback = Some(callback);
            }
            None => {
                self.callback = None;
                self.unread_count = 0;
            }
        }
    }

    /// Triggers this guard condition, activating the wait set, and calling the optionally assigned callback.
    pub fn trigger(&mut self) -> Result<(), RclrsError> {
        unsafe {
            // SAFETY: The rcl_guard_condition_t is valid.
            rcl_trigger_guard_condition(&mut *self.rcl_guard_condition.lock().unwrap()).ok()?;
        }
        match &self.callback {
            Some(callback) => callback(1),
            None => self.unread_count += 1,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WaitSet;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_guard_condition() -> Result<(), RclrsError> {
        let context = Context::new([])?;
        let mut gc = GuardCondition::new(&context);

        let atomic_usize = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let atomic_usize_for_closure = Arc::clone(&atomic_usize);

        gc.set_on_trigger_callback(Some(Box::new(move |count| {
            atomic_usize_for_closure.store(count, Ordering::Relaxed);
        })));

        gc.trigger()?;

        assert_eq!(atomic_usize.load(Ordering::Relaxed), 1);

        Ok(())
    }

    #[test]
    fn test_guard_condition_wait() -> Result<(), RclrsError> {
        let context = Context::new([])?;
        let gc = Arc::new(Mutex::new(GuardCondition::new(&context)));

        let atomic_usize = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let atomic_usize_for_closure = Arc::clone(&atomic_usize);

        gc.lock()
            .unwrap()
            .set_on_trigger_callback(Some(Box::new(move |count| {
                atomic_usize_for_closure.store(count, Ordering::Relaxed);
            })));

        let mut ws = WaitSet::new(0, 1, 0, 0, 0, 0, &context)?;
        ws.add_guard_condition(Arc::clone(&gc))?;
        gc.lock().unwrap().trigger()?;

        assert_eq!(atomic_usize.load(Ordering::Relaxed), 1);
        let wait_result = ws.wait(Some(std::time::Duration::from_millis(10)))?;
        assert_eq!(wait_result.guard_conditions.len(), 1);

        Ok(())
    }
}
