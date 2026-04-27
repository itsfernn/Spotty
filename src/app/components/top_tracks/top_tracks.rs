use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::rc::Rc;

use super::TopTracksModel;
use crate::app::components::{Component, EventListener, Playlist};
use crate::app::state::LoginEvent;
use crate::app::{AppEvent, Worker};
use libadwaita::subclass::prelude::BinImpl;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/itsfernn/Spotty/components/top_tracks.ui")]
    pub struct TopTracksWidget {
        #[template_child]
        pub song_list: TemplateChild<gtk::ListView>,

        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TopTracksWidget {
        const NAME: &'static str = "TopTracksWidget";
        type Type = super::TopTracksWidget;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TopTracksWidget {}
    impl WidgetImpl for TopTracksWidget {}
    impl BinImpl for TopTracksWidget {}
}

glib::wrapper! {
    pub struct TopTracksWidget(ObjectSubclass<imp::TopTracksWidget>) @extends gtk::Widget, libadwaita::Bin;
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
}

pub struct TopTracks {
    widget: TopTracksWidget,
    model: Rc<TopTracksModel>,
    children: Vec<Box<dyn EventListener>>,
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

        let playlist = Playlist::new(widget.song_list_widget().clone(), model.clone(), worker);

        Self {
            widget,
            model,
            children: vec![Box::new(playlist)],
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
            _ => {}
        }
        self.broadcast_event(event);
    }
}
