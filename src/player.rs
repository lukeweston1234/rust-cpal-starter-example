use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use crate::mixer::{Mixer, MixerController, MixerState};
use crate::recorder::RecorderController;

#[derive(Debug)]
pub enum PlayerMessage {
    Play,
    Pause,
    Stop,
    IncrementTotalClips,
    DecrementTotalClips,
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
    pub fn on_clip_add(&self) {
        let _ = self.sender.send(PlayerMessage::IncrementTotalClips);
    }
    pub fn on_clip_remove(&self) {
        let _ = self.sender.send(PlayerMessage::DecrementTotalClips);
    }
}

pub struct Player {
    controller_receiver: Receiver<PlayerMessage>,
    mixer_controller: Arc<MixerController>,
    recorder_controller: Arc<RecorderController>,
    total_clips: usize,
    maximum_clips: usize,
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
            total_clips: 0,
            maximum_clips: 8,
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
    pub fn increment_total_clips(&mut self) {
        if self.total_clips >= self.maximum_clips {
            self.stop_record();
            return;
        }
        self.total_clips += 1;
    }
    pub fn decrement_total_clips(&mut self) {
        self.total_clips -= 1;
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

pub fn run_player(mut player: Player) {
    std::thread::spawn(move || loop {
        match player.controller_receiver.try_recv() {
            Ok(message) => match message {
                PlayerMessage::Play => player.play(),
                PlayerMessage::Pause => player.pause(),
                PlayerMessage::Stop => player.stop(),
                PlayerMessage::Record => player.record(),
                PlayerMessage::StopRecord => player.stop_record(),
                PlayerMessage::IncrementTotalClips => player.increment_total_clips(),
                PlayerMessage::DecrementTotalClips => player.decrement_total_clips(),
                PlayerMessage::Exit => break,
            },
            Err(_) => break,
        }
    });
}
