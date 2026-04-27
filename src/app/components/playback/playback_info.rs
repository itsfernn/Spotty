use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/itsfernn/Spotty/components/playback_info.ui")]
    pub struct PlaybackInfoWidget {
        #[template_child]
        pub playing_image: TemplateChild<gtk::Picture>,

        #[template_child]
        pub title_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub artist_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackInfoWidget {
        const NAME: &'static str = "PlaybackInfoWidget";
        type Type = super::PlaybackInfoWidget;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackInfoWidget {}
    impl WidgetImpl for PlaybackInfoWidget {}
    impl BoxImpl for PlaybackInfoWidget {}
}

glib::wrapper! {
    pub struct PlaybackInfoWidget(ObjectSubclass<imp::PlaybackInfoWidget>) @extends gtk::Widget, gtk::Box;
}

impl PlaybackInfoWidget {
    fn get_title_label(&self) -> gtk::Label {
        self.imp()
            .title_button
            .child()
            .and_then(|c| c.downcast::<gtk::Label>().ok())
            .unwrap()
    }

    fn get_artist_label(&self) -> gtk::Label {
        self.imp()
            .artist_button
            .child()
            .and_then(|c| c.downcast::<gtk::Label>().ok())
            .unwrap()
    }

    pub fn set_title_and_artist(&self, title: &str, artist: &str) {
        let title = glib::markup_escape_text(title);
        let artist = glib::markup_escape_text(artist);
        self.get_title_label()
            .set_markup(&format!("<b>{}</b>", title.as_str()));
        self.get_artist_label().set_label(artist.as_str());
    }

    pub fn reset_info(&self) {
        let widget = self.imp();
        self.get_title_label()
            // translators: Short text displayed instead of a song title when nothing plays
            .set_label(&gettext("No song playing"));
        self.get_artist_label().set_label("");
        widget
            .playing_image
            .set_paintable(None::<gdk::Paintable>.as_ref());
    }

    pub fn set_info_visible(&self, visible: bool) {
        self.imp().title_button.set_visible(visible);
        self.imp().artist_button.set_visible(visible);
    }

    pub fn set_artwork(&self, pixbuf: &gdk_pixbuf::Pixbuf) {
        let texture = gdk::Texture::for_pixbuf(pixbuf);
        self.imp().playing_image.set_paintable(Some(&texture));
    }

    pub fn connect_album_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().title_button.connect_clicked(move |_| f());
    }

    pub fn connect_artist_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        self.imp().artist_button.connect_clicked(move |_| f());
    }

    pub fn connect_now_playing_clicked<F>(&self, f: F)
    where
        F: Fn() + 'static,
    {
        let controller = gtk::GestureClick::new();
        controller.connect_pressed(move |_, _, _, _| f());
        self.imp().playing_image.add_controller(controller);
    }
}
