use std::ops::Deref;
use std::rc::Rc;

use crate::app::components::device_selector::{DeviceSelector, DeviceSelectorModel};
use crate::app::components::EventListener;
use crate::app::models::*;
use crate::app::state::{PlaybackAction, PlaybackEvent, ScreenName, SelectionEvent};
use crate::app::{
    ActionDispatcher, AppAction, AppEvent, AppModel, AppState, BrowserAction, Worker,
};

use super::playback_widget::PlaybackWidget;

pub struct PlaybackModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl PlaybackModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    fn state(&self) -> impl Deref<Target = AppState> + '_ {
        self.app_model.get_state()
    }

    fn go_home(&self) {
        self.dispatcher.dispatch(AppAction::ViewNowPlaying);
        self.dispatcher
            .dispatch(BrowserAction::NavigationPopTo(ScreenName::Home).into());
    }

    fn is_playing(&self) -> bool {
        self.state().playback.is_playing()
    }

    fn is_shuffled(&self) -> bool {
        self.state().playback.is_shuffled()
    }

    fn current_song(&self) -> Option<SongDescription> {
        self.app_model.get_state().playback.current_song()
    }

    fn play_next_song(&self) {
        self.dispatcher.dispatch(PlaybackAction::Next.into());
    }

    fn play_prev_song(&self) {
        self.dispatcher.dispatch(PlaybackAction::Previous.into());
    }

    fn toggle_playback(&self) {
        self.dispatcher.dispatch(PlaybackAction::TogglePlay.into());
    }

    fn toggle_shuffle(&self) {
        self.dispatcher
            .dispatch(PlaybackAction::ToggleShuffle.into());
    }

    fn toggle_repeat(&self) {
        self.dispatcher
            .dispatch(PlaybackAction::ToggleRepeat.into());
    }

    fn seek_to(&self, position: u32) {
        self.dispatcher
            .dispatch(PlaybackAction::Seek(position).into());
    }

    fn set_volume(&self, value: f64) {
        self.dispatcher
            .dispatch(PlaybackAction::SetVolume(value).into())
    }

    pub fn device_selector_model(&self) -> DeviceSelectorModel {
        DeviceSelectorModel::new(self.app_model.clone(), self.dispatcher.box_clone())
    }

    fn view_album(&self, id: String) {
        self.dispatcher.dispatch(AppAction::ViewAlbum(id));
    }

    fn view_artist(&self, id: String) {
        self.dispatcher.dispatch(AppAction::ViewArtist(id));
    }
}

pub struct PlaybackControl {
    model: Rc<PlaybackModel>,
    widget: PlaybackWidget,
    worker: Worker,
    device_selector: DeviceSelector,
}

impl PlaybackControl {
    pub fn new(model: PlaybackModel, widget: PlaybackWidget, worker: Worker) -> Self {
        let model = Rc::new(model);

        widget.connect_play_pause(clone!(
            #[weak]
            model,
            move || model.toggle_playback()
        ));
        widget.connect_next(clone!(
            #[weak]
            model,
            move || model.play_next_song()
        ));
        widget.connect_prev(clone!(
            #[weak]
            model,
            move || model.play_prev_song()
        ));
        widget.connect_shuffle(clone!(
            #[weak]
            model,
            move || model.toggle_shuffle()
        ));
        widget.connect_repeat(clone!(
            #[weak]
            model,
            move || model.toggle_repeat()
        ));
        widget.connect_seek(clone!(
            #[weak]
            model,
            move |position| model.seek_to(position)
        ));
        widget.connect_now_playing_clicked(clone!(
            #[weak]
            model,
            move || model.go_home()
        ));
        widget.connect_volume_changed(clone!(
            #[weak]
            model,
            move |value| model.set_volume(value)
        ));

        widget.connect_queue_clicked(clone!(
            #[weak]
            model,
            move || model.go_home()
        ));

        widget.connect_album_clicked(clone!(
            #[weak]
            model,
            move || {
                if let Some(song) = model.current_song() {
                    model.view_album(song.album.id);
                }
            }
        ));

        widget.connect_artist_clicked(clone!(
            #[weak]
            model,
            move || {
                if let Some(song) = model.current_song() {
                    if let Some(artist) = song.artists.first() {
                        model.view_artist(artist.id.clone());
                    }
                }
            }
        ));

        let device_selector = DeviceSelector::new(
            widget.device_selector_widget(),
            model.device_selector_model(),
        );

        Self {
            model,
            widget,
            worker,
            device_selector,
        }
    }

    fn update_repeat(&self, mode: &RepeatMode) {
        self.widget.set_repeat_mode(*mode);
    }

    fn update_shuffled(&self) {
        self.widget.set_shuffled(self.model.is_shuffled());
    }

    fn update_playing(&self) {
        let is_playing = self.model.is_playing();
        self.widget.set_playing(is_playing);
    }

    fn update_current_info(&self) {
        if let Some(song) = self.model.current_song() {
            self.widget
                .set_title_and_artist(&song.title, &song.artists_name());
            self.widget.set_song_duration(Some(song.duration as f64));
            if let Some(url) = song.art {
                self.widget.set_artwork_from_url(url, &self.worker);
            }
        } else {
            self.widget.reset_info();
        }
    }

    fn sync_seek(&self, pos: u32) {
        self.widget.set_seek_position(pos as f64);
    }
}

impl EventListener for PlaybackControl {
    fn on_event(&mut self, event: &AppEvent) {
        self.device_selector.on_event(event);
        match event {
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackPaused)
            | AppEvent::PlaybackEvent(PlaybackEvent::PlaybackResumed) => {
                self.update_playing();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::RepeatModeChanged(mode)) => {
                self.update_repeat(mode);
            }
            AppEvent::PlaybackEvent(PlaybackEvent::ShuffleChanged(_)) => {
                self.update_shuffled();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::TrackChanged(_)) => {
                self.update_current_info();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::PlaybackStopped) => {
                self.update_playing();
                self.update_current_info();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::SeekSynced(pos))
            | AppEvent::PlaybackEvent(PlaybackEvent::TrackSeeked(pos)) => {
                self.sync_seek(*pos);
            }
            AppEvent::SelectionEvent(SelectionEvent::SelectionModeChanged(active)) => {
                self.widget.set_seekbar_visible(!active);
            }
            AppEvent::PlaybackEvent(PlaybackEvent::VolumeSet(value)) => {
                self.widget.set_volume(*value)
            }
            _ => {}
        }
    }
}
