use crate::app::{
    components::EventListener,
    models::RepeatMode,
    state::{PlaybackAction, PlaybackEvent},
    AppAction, AppEvent,
};
use gio::prelude::SettingsExt;
use libadwaita::ColorScheme;

const SETTINGS: &str = "dev.diegovsky.Riff";

#[derive(Clone, Debug, Default)]
pub struct WindowGeometry {
    pub width: i32,
    pub height: i32,
    pub is_maximized: bool,
}

impl WindowGeometry {
    pub fn new_from_gsettings() -> Self {
        let settings = gio::Settings::new(SETTINGS);
        Self {
            width: settings.int("window-width"),
            height: settings.int("window-height"),
            is_maximized: settings.boolean("window-is-maximized"),
        }
    }

    pub fn save(&self) -> Option<()> {
        let settings = gio::Settings::new(SETTINGS);
        settings.delay();
        settings.set_int("window-width", self.width).ok()?;
        settings.set_int("window-height", self.height).ok()?;
        settings
            .set_boolean("window-is-maximized", self.is_maximized)
            .ok()?;
        settings.apply();
        Some(())
    }
}

#[derive(Debug, Clone)]
pub struct RiffSettings {
    pub theme_preference: ColorScheme,
    pub volume: f64,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub window: WindowGeometry,
}

impl RiffSettings {
    fn load_player_settings(settings: &gio::Settings) -> (f64, bool, RepeatMode) {
        let volume = settings.double("volume");
        let shuffle = settings.boolean("shuffle");
        let repeat = match settings.string("repeat").as_str() {
            "song" => RepeatMode::Song,
            "playlist" => RepeatMode::Playlist,
            "none" | _ => RepeatMode::None,
        };
        (volume, shuffle, repeat)
    }

    pub fn actions(&self) -> Vec<AppAction> {
        use PlaybackAction::*;
        vec![
            SetVolume(self.volume).into(),
            SetShuffled(self.shuffle).into(),
            SetRepeatMode(self.repeat).into(),
        ]
    }
}

// Application settings
impl RiffSettings {
    pub fn new_from_gsettings() -> Option<Self> {
        let settings = gio::Settings::new(SETTINGS);
        let theme_preference = match settings.enum_("theme-preference") {
            0 => Some(ColorScheme::ForceLight),
            1 => Some(ColorScheme::ForceDark),
            2 => Some(ColorScheme::Default),
            _ => None,
        }?;
        let (volume, shuffle, repeat) = Self::load_player_settings(&settings);
        Some(Self {
            theme_preference,
            volume,
            shuffle,
            repeat,
            window: WindowGeometry::new_from_gsettings(),
        })
    }
}

impl Default for RiffSettings {
    fn default() -> Self {
        Self {
            theme_preference: ColorScheme::PreferDark,
            volume: 0.7,
            shuffle: false,
            repeat: RepeatMode::None,
            window: Default::default(),
        }
    }
}

/// Observes some app state changes and records them into GSettings.
pub struct StateTracker {
    settings: gio::Settings,
}

type GResult = Result<(), glib::error::BoolError>;
impl StateTracker {
    pub fn new_from_gsettings() -> Self {
        Self {
            settings: gio::Settings::new(SETTINGS),
        }
    }
    fn on_playback_event(&self, event: &PlaybackEvent) -> GResult {
        use PlaybackEvent::*;
        match event {
            VolumeSet(volume) => self.settings.set_double("volume", *volume)?,
            ShuffleChanged(shuffle) => self.settings.set_boolean("shuffle", *shuffle)?,
            RepeatModeChanged(repeat) => self.settings.set_string(
                "repeat",
                match *repeat {
                    RepeatMode::Song => "song",
                    RepeatMode::Playlist => "playlist",
                    RepeatMode::None => "none",
                },
            )?,
            _ => (),
        }
        Ok(())
    }

    fn handle_event(&self, event: &AppEvent) -> GResult {
        match event {
            AppEvent::PlaybackEvent(event) => self.on_playback_event(event)?,
            _ => (),
        }
        Ok(())
    }
}

impl EventListener for StateTracker {
    fn on_event(&mut self, event: &AppEvent) {
        if let Err(e) = self.handle_event(event) {
            error!("Trying to update gsettings: {e}")
        }
    }
}
