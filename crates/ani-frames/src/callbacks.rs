use std::marker::PhantomData;
use std::mem::{replace};
use std::rc::Rc;
use std::sync::Mutex;

use wasm_bindgen::{JsValue, JsCast, UnwrapThrowExt, closure::Closure};
use std::fmt::Debug;

#[inline]
fn request_af<F: FnOnce() + 'static>(cls: F) -> i32 {
    web_sys::window().unwrap_throw()
        .request_animation_frame(Closure::once(cls).as_ref().unchecked_ref())
        .unwrap_throw()
}

#[inline]
fn cancel_af(index: i32) -> Result<(), JsValue> {
    web_sys::window().unwrap_throw()
        .cancel_animation_frame(index)
}


pub struct Animation<C: 'static>(Mutex<Ani<C>>);

impl Animation<()> {
    pub fn request_frame<F: FnOnce() + 'static>(callback: F) -> AniIndex<F> {
        AniIndex::new(request_af(callback))
    }

    pub fn cancel_frame<F>(index: AniIndex<F>) {
        cancel_af(index.inner).unwrap_throw()
    }
}

impl<C: 'static> Animation<C> {
    fn from_ani(ani: Ani<C>) -> Self {
        Animation(Mutex::new(ani))
    }

    pub fn new() -> Rc<Self> {
        let ani = Rc::new(Animation::from_ani(Ani::paused()));
        ani.start();
        ani
    }

    pub fn paused() -> Rc<Self> {
        Rc::new(Animation::from_ani(Ani::paused()))
    }

    pub fn add<F>(&mut self, cb: &'static F) -> AniIndex<C>
        where F: Fn(&AniState) + 'static
    {
        let mut guard = self.0.lock().unwrap_throw();

        let new_ix = AniIndex::new(guard.next_index.inner + 1);
        let old_ix = replace(&mut guard.next_index, new_ix);

        guard.callbacks.push((AniIndex::new(old_ix.inner), cb));
        return old_ix
    }

    pub fn remove(&mut self, index: AniIndex<C>) -> Result<(), ()> {
        let mut guard = self.0.lock().unwrap_throw();

        let pos = guard.callbacks.iter()
            .position(|(ix, _)| ix.inner == index.inner)
            .ok_or(())?;
        guard.callbacks.remove(pos);
        Ok(())
    }


    fn do_loop(ani: Rc<Self>) {
        let cloned = ani.clone();
        let num = request_af(|| Self::do_loop(cloned));

        let mut guard = ani.0.lock().unwrap_throw();
        guard.state = AniState::Running(num);

        for (_, cb) in &guard.callbacks {
            cb(&guard.state)
        }
    }
}

pub trait AnimationRc {
    fn start(&self);
    fn pause(&self);
    fn once(&self);
}

impl<C> AnimationRc for Rc<Animation<C>> {
    fn start(&self) {
        let mut guard = self.0.lock().unwrap_throw();
        match guard.state {
            AniState::Paused => {
                let cloned = self.clone();
                let num = request_af(move || Animation::do_loop(cloned));
                guard.state = AniState::Running(num);
            }
            AniState::Running(_) => (),
            AniState::RunningOnce(ix) => {
                cancel_af(ix).unwrap_throw();

                let cloned = self.clone();
                let num = request_af(|| Animation::do_loop(cloned));
                guard.state = AniState::Running(num);
            }
        }
    }

    fn pause(&self) {
        let mut guard = self.0.lock().unwrap_throw();
        match guard.state {
            AniState::Paused => (),
            AniState::Running(ix) => {
                cancel_af(ix).unwrap_throw();
                guard.state = AniState::Paused;
            }
            AniState::RunningOnce(ix) => {
                cancel_af(ix).unwrap_throw();
                guard.state = AniState::Paused;
            }
        }
    }

    fn once(&self) {
        let mut guard = self.0.lock().unwrap_throw();
        match guard.state {
            AniState::Paused => {
                let cloned = self.clone();
                let num = request_af(move || {
                    let guard = cloned.0.lock().unwrap_throw();

                    for (_, cb) in &guard.callbacks {
                        cb(&guard.state)
                    }
                });
                guard.state = AniState::RunningOnce(num);
            }
            AniState::Running(_) | AniState::RunningOnce(_) => ()
        }
    }
}


struct Ani<C: 'static> {
    state: AniState,
    next_index: AniIndex<C>,
    callbacks: Vec<(AniIndex<C>, &'static dyn Fn(&AniState))>,
}

impl<C: 'static> Ani<C> {
    fn paused() -> Self {
        Ani {
            state: AniState::Paused,
            next_index: AniIndex::new(1),
            callbacks: vec![]
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AniState {
    Paused,
    Running(i32),
    RunningOnce(i32),
}

pub struct AniIndex<C> {
    inner: i32,
    _marker: PhantomData<C>,
}

impl<C> Debug for AniIndex<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "AniIndex({})", self.inner)
    }
}

impl<C> AniIndex<C> {
    fn new(index: i32) -> Self {
        AniIndex { inner: index, _marker: PhantomData }
    }
}