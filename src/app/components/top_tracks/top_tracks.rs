use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::TopTracksModel;
use crate::app::components::utils::wrap_flowbox_item;
use crate::app::components::{ArtistWidget, Component, EventListener, Playlist};
use crate::app::models::ArtistModel;
use crate::app::state::LoginEvent;
use crate::app::{AppEvent, BrowserEvent, Worker};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/itsfernn/Spotty/components/top_tracks.ui")]
    pub struct TopTracksWidget {
        #[template_child]
        pub song_list: TemplateChild<gtk::ListView>,

        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub artist_results: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TopTracksWidget {
        const NAME: &'static str = "TopTracksWidget";
        type Type = super::TopTracksWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TopTracksWidget {}
    impl WidgetImpl for TopTracksWidget {}
    impl BoxImpl for TopTracksWidget {}
}

glib::wrapper! {
    pub struct TopTracksWidget(ObjectSubclass<imp::TopTracksWidget>) @extends gtk::Widget, gtk::Box;
}

impl TopTracksWidget {
    fn new() -> Self {
        glib::Object::new()
    }

    fn connect_bottom_edge<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp()
            .scrolled_window
            .connect_edge_reached(move |_, pos| {
                if let gtk::PositionType::Bottom = pos {
                    f()
                }
            });
    }

    fn song_list_widget(&self) -> &gtk::ListView {
        self.imp().song_list.as_ref()
    }

    fn bind_artists<F>(&self, worker: Worker, store: &gio::ListStore, on_artist_pressed: F)
    where
        F: Fn(String) + Clone + 'static,
    {
        self.imp()
            .artist_results
            .bind_model(Some(store), move |item| {
                wrap_flowbox_item(item, |artist_model| {
                    let f = on_artist_pressed.clone();
                    let artist = ArtistWidget::for_model(artist_model, worker.clone());
                    artist.connect_artist_pressed(clone!(
                        #[weak]
                        artist_model,
                        move || {
                            f(artist_model.id());
                        }
                    ));
                    artist
                })
            });
    }
}

pub struct TopTracks {
    widget: TopTracksWidget,
    model: Rc<TopTracksModel>,
    children: Vec<Box<dyn EventListener>>,
    artist_results_model: gio::ListStore,
}

impl TopTracks {
    pub fn new(model: Rc<TopTracksModel>, worker: Worker) -> Self {
        let widget = TopTracksWidget::new();

        widget.connect_bottom_edge(clone!(
            #[weak]
            model,
            move || {
                model.load_more();
            }
        ));

        let artist_results_model = gio::ListStore::new::<ArtistModel>();

        widget.bind_artists(
            worker.clone(),
            &artist_results_model,
            clone!(
                #[weak]
                model,
                move |id| {
                    model.view_artist(id);
                }
            ),
        );

        let playlist = Playlist::new(widget.song_list_widget().clone(), model.clone(), worker);

        Self {
            widget,
            model,
            children: vec![Box::new(playlist)],
            artist_results_model,
        }
    }

    fn update_artists(&self) {
        if let Some(artists) = self.model.get_top_artists() {
            self.artist_results_model.remove_all();
            for artist in artists.iter() {
                self.artist_results_model
                    .append(&ArtistModel::new(&artist.name, &artist.photo, &artist.id));
            }
        }
    }
}

impl Component for TopTracks {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }

    fn get_children(&mut self) -> Option<&mut Vec<Box<dyn EventListener>>> {
        Some(&mut self.children)
    }
}

impl EventListener for TopTracks {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Started | AppEvent::LoginEvent(LoginEvent::LoginCompleted) => {
                self.model.load_initial();
            }
            AppEvent::BrowserEvent(BrowserEvent::TopArtistsUpdated) => {
                self.update_artists();
            }
            _ => {}
        }
        self.broadcast_event(event);
    }
}
