//! What the companion is doing, for every window (avatar R1), and its look and floating window in
//! the submodules.

mod look;
mod place;

pub use look::{AvatarLook, AvatarManifest, AvatarProblem, AvatarSource, Expression, Motion};
pub use place::{FloatingSettings, Screen, place};

use crate::Core;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tokio::sync::watch;
use ts_rs::TS;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Activity {
    #[default]
    Idle,
    Listening,
    Thinking,
    Speaking,
    Sleeping,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Mood {
    #[default]
    Neutral,
    Happy,
    Playful,
    Concerned,
    Sad,
    Surprised,
}

impl Mood {
    /// A marker's word, ignoring case; an unknown word is neutral (avatar FR-010).
    pub(crate) fn named(word: &str) -> Self {
        serde_json::from_value(Value::String(word.to_ascii_lowercase())).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct AvatarState {
    pub activity: Activity,
    pub mood: Mood,
    /// Characters of reply text streamed this turn; the mouth follows how fast it grows.
    pub voice: u32,
    /// Bumped whenever the app writes the manifest; windows read it again when it changes (avatar FR-029).
    pub look: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) enum TurnPhase {
    #[default]
    None,
    Thinking,
    Speaking,
}

#[derive(Debug, Clone, Copy, Default)]
struct Inputs {
    turn: TurnPhase,
    listening: bool,
    dreaming: bool,
    mood: Mood,
    /// The last marker of the turn running now.
    turn_mood: Option<Mood>,
    voice: u32,
    look: u32,
}

/// avatar FR-004: a turn wins over listening, listening over sleeping.
fn activity(i: &Inputs) -> Activity {
    match (i.turn, i.listening, i.dreaming) {
        (TurnPhase::Thinking, ..) => Activity::Thinking,
        (TurnPhase::Speaking, ..) => Activity::Speaking,
        (TurnPhase::None, true, _) => Activity::Listening,
        (TurnPhase::None, false, true) => Activity::Sleeping,
        _ => Activity::Idle,
    }
}

/// The inputs and the state the shell broadcasts; a watch channel keeps only the latest (avatar R1).
pub(crate) struct AvatarRuntime {
    inputs: Mutex<Inputs>,
    state: watch::Sender<AvatarState>,
    /// The chosen character was gone and the default replaced it, until the user chooses again (avatar FR-032).
    gone: Mutex<bool>,
}

impl Default for AvatarRuntime {
    fn default() -> Self {
        Self {
            inputs: Mutex::default(),
            state: watch::Sender::new(AvatarState::default()),
            gone: Mutex::default(),
        }
    }
}

impl AvatarRuntime {
    fn update(&self, change: impl FnOnce(&mut Inputs)) {
        let mut inputs = crate::chat::lock(&self.inputs);
        change(&mut inputs);
        let next = AvatarState {
            activity: activity(&inputs),
            mood: inputs.mood,
            voice: inputs.voice,
            look: inputs.look,
        };
        self.state.send_if_modified(|s| {
            let changed = *s != next;
            *s = next;
            changed
        });
    }

    pub(crate) fn turn(&self, phase: TurnPhase) {
        self.update(|i| i.turn = phase);
    }

    pub(crate) fn mood_read(&self, mood: Mood) {
        self.update(|i| {
            i.turn_mood = Some(mood);
            i.mood = mood;
        });
    }

    pub(crate) fn spoke(&self, chars: usize) {
        self.update(|i| {
            i.turn = TurnPhase::Speaking;
            i.voice = i.voice.saturating_add(chars as u32);
        });
    }

    pub(crate) fn dreaming(&self, on: bool) {
        self.update(|i| i.dreaming = on);
    }
}

/// Ends the turn's part of the state however the turn ends; the mood is the turn's last marker (avatar FR-011).
pub(crate) struct Turn<'a>(&'a AvatarRuntime);

impl<'a> Turn<'a> {
    pub(crate) fn start(avatar: &'a AvatarRuntime) -> Self {
        avatar.update(|i| {
            *i = Inputs {
                turn: TurnPhase::Thinking,
                listening: false,
                turn_mood: None,
                voice: 0,
                ..*i
            }
        });
        Self(avatar)
    }
}

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        self.0.update(|i| {
            i.turn = TurnPhase::None;
            i.mood = i.turn_mood.take().unwrap_or_default();
        });
    }
}

impl Core {
    pub fn avatar_state(&self) -> AvatarState {
        *self.avatar.state.borrow()
    }

    pub fn subscribe_avatar(&self) -> watch::Receiver<AvatarState> {
        self.avatar.state.subscribe()
    }

    /// avatar FR-004: the message box has text and focus. A turn clears it and ignores it while running.
    pub fn set_listening(&self, listening: bool) {
        self.avatar.update(|i| {
            if i.turn == TurnPhase::None {
                i.listening = listening;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_activity_follows_the_turn_then_listening_then_sleep() {
        use TurnPhase as T;
        let cases = [
            (T::Thinking, true, true, Activity::Thinking),
            (T::Speaking, true, false, Activity::Speaking),
            (T::None, true, true, Activity::Listening),
            (T::None, false, true, Activity::Sleeping),
            (T::None, false, false, Activity::Idle),
        ];
        for (turn, listening, dreaming, expected) in cases {
            let inputs = Inputs {
                turn,
                listening,
                dreaming,
                ..Inputs::default()
            };
            assert_eq!(
                activity(&inputs),
                expected,
                "{turn:?} {listening} {dreaming}"
            );
        }
    }

    #[test]
    fn mood_words_ignore_case_and_unknown_ones_are_neutral() {
        assert_eq!(Mood::named("Happy"), Mood::Happy);
        assert_eq!(Mood::named("SURPRISED"), Mood::Surprised);
        assert_eq!(Mood::named("ecstatic"), Mood::Neutral);
    }
}
