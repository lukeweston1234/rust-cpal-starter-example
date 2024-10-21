use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

pub enum PlayerMessage {
    Play,
    Pause,
    Stop,
    Record,
    StopRecord,
    Exit,
}

pub struct PlayerController {
    sender: Sender<PlayerMessage>,
}
impl PlayerController {
    pub fn new(sender: Sender<PlayerMessage>) -> Self {
        Self { sender }
    }
    pub fn play(&self) {
        let _ = self.sender.send(PlayerMessage::Play);
    }
    pub fn pause(&self) {
        let _ = self.sender.send(PlayerMessage::Pause);
    }
    pub fn stop(&self) {
        let _ = self.sender.send(PlayerMessage::Stop);
    }
    pub fn record(&self) {
        let _ = self.sender.send(PlayerMessage::Record);
    }
    pub fn stop_record(&self) {
        let _ = self.sender.send(PlayerMessage::StopRecord);
    }
    pub fn exit(&self) {
        let _ = self.sender.send(PlayerMessage::Exit);
    }
}

pub struct Player {
    receiver: Receiver<PlayerMessage>,
}
impl Player {
    pub fn new(receiver: Receiver<PlayerMessage>) -> Self {
        Self { receiver }
    }
    pub fn play(&self) {}

    pub fn pause(&self) {}

    pub fn stop(&self) {}

    pub fn record(&self) {}

    pub fn stop_record(&self) {}
}

pub fn player() -> (Player, Arc<PlayerController>) {
    let (sender, receiver) = channel::<PlayerMessage>();
    let controller = Arc::new(PlayerController::new(sender));
    let player = Player::new(receiver);

    (player, controller)
}

pub fn run_player(player: Player) {
    for msg in &player.receiver {
        match msg {
            PlayerMessage::Play => player.play(),
            PlayerMessage::Pause => player.pause(),
            PlayerMessage::Stop => player.stop(),
            PlayerMessage::Record => player.record(),
            PlayerMessage::StopRecord => player.stop_record(),
            PlayerMessage::Exit => break,
        }
    }
}
