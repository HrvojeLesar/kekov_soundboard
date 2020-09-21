use crate::prelude::*;

use actix::prelude::*;

use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "Option<(usize, String)>")]
pub struct NewActor {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct KillActor {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateActors;

pub struct VoiceMonitorServer {
    sessions: HashMap<usize, Recipient<Message>>,
    data: web::Data<RwLock<websocket_voice_monitor::ConnectedMembers>>,
    rng: ThreadRng
}

impl VoiceMonitorServer {
    pub fn new(data: web::Data<RwLock<websocket_voice_monitor::ConnectedMembers>>) -> Self {
        VoiceMonitorServer {
            sessions: HashMap::new(),
            data: data,
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for VoiceMonitorServer {
    type Context = Context<Self>;
}

impl Handler<NewActor> for VoiceMonitorServer {
    type Result = Option<(usize, String)>;

    fn handle(&mut self, msg: NewActor, _: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        let data = serde_json::to_string(&*self.data.read().unwrap()).unwrap();

        Some((id, data))
    }
}

impl Handler<KillActor> for VoiceMonitorServer {
    type Result = ();

    fn handle(&mut self, msg: KillActor, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<UpdateActors> for VoiceMonitorServer {
    type Result = (); 

    fn handle(&mut self, _: UpdateActors, _: &mut Context<Self>) {
        for actor in self.sessions.values() {
            let data = serde_json::to_string(&*self.data.read().unwrap()).unwrap();
            actor.do_send(Message(data)).unwrap();
        }
    }
}