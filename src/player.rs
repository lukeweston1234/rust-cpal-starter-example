use std::sync::mpsc::{channel, Receiver, Sender};

pub enum PlayerCommand {
    Playing,
    Paused,
    Stopped,
    Recording,
    Exit,
}

pub struct PlayerController {
    sender: Sender<PlayerCommand>,
}
impl PlayerController {
    pub fn play(&self) {
        self.sender.send(PlayerCommand::Playing);
    }
    pub fn pause(&self) {
        self.sender.send(PlayerCommand::Paused);
    }
    pub fn stop(&self) {
        self.sender.send(PlayerCommand::Stopped);
    }
    pub fn record(&self) {
        self.sender.send(PlayerCommand::Recording);
    }
    pub fn exit(&self) {
        self.sender.send(PlayerCommand::Exit);
    }
}

pub fn get_controller() -> (PlayerController, Receiver<PlayerCommand>) {
    let (sender, receiver) = channel::<PlayerCommand>();
    let controller = PlayerController { sender };

    (controller, receiver)
}
