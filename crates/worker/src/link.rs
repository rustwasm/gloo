use crate::Shared;
use crate::{Agent, HandlerId};
use std::cell::RefCell;
use std::fmt;
#[cfg(feature = "futures")]
use std::future::Future;
use std::rc::Rc;

/// Defines communication from Worker to Consumers
pub(crate) trait Responder<AGN: Agent> {
    /// Implementation for communication channel from Worker to Consumers
    fn respond(&self, id: HandlerId, output: AGN::Output);
}

/// Link to agent's scope for creating callbacks.
pub struct AgentLink<AGN: Agent> {
    scope: AgentScope<AGN>,
    responder: Rc<dyn Responder<AGN>>,
}

impl<AGN: Agent> AgentLink<AGN> {
    /// Create link for a scope.
    pub(crate) fn connect<T>(scope: &AgentScope<AGN>, responder: T) -> Self
    where
        T: Responder<AGN> + 'static,
    {
        AgentLink {
            scope: scope.clone(),
            responder: Rc::new(responder),
        }
    }

    /// Send response to an agent.
    pub fn respond(&self, id: HandlerId, output: AGN::Output) {
        self.responder.respond(id, output);
    }

    /// Send a message to the agent
    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<AGN::Message>,
    {
        self.scope.send(AgentLifecycleEvent::Message(msg.into()));
    }

    /// Send an input to self
    pub fn send_input<T>(&self, input: T)
    where
        T: Into<AGN::Input>,
    {
        let handler_id = HandlerId::new(0, false);
        self.scope
            .send(AgentLifecycleEvent::Input(input.into(), handler_id));
    }

    /// Create a callback which will send a message to the agent when invoked.
    pub fn callback<F, IN, M>(&self, function: F) -> Rc<dyn Fn(IN)>
    where
        M: Into<AGN::Message>,
        F: Fn(IN) -> M + 'static,
    {
        let scope = self.scope.clone();
        let closure = move |input| {
            let output = function(input).into();
            scope.send(AgentLifecycleEvent::Message(output));
        };
        Rc::new(closure)
    }

    /// This method creates a callback which returns a Future which
    /// returns a message to be sent back to the agent
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and
    /// will leak.
    #[cfg(feature = "futures")]
    #[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
    pub fn callback_future<FN, FU, IN, M>(&self, function: FN) -> Rc<dyn Fn(IN)>
    where
        M: Into<AGN::Message>,
        FU: Future<Output = M> + 'static,
        FN: Fn(IN) -> FU + 'static,
    {
        let link = self.clone();

        let closure = move |input: IN| {
            let future: FU = function(input);
            link.send_future(future);
        };

        Rc::new(closure)
    }

    /// This method processes a Future that returns a message and sends it back to the agent.
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    #[cfg(feature = "futures")]
    #[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
    pub fn send_future<F, M>(&self, future: F)
    where
        M: Into<AGN::Message>,
        F: Future<Output = M> + 'static,
    {
        let link: AgentLink<AGN> = self.clone();
        let js_future = async move {
            let message: AGN::Message = future.await.into();
            let cb = link.callback(|m: AGN::Message| m);
            (*cb)(message);
        };
        wasm_bindgen_futures::spawn_local(js_future);
    }
}

impl<AGN: Agent> fmt::Debug for AgentLink<AGN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AgentLink<_>")
    }
}

impl<AGN: Agent> Clone for AgentLink<AGN> {
    fn clone(&self) -> Self {
        AgentLink {
            scope: self.scope.clone(),
            responder: self.responder.clone(),
        }
    }
}
/// This struct holds a reference to a component and to a global scheduler.
pub(crate) struct AgentScope<AGN: Agent> {
    state: Shared<AgentState<AGN>>,
}

impl<AGN: Agent> fmt::Debug for AgentScope<AGN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AgentScope<_>")
    }
}

impl<AGN: Agent> Clone for AgentScope<AGN> {
    fn clone(&self) -> Self {
        AgentScope {
            state: self.state.clone(),
        }
    }
}

impl<AGN: Agent> AgentScope<AGN> {
    /// Create agent scope
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(AgentState::new()));
        AgentScope { state }
    }

    /// Schedule message for sending to agent
    pub fn send(&self, event: AgentLifecycleEvent<AGN>) {
        let runnable = Box::new(AgentRunnable {
            state: self.state.clone(),
            event,
        });
        runnable.run();
    }
}

impl<AGN: Agent> Default for AgentScope<AGN> {
    fn default() -> Self {
        Self::new()
    }
}

struct AgentState<AGN> {
    agent: Option<AGN>,
    // TODO(#939): Use agent field to control create message this flag
    destroyed: bool,
}

impl<AGN> AgentState<AGN> {
    fn new() -> Self {
        AgentState {
            agent: None,
            destroyed: false,
        }
    }
}

/// Internal Agent lifecycle events
#[derive(Debug)]
pub(crate) enum AgentLifecycleEvent<AGN: Agent> {
    /// Request to create link
    Create(AgentLink<AGN>),
    /// Internal Agent message
    Message(AGN::Message),
    /// Client connected
    Connected(HandlerId),
    /// Received message from Client
    Input(AGN::Input, HandlerId),
    /// Client disconnected
    Disconnected(HandlerId),
    /// Request to destroy agent
    Destroy,
}

struct AgentRunnable<AGN: Agent> {
    state: Shared<AgentState<AGN>>,
    event: AgentLifecycleEvent<AGN>,
}

impl<AGN> AgentRunnable<AGN>
where
    AGN: Agent,
{
    fn run(self) {
        let mut state = self.state.borrow_mut();
        if state.destroyed {
            return;
        }
        match self.event {
            AgentLifecycleEvent::Create(link) => {
                state.agent = Some(AGN::create(link));
            }
            AgentLifecycleEvent::Message(msg) => {
                state
                    .agent
                    .as_mut()
                    .expect("agent was not created to process messages")
                    .update(msg);
            }
            AgentLifecycleEvent::Connected(id) => {
                state
                    .agent
                    .as_mut()
                    .expect("agent was not created to send a connected message")
                    .connected(id);
            }
            AgentLifecycleEvent::Input(inp, id) => {
                state
                    .agent
                    .as_mut()
                    .expect("agent was not created to process inputs")
                    .handle_input(inp, id);
            }
            AgentLifecycleEvent::Disconnected(id) => {
                state
                    .agent
                    .as_mut()
                    .expect("agent was not created to send a disconnected message")
                    .disconnected(id);
            }
            AgentLifecycleEvent::Destroy => {
                let mut agent = state
                    .agent
                    .take()
                    .expect("trying to destroy not existent agent");
                agent.destroy();
                state.destroyed = true;
            }
        }
    }
}
