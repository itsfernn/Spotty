use crate::{
    app::state::{AppAction, AppEvent, UpdatableState},
    settings::SpottySettings,
};

#[derive(Clone, Debug)]
pub enum SettingsAction {
    ChangeSettings,
}

impl From<SettingsAction> for AppAction {
    fn from(settings_action: SettingsAction) -> Self {
        Self::SettingsAction(settings_action)
    }
}

#[derive(Clone, Debug)]
pub enum SettingsEvent {}

impl From<SettingsEvent> for AppEvent {
    fn from(settings_event: SettingsEvent) -> Self {
        Self::SettingsEvent(settings_event)
    }
}

#[derive(Default)]
pub struct SettingsState {
    // Probably shouldn't be stored, the source of truth is GSettings anyway
    pub settings: SpottySettings,
}

impl UpdatableState for SettingsState {
    type Action = SettingsAction;
    type Event = AppEvent;

    fn update_with(&mut self, action: std::borrow::Cow<Self::Action>) -> Vec<Self::Event> {
        match action.into_owned() {
            SettingsAction::ChangeSettings => {
                self.settings = SpottySettings::new_from_gsettings().unwrap_or_default();
                vec![]
            }
        }
    }
}
