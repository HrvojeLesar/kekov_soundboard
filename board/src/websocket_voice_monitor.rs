use crate::prelude::*;

use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use std::io::{BufReader};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

const LOCALPORT: u16 = 2222;

pub struct VoiceMonitor {
    id: usize,
    server_addr: Addr<websocket_voice_monitor_server::VoiceMonitorServer>,
    heartbeat: Instant,
}

impl Actor for VoiceMonitor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);

        let address = ctx.address().recipient();
        self.server_addr
            .send(websocket_voice_monitor_server::NewActor {
                addr: address,
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        let res = res.unwrap();
                        act.id = res.0;
                        ctx.text(res.1);
                    },
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server_addr.do_send(websocket_voice_monitor_server::KillActor {
            id: self.id
        });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for VoiceMonitor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            },
            Ok(ws::Message::Binary(_)) | Ok(ws::Message::Text(_)) => (),
            Ok(ws::Message::Close(reason)) => {
                println!("WSM | Close websocket");
                ctx.close(reason);
                ctx.stop();
            }
            _ => {
                println!("WSM | Recieved unknown message!\nStopping websocket...");
                ctx.stop()
            }
        }
    }
}

impl Handler<websocket_voice_monitor_server::Message> for VoiceMonitor {
    type Result = ();

    fn handle(&mut self, msg: websocket_voice_monitor_server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl VoiceMonitor {
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                println!("Websocket heartbeat failed, disconnecing socket");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedMembers {
    pub members: Option<Vec<String>>,
    pub avatars: Option<Vec<String>>,
    pub channel:  Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
struct ConnectedMembersString {
    members: String,
}

impl Default for ConnectedMembers {
    fn default() -> Self {
        ConnectedMembers {
            members: None,
            avatars: None,
            channel: None
        }
    }
}

pub async fn start_voice_monitor(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<websocket_voice_monitor_server::VoiceMonitorServer>>
) -> Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(VoiceMonitor {
            server_addr: srv.get_ref().clone(),
            heartbeat: Instant::now(),
            id: 0,
        },
        &req, stream);
    resp
}

pub fn local_communication(
    connected_members: web::Data<RwLock<ConnectedMembers>>,
    srv: web::Data<Addr<websocket_voice_monitor_server::VoiceMonitorServer>>
) {
    std::thread::spawn(move || {
        let listener = std::net::TcpListener::bind(format!("localhost:{}", LOCALPORT)).expect("Cannot create tcp listener");
        println!("WSM | TCP listener started!");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Accepted connection from local");
                    let connected_members_clone = connected_members.clone();
                    let srv = srv.clone();
                    std::thread::spawn(move || {
                        handle_client(stream, connected_members_clone);
                        srv.do_send(websocket_voice_monitor_server::UpdateActors);
                    });
                },
                Err(e) => {
                    println!("Tcp stream listener error: {}", e);
                    break;
                }
            }
        }
        println!("WSM | TCP listener stopped!");
    });
}

fn handle_client(stream: TcpStream, connected_members: web::Data<RwLock<ConnectedMembers>>) {
    let buffer = BufReader::new(stream);
    let mut conn = connected_members.write().unwrap();
    *conn = match serde_json::from_reader(buffer) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    println!("New connected members data:\n{:?}", *conn);
}