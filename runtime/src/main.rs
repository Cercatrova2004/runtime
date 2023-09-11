use std::future::Future;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::cell::RefCell;
use std::task::Waker;
use std::task::Poll;
use std::task::Context;
use std::task::Wake;
use std::sync::Mutex;
use std::sync::Condvar;
use std::collections::VecDeque;
use async_channel::bounded;

struct Task {
    future: RefCell<BoxFuture<'static, ()>>,     
    signal: Arc<Signal>,
}

struct Signal{
    state:Mutex<State>,
    cond:Condvar,
}

impl Signal {
    fn new() -> Signal {
        Signal { state: std::sync::Mutex::new(State::Notified), cond: Condvar::new() }
    }

    fn wait(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Notified => *state = State::Empty,
            State::Waiting => {
                panic!("multiple wait");
            }
            State::Empty => {
                *state = State::Waiting;
                while let State::Waiting = *state {
                    state = self.cond.wait(state).unwrap();
                }
            }
        }
    }

    fn notify(&self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Notified => {}
            State::Empty => *state = State::Notified,
            State::Waiting => {
                *state = State::Empty;
                self.cond.notify_one();
            }
        }
    }
}

impl Wake for Signal {
    fn wake(self: Arc<Self>) {
        self.notify();
    }
}

unsafe impl Send for Task {}   
unsafe impl Sync for Task {}
    
impl Wake for Task{    
    fn wake(self: Arc<Self>) {    
        RUNNABLE.with(|runnable: &Mutex<VecDeque<Arc<Task>>>|runnable.lock().unwrap().push_back(self.clone()));
        self.signal.notify();
    }
}

fn block_on<F:Future>(future:F)->F::Output{
    scoped_tls::scoped_thread_local! (static SIGNAL: Arc<Signal>);
    scoped_tls::scoped_thread_local! (static RUNNABLE: Mutex<VecDeque<Arc<Task>>>);

    static runnable: Mutex<VecDeque<Arc<Task>>> = Mutex::new(VecDeque::with_capacity(1024));
    SIGNAL.set(&signal, ||{
        RUNNABLE.set(&runnable, || {
            loop {
                if let Poll::Ready(Output) = main_fut.as_mut().Poll(&mut cx) {
                    return output;
                }
                while let Some(task) = runnabte.lock().unwrap().pop_front() {
                    let waker: Waker = Waker::from(task.ctonefl);
                    let mut cx = Context::from_waker(&waker);
                    let _ = task.future.borrow_mut().as_mut().Poll(&mut cx);
                }
            signal.wait ()
            }
        })
    })
}

async fn demo() {
    let (tx,rx) = async_channel::bounded(1);
    println!("hello worLd!");
    let _= rx.recv().await;
}

async fn demo2(tx: async_channel::Sender<()>) {
    println!("hello world2!");
    let _= tx.send(()).await;
}

fn main() {
    block_on(future:demo());
    println!("It's blocked!");
}