use std::rc::Rc;

use futures::channel::mpsc::UnboundedSender;

use crate::api::{oauth2::RiffOauthClient, TokenStore};
use crate::app::components::EventListener;
use crate::app::models::ConnectDevice;
use crate::app::state::{LoginAction, LoginEvent, LoginStartedEvent, PlaybackAction, PlaybackEvent};
use crate::app::{ActionDispatcher, AppAction, AppEvent, AppModel, SongsSource};
use crate::connect::ConnectCommand;

enum CurrentlyPlaying {
    WithSource {
        source: SongsSource,
        offset: usize,
        song: String,
    },
    Songs {
        songs: Vec<String>,
        offset: usize,
    },
}

impl CurrentlyPlaying {
    fn song_id(&self) -> &String {
        match self {
            Self::WithSource { song, .. } => song,
            Self::Songs { songs, offset } => &songs[*offset],
        }
    }
}

pub struct PlayerNotifier {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
    sender: UnboundedSender<AppAction>,
    connect_command_sender: UnboundedSender<ConnectCommand>,
    token_store: TokenStore,
}

impl PlayerNotifier {
    pub fn new(
        app_model: Rc<AppModel>,
        dispatcher: Box<dyn ActionDispatcher>,
        sender: UnboundedSender<AppAction>,
        connect_command_sender: UnboundedSender<ConnectCommand>,
        token_store: TokenStore,
    ) -> Self {
        Self {
            app_model,
            dispatcher,
            sender,
            connect_command_sender,
            token_store,
        }
    }

    fn is_playing(&self) -> bool {
        self.app_model.get_state().playback.is_playing()
    }

    fn currently_playing(&self) -> Option<CurrentlyPlaying> {
        let state = self.app_model.get_state();
        let song = state.playback.current_song_id()?;
        let offset = state.playback.current_song_index()?;
        let source = state.playback.current_source().cloned();
        let result = match source {
            Some(source) if source.has_spotify_uri() => CurrentlyPlaying::WithSource {
                source,
                offset,
                song,
            },
            _ => CurrentlyPlaying::Songs {
                songs: state.playback.songs().map_collect(|s| s.id),
                offset,
            },
        };
        Some(result)
    }

    fn notify_login(&self, event: &LoginEvent) {
        match event {
            LoginEvent::LoginStarted(LoginStartedEvent::InitLogin) => {
                let sender = self.sender.clone();
                let token_store = self.token_store.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async {
                        let oauth = RiffOauthClient::new(token_store);
                        match oauth.spawn_authcode_listener(|| {}).await {
                            Ok(challenge) => {
                                sender
                                    .unbounded_send(
                                        LoginAction::OpenLoginUrl(challenge.auth_url.clone())
                                            .into(),
                                    )
                                    .unwrap();
                                match oauth.exchange_authcode(challenge).await {
                                    Ok(_creds) => {
                                        info!("Login successful");
                                        sender
                                            .unbounded_send(
                                                LoginAction::SetLoginSuccess(String::new()).into(),
                                            )
                                            .unwrap();
                                    }
                                    Err(e) => {
                                        error!("OAuth exchange failed: {}", e);
                                        sender
                                            .unbounded_send(LoginAction::SetLoginFailure.into())
                                            .unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                error!("OAuth init failed: {}", e);
                                sender
                                    .unbounded_send(LoginAction::SetLoginFailure.into())
                                    .unwrap();
                            }
                        }
                    });
                });
            }
            LoginEvent::LoginStarted(LoginStartedEvent::CompleteLogin) => {
                error!("CompleteLogin event received but no pending login");
            }
            LoginEvent::LoginStarted(LoginStartedEvent::Restore) => {
                let sender = self.sender.clone();
                let token_store = self.token_store.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async {
                        let oauth = RiffOauthClient::new(token_store);
                        match oauth.get_valid_token().await {
                            Ok(_creds) => {
                                info!("Restored session");
                                sender
                                    .unbounded_send(
                                        LoginAction::SetLoginSuccess(String::new()).into(),
                                    )
                                    .unwrap();
                            }
                            Err(_) => {
                                debug!("No saved session to restore");
                                sender
                                    .unbounded_send(LoginAction::ShowLogin.into())
                                    .unwrap();
                            }
                        }
                    });
                });
            }
            LoginEvent::FreshTokenRequested => {
                let sender = self.sender.clone();
                let token_store = self.token_store.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async {
                        let oauth = RiffOauthClient::new(token_store);
                        match oauth.refresh_token_at_expiry().await {
                            Ok(_) => {
                                sender
                                    .unbounded_send(LoginAction::TokenRefreshed.into())
                                    .unwrap();
                            }
                            Err(_) => {
                                sender
                                    .unbounded_send(LoginAction::SetLoginFailure.into())
                                    .unwrap();
                            }
                        }
                    });
                });
            }
            LoginEvent::LogoutCompleted => {
                let token_store = self.token_store.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async {
                        token_store.clear().await;
                    });
                });
            }
            LoginEvent::LoginCompleted => {
                let sender = self.sender.clone();
                let api = self.app_model.get_spotify();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    rt.block_on(async {
                        match api.list_available_devices().await {
                            Ok(devices) => {
                                // Select device tagged as active
                                if let Some(device) = devices.iter().find(|d| d.is_active) {
                                    sender
                                        .unbounded_send(
                                            PlaybackAction::SwitchDevice(Some(device.clone())).into(),
                                        )
                                        .unwrap();
                                }
                                sender
                                    .unbounded_send(
                                        PlaybackAction::SetAvailableDevices(devices).into(),
                                    )
                                    .unwrap();
                            }
                            Err(e) => {
                                debug!("Failed to fetch devices on startup: {e}");
                            }
                        }
                    });
                });
            }
            _ => {}
        }
    }

    fn notify_connect_player(&self, event: &PlaybackEvent) {
        let event = event.clone();
        let currently_playing = self.currently_playing();
        let command = match event {
            PlaybackEvent::TrackChanged(_) | PlaybackEvent::SourceChanged => {
                match currently_playing {
                    Some(CurrentlyPlaying::WithSource {
                        source,
                        offset,
                        song,
                    }) => Some(ConnectCommand::PlayerLoadInContext {
                        source,
                        offset,
                        song,
                    }),
                    Some(CurrentlyPlaying::Songs { songs, offset }) => {
                        Some(ConnectCommand::PlayerLoad { songs, offset })
                    }
                    None => None,
                }
            }
            PlaybackEvent::TrackSeeked(position) => {
                Some(ConnectCommand::PlayerSeek(position as usize))
            }
            PlaybackEvent::PlaybackPaused => Some(ConnectCommand::PlayerPause),
            PlaybackEvent::PlaybackResumed => Some(ConnectCommand::PlayerResume),
            PlaybackEvent::VolumeSet(volume) => Some(ConnectCommand::PlayerSetVolume(
                (volume * 100f64).trunc() as u8,
            )),
            PlaybackEvent::RepeatModeChanged(mode) => Some(ConnectCommand::PlayerRepeat(mode)),
            PlaybackEvent::ShuffleChanged(shuffled) => {
                Some(ConnectCommand::PlayerShuffle(shuffled))
            }
            _ => None,
        };

        if let Some(command) = command {
            self.connect_command_sender.unbounded_send(command).unwrap();
        }
    }

    fn switch_device(&mut self, device: &Option<ConnectDevice>) {
        match device {
            Some(device) => {
                self.send_command_to_connect_player(ConnectCommand::SetDevice(device.id.clone()));
                self.notify_connect_player(&PlaybackEvent::SourceChanged);
            }
            None => {
                self.send_command_to_connect_player(ConnectCommand::PlayerStop);
            }
        }
    }

    fn send_command_to_connect_player(&self, command: ConnectCommand) {
        self.connect_command_sender.unbounded_send(command).unwrap();
    }

}

impl EventListener for PlayerNotifier {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::LoginEvent(event) => self.notify_login(event),
            AppEvent::PlaybackEvent(PlaybackEvent::SwitchedDevice(d)) => self.switch_device(d),
            AppEvent::PlaybackEvent(event) => self.notify_connect_player(event),
            _ => {}
        }
    }
}
