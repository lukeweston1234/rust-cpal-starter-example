use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use crate::mixer::{Mixer, MixerController, MixerState};
use crate::recorder::RecorderController;

#[derive(Debug)]
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
        println!("In play!");
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
    controller_receiver: Receiver<PlayerMessage>,
    mixer_controller: Arc<MixerController>,
    recorder_controller: Arc<RecorderController>,
}
impl Player {
    pub fn new(
        controller_receiver: Receiver<PlayerMessage>,
        mixer_controller: Arc<MixerController>,
        recorder_controller: Arc<RecorderController>,
    ) -> Self {
        Self {
            controller_receiver,
            mixer_controller,
            recorder_controller,
        }
    }
    pub fn play(&self) {
        self.mixer_controller
            .set_mixer_state(MixerState::PlayingLooping);
    }

    pub fn pause(&self) {
        self.mixer_controller.set_mixer_state(MixerState::Paused);
    }

    pub fn stop(&self) {
        self.mixer_controller.set_mixer_state(MixerState::Stopped);
    }

    pub fn record(&self) {
        self.recorder_controller.start_recording();
    }

    pub fn stop_record(&self) {
        self.recorder_controller.stop_recording();
    }
}

pub fn player(
    mixer_controller: Arc<MixerController>,
    recorder_controller: Arc<RecorderController>,
) -> (Player, Arc<PlayerController>) {
    let (sender, receiver) = channel::<PlayerMessage>();
    let player_controller = Arc::new(PlayerController::new(sender));
    let player = Player::new(receiver, mixer_controller, recorder_controller);

    (player, player_controller)
}

pub fn run_player(player: Player) {
    std::thread::spawn(move || loop {
        match player.controller_receiver.recv() {
            Ok(message) => match message {
                PlayerMessage::Play => player.play(),
                PlayerMessage::Pause => player.pause(),
                PlayerMessage::Stop => player.stop(),
                PlayerMessage::Record => player.record(),
                PlayerMessage::StopRecord => player.stop_record(),
                PlayerMessage::Exit => break,
            },
            Err(_) => break,
        }
    });
}
