use std::any::type_name;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, ThreadId};

use godot::builtin::{Callable, RustCallable, Signal, Variant};
use godot::classes::object::ConnectFlags;
use godot::classes::Os;
use godot::meta::{FromGodot, ToGodot};
use godot::obj::{EngineEnum, Gd, NewGd};
use godot::prelude::{godot_api, GodotClass};

pub fn godot_task(future: impl Future<Output = ()> + 'static) -> TaskHandle {
    let os = Os::singleton();

    // Spawning new tasks is only allowed on the main thread for now.
    // We can not accept Sync + Send futures since all object references (i.e. Gd<T>) are not thread-safe. So a future has to remain on the
    // same thread it was created on. Godots signals on the other hand can be emitted on any thread, so it can't be guaranteed on which thread
    // a future will be polled.
    // By limiting async tasks to the main thread we can redirect all signal callbacks back to the main thread via `call_deferred`.
    //
    // Once thread-safe futures are possible the restriction can be lifted.
    if os.get_thread_caller_id() != os.get_main_thread_id() {
        panic!("godot_task can only be used on the main thread!");
    }

    let (task_handle, waker): (_, Waker) = ASYNC_RUNTIME.with_borrow_mut(move |rt| {
        let task_handle = rt.add_task(Box::pin(future));
        let waker = Arc::new(GodotWaker::new(
            task_handle.index,
            task_handle.id,
            thread::current().id(),
        ))
        .into();

        (task_handle, waker)
    });

    waker.wake();
    task_handle
}

thread_local! { pub(crate) static ASYNC_RUNTIME: RefCell<AsyncRuntime> = RefCell::new(AsyncRuntime::new()); }

#[derive(Default)]
enum FutureSlotState<T> {
    /// Slot is currently empty.
    #[default]
    Empty,
    /// Slot was previously occupied but the future has been canceled or the slot reused.
    Gone,
    /// Slot contains a pending future.
    Pending(T),
    /// slot contains a future which is currently being polled.
    Polling,
}

struct FutureSlot<T> {
    value: FutureSlotState<T>,
    id: u64,
}

impl<T> FutureSlot<T> {
    fn pending(id: u64, value: T) -> Self {
        Self {
            value: FutureSlotState::Pending(value),
            id,
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self.value, FutureSlotState::Empty | FutureSlotState::Gone)
    }

    fn clear(&mut self) {
        self.value = FutureSlotState::Empty;
    }

    #[allow(dead_code)]
    fn cancel(&mut self) {
        self.value = FutureSlotState::Gone;
    }

    fn take(&mut self, id: u64) -> FutureSlotState<T> {
        match self.value {
            FutureSlotState::Empty => FutureSlotState::Empty,
            FutureSlotState::Polling => FutureSlotState::Polling,
            FutureSlotState::Gone => FutureSlotState::Gone,
            FutureSlotState::Pending(_) if self.id != id => FutureSlotState::Gone,
            FutureSlotState::Pending(_) => {
                std::mem::replace(&mut self.value, FutureSlotState::Polling)
            }
        }
    }

    fn park(&mut self, value: T) {
        match self.value {
            FutureSlotState::Empty | FutureSlotState::Gone => {
                panic!("Future slot is currently unoccupied, future can not be parked here!");
            }
            FutureSlotState::Pending(_) => {
                panic!("Future slot is already occupied by a different future!")
            }
            FutureSlotState::Polling => {
                self.value = FutureSlotState::Pending(value);
            }
        }
    }
}

pub struct TaskHandle {
    index: usize,
    id: u64,
    _pd: PhantomData<*const ()>,
}

impl TaskHandle {
    fn new(index: usize, id: u64) -> Self {
        Self {
            index,
            id,
            _pd: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(self) {
        ASYNC_RUNTIME.with_borrow_mut(|rt| {
            let Some(task) = rt.tasks.get(self.index) else {
                return;
            };

            let alive = match task.value {
                FutureSlotState::Empty | FutureSlotState::Gone => false,
                FutureSlotState::Pending(_) => task.id == self.id,
                FutureSlotState::Polling => panic!("Can not cancel future from inside it!"),
            };

            if !alive {
                return;
            }

            rt.cancel_task(self.index);
        })
    }

    pub fn is_pending(&self) -> bool {
        ASYNC_RUNTIME.with_borrow(|rt| {
            let slot = rt.tasks.get(self.index).expect("Slot at index must exist!");

            if slot.id != self.id {
                return false;
            }

            matches!(slot.value, FutureSlotState::Pending(_))
        })
    }
}

#[derive(Default)]
pub(crate) struct AsyncRuntime {
    tasks: Vec<FutureSlot<Pin<Box<dyn Future<Output = ()>>>>>,
    task_counter: u64,
}

impl AsyncRuntime {
    fn new() -> Self {
        Self {
            tasks: Vec::with_capacity(10),
            task_counter: 0,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.task_counter;
        self.task_counter += 1;
        id
    }

    fn add_task<F: Future<Output = ()> + 'static>(&mut self, future: F) -> TaskHandle {
        let id = self.next_id();
        let slot = self
            .tasks
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_empty());

        let boxed = Box::pin(future);

        let index = match slot {
            Some((index, slot)) => {
                *slot = FutureSlot::pending(id, boxed);
                index
            }
            None => {
                self.tasks.push(FutureSlot::pending(id, boxed));
                self.tasks.len() - 1
            }
        };

        TaskHandle::new(index, id)
    }

    fn get_task(
        &mut self,
        index: usize,
        id: u64,
    ) -> FutureSlotState<Pin<Box<dyn Future<Output = ()> + 'static>>> {
        let slot = self.tasks.get_mut(index);

        slot.map(|inner| inner.take(id)).unwrap_or_default()
    }

    fn clear_task(&mut self, index: usize) {
        self.tasks[index].clear();
    }

    fn cancel_task(&mut self, index: usize) {
        self.tasks[index].cancel();
    }

    fn park_task(&mut self, index: usize, future: Pin<Box<dyn Future<Output = ()>>>) {
        self.tasks[index].park(future);
    }
}

struct GodotWaker {
    runtime_index: usize,
    task_id: u64,
    thread_id: ThreadId,
}

impl GodotWaker {
    fn new(index: usize, task_id: u64, thread_id: ThreadId) -> Self {
        Self {
            runtime_index: index,
            thread_id,
            task_id,
        }
    }
}

impl Wake for GodotWaker {
    fn wake(self: std::sync::Arc<Self>) {
        let callable = Callable::from_fn("GodotWaker::wake", move |_args| {
            let current_thread = thread::current().id();

            if self.thread_id != current_thread {
                panic!("trying to poll future on a different thread!\nCurrent Thread: {:?}, Future Thread: {:?}", current_thread, self.thread_id);
            }

            let waker: Waker = self.clone().into();
            let mut ctx = Context::from_waker(&waker);

            // take future out of the runtime.
            let future = ASYNC_RUNTIME.with_borrow_mut(|rt| {
                match rt.get_task(self.runtime_index, self.task_id) {
                    FutureSlotState::Empty => {
                        panic!("Future no longer exists when waking it! This is a bug!");
                    },

                    FutureSlotState::Gone => {
                        None
                    }

                    FutureSlotState::Polling => {
                        panic!("The same GodotWaker has been called recursively, this is not expected!");
                    }

                    FutureSlotState::Pending(future) => Some(future),
                }
            });

            let Some(mut future) = future else {
                // future has been canceled while the waker was already triggered.
                return Ok(Variant::nil());
            };

            let result = future.as_mut().poll(&mut ctx);

            // update runtime.
            ASYNC_RUNTIME.with_borrow_mut(|rt| match result {
                Poll::Pending => rt.park_task(self.runtime_index, future),
                Poll::Ready(()) => rt.clear_task(self.runtime_index),
            });

            Ok(Variant::nil())
        });

        // shedule waker to poll the future on the end of the frame.
        callable.to_variant().call("call_deferred", &[]);
    }
}

pub struct SignalFuture<R: FromSignalArgs> {
    state: Arc<Mutex<(Option<R>, Option<Waker>)>>,
    callable: Callable,
    signal: Signal,
}

impl<R: FromSignalArgs> SignalFuture<R> {
    fn new(signal: Signal) -> Self {
        let state = Arc::new(Mutex::new((None, Option::<Waker>::None)));
        let callback_state = state.clone();

        // the callable currently requires that the return value is Sync + Send
        let callable = Callable::from_fn("async_task", move |args: &[&Variant]| {
            let mut lock = callback_state.lock().unwrap();
            let waker = lock.1.take();

            lock.0.replace(R::from_args(args));
            drop(lock);

            if let Some(waker) = waker {
                waker.wake();
            }

            Ok(Variant::nil())
        });

        signal.connect(callable.clone(), ConnectFlags::ONE_SHOT.ord() as i64);

        Self {
            state,
            callable,
            signal,
        }
    }
}

impl<R: FromSignalArgs> Future for SignalFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut lock = self.state.lock().unwrap();

        if let Some(result) = lock.0.take() {
            return Poll::Ready(result);
        }

        lock.1.replace(cx.waker().clone());

        Poll::Pending
    }
}

impl<R: FromSignalArgs> Drop for SignalFuture<R> {
    fn drop(&mut self) {
        if !self.callable.is_valid() {
            return;
        }

        if self.signal.object().is_none() {
            return;
        }

        if self.signal.is_connected(self.callable.clone()) {
            self.signal.disconnect(self.callable.clone());
        }
    }
}

struct GuaranteedSignalFutureWaker<R> {
    state: Arc<Mutex<(GuaranteedSignalFutureState<R>, Option<Waker>)>>,
}

impl<R> Clone for GuaranteedSignalFutureWaker<R> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<R> GuaranteedSignalFutureWaker<R> {
    fn new(state: Arc<Mutex<(GuaranteedSignalFutureState<R>, Option<Waker>)>>) -> Self {
        Self { state }
    }
}

impl<R> std::hash::Hash for GuaranteedSignalFutureWaker<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.state) as usize);
    }
}

impl<R> PartialEq for GuaranteedSignalFutureWaker<R> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<R: FromSignalArgs> RustCallable for GuaranteedSignalFutureWaker<R> {
    fn invoke(&mut self, args: &[&Variant]) -> Result<Variant, ()> {
        let mut lock = self.state.lock().unwrap();
        let waker = lock.1.take();

        lock.0 = GuaranteedSignalFutureState::Ready(R::from_args(args));
        drop(lock);

        if let Some(waker) = waker {
            waker.wake();
        }

        Ok(Variant::nil())
    }
}

impl<R> Display for GuaranteedSignalFutureWaker<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SafeCallable::<{}>", type_name::<R>())
    }
}

impl<R> Drop for GuaranteedSignalFutureWaker<R> {
    fn drop(&mut self) {
        let mut lock = self.state.lock().unwrap();

        if !matches!(lock.0, GuaranteedSignalFutureState::Pending) {
            return;
        }

        lock.0 = GuaranteedSignalFutureState::Dead;

        if let Some(ref waker) = lock.1 {
            waker.wake_by_ref();
        }
    }
}

#[derive(Default)]
enum GuaranteedSignalFutureState<T> {
    #[default]
    Pending,
    Ready(T),
    Dead,
    Dropped,
}

impl<T> GuaranteedSignalFutureState<T> {
    fn take(&mut self) -> Self {
        let new_value = match self {
            Self::Pending => Self::Pending,
            Self::Ready(_) | Self::Dead => Self::Dead,
            Self::Dropped => Self::Dropped,
        };

        std::mem::replace(self, new_value)
    }
}

pub struct GuaranteedSignalFuture<R: FromSignalArgs> {
    state: Arc<Mutex<(GuaranteedSignalFutureState<R>, Option<Waker>)>>,
    callable: GuaranteedSignalFutureWaker<R>,
    signal: Signal,
}

impl<R: FromSignalArgs + Debug> GuaranteedSignalFuture<R> {
    fn new(signal: Signal) -> Self {
        let state = Arc::new(Mutex::new((
            GuaranteedSignalFutureState::Pending,
            Option::<Waker>::None,
        )));

        // the callable currently requires that the return value is Sync + Send
        let callable = GuaranteedSignalFutureWaker::new(state.clone());

        signal.connect(
            Callable::from_custom(callable.clone()),
            ConnectFlags::ONE_SHOT.ord() as i64,
        );

        Self {
            state,
            callable,
            signal,
        }
    }
}

impl<R: FromSignalArgs> Future for GuaranteedSignalFuture<R> {
    type Output = Option<R>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut lock = self.state.lock().unwrap();

        lock.1.replace(cx.waker().clone());

        let value = lock.0.take();

        match value {
            GuaranteedSignalFutureState::Pending => Poll::Pending,
            GuaranteedSignalFutureState::Dropped => unreachable!(),
            GuaranteedSignalFutureState::Dead => Poll::Ready(None),
            GuaranteedSignalFutureState::Ready(value) => Poll::Ready(Some(value)),
        }
    }
}

impl<R: FromSignalArgs> Drop for GuaranteedSignalFuture<R> {
    fn drop(&mut self) {
        if self.signal.object().is_none() {
            return;
        }

        self.state.lock().unwrap().0 = GuaranteedSignalFutureState::Dropped;

        let gd_callable = Callable::from_custom(self.callable.clone());

        if self.signal.is_connected(gd_callable.clone()) {
            self.signal.disconnect(gd_callable);
        }
    }
}

pub trait FromSignalArgs: Sync + Send + 'static {
    fn from_args(args: &[&Variant]) -> Self;
}

impl<R: FromGodot + Sync + Send + 'static> FromSignalArgs for R {
    fn from_args(args: &[&Variant]) -> Self {
        args.first()
            .map(|arg| (*arg).to_owned())
            .unwrap_or_default()
            .to()
    }
}

// more of these should be generated via macro to support more than two signal arguments
// impl<R1: FromGodot + Sync + Send + 'static, R2: FromGodot + Sync + Send + 'static> FromSignalArgs
//     for (R1, R2)
// {
//     fn from_args(args: &[&Variant]) -> Self {
//         (args[0].to(), args[0].to())
//     }
// }

// Signal should implement IntoFuture for convenience. Keeping ToSignalFuture around might still be desirable, though. It allows to reuse i
// the same signal instance multiple times.
pub trait ToSignalFuture<R: FromSignalArgs> {
    fn to_future(&self) -> SignalFuture<R>;
}

impl<R: FromSignalArgs> ToSignalFuture<R> for Signal {
    fn to_future(&self) -> SignalFuture<R> {
        SignalFuture::new(self.clone())
    }
}

pub trait ToGuaranteedSignalFuture<R: FromSignalArgs + Debug> {
    #[allow(dead_code)]
    fn to_guaranteed_future(&self) -> GuaranteedSignalFuture<R>;
}

impl<R: FromSignalArgs + Debug> ToGuaranteedSignalFuture<R> for Signal {
    fn to_guaranteed_future(&self) -> GuaranteedSignalFuture<R> {
        GuaranteedSignalFuture::new(self.clone())
    }
}

#[derive(GodotClass)]
#[class(base = RefCounted, init)]
pub struct GodotFuture;

#[godot_api]
impl GodotFuture {
    /// Returns an object which emits the completed signal once the asynchronus method has finished processing.

    /// Is emitted as soon as the async operation of the function has been completed.
    #[signal]
    fn completed(result: Variant);
}

/// Creates a new GodotFuture that can be returned from a function which performs an async operation. This works similar to GdFunctionState.
///
/// Example:
/// ```rs
/// fn async_do_task() -> Gd<GodotFuture> {
///     let (resolve, future) = godot_future();
///
///     godot_task(async move {
///         // do async operations
///         resolve(true);
///     });
///
///     future
/// }
/// ```
pub fn godot_future<R: ToGodot>() -> (impl Fn(R), Gd<GodotFuture>) {
    let future = GodotFuture::new_gd();
    let sender = future.clone();

    (
        move |value: R| {
            Signal::from_object_signal(&sender, "completed").emit(&[value.to_variant()])
        },
        future,
    )
}

#[cfg(test)]
mod tests {
    use std::{
        hash::{DefaultHasher, Hash, Hasher},
        sync::Arc,
    };

    use super::GuaranteedSignalFutureWaker;

    #[test]
    fn guaranteed_future_waker_cloned_hash() {
        let waker_a = GuaranteedSignalFutureWaker::<u8>::new(Arc::default());
        let waker_b = waker_a.clone();

        let mut hasher = DefaultHasher::new();
        waker_a.hash(&mut hasher);
        let hash_a = hasher.finish();

        let mut hasher = DefaultHasher::new();
        waker_b.hash(&mut hasher);
        let hash_b = hasher.finish();

        assert_eq!(hash_a, hash_b);
    }
}
