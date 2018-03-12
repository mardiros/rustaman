use gtk::{self, IconSize, Orientation, ReliefStyle};
use gtk::prelude::*;
use sourceview::{self, LanguageManager, StyleSchemeManager, View as SourceView};
use sourceview::prelude::*;
use relm::{Relm, Update, Widget};

use super::super::helpers::path;

#[derive(Msg)]
pub enum Msg {
}

pub struct EnvironEditor {
    notebook: gtk::Notebook,
    environ_view: SourceView,
    //relm: Relm<EnvironEditor>,
}

impl Update for EnvironEditor {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {}
    }
}

impl Widget for EnvironEditor {
    type Root = gtk::Notebook;

    fn root(&self) -> Self::Root {
        self.notebook.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        fn create_tab(title: &str) -> gtk::Box {
            let close_image =
                gtk::Image::new_from_icon_name("window-close", IconSize::Button.into());
            let button = gtk::Button::new();
            let label = gtk::Label::new(title);
            let tab = gtk::Box::new(Orientation::Horizontal, 0);

            button.set_relief(ReliefStyle::None);
            button.set_focus_on_click(false);
            button.add(&close_image);

            tab.pack_start(&label, false, false, 0);
            tab.pack_start(&button, false, false, 0);
            tab.show_all();
            tab
        }

        info!("Creating Environ widget");

        let notebook = gtk::Notebook::new();
        notebook.set_hexpand(false);
        notebook.set_vexpand(true);
        notebook.set_margin_top(10);
        notebook.set_margin_left(10);

        let langmngr = LanguageManager::get_default().unwrap();
        let mut search_path = langmngr.get_search_path();
        search_path.push(path::config_dir().unwrap().to_str().unwrap().to_owned());
        let path2: Vec<&str> = search_path.iter().map(|path| path.as_str()).collect();
        langmngr.set_search_path(path2.as_slice());
        let lang = langmngr.get_language("yaml").unwrap();

        let stylemngr = StyleSchemeManager::get_default().unwrap();
        let style = stylemngr.get_scheme("solarized-dark").unwrap();

        let buffer = sourceview::Buffer::new_with_language(&lang);
        buffer.set_style_scheme(&style);

        let environ_view = SourceView::new_with_buffer(&buffer);
        environ_view.set_show_line_numbers(true);

        environ_view.set_hexpand(true);
        environ_view.set_vexpand(true);
        environ_view.show();

        let tab = create_tab("Dev");
        let _index = notebook.append_page(&environ_view, Some(&tab));

        let tab = gtk::Box::new(Orientation::Horizontal, 0);
        let btn = gtk::Button::new();
        btn.set_label("+");
        tab.pack_start(&btn, false, false, 0);
        tab.show_all();

        let empty = gtk::Box::new(Orientation::Horizontal, 0);
        empty.show();
        let _index = notebook.append_page(&empty, Some(&tab));

        /*
        button.connect_clicked(move |_| {
            let index = notebook_clone.page_num(&widget)
                                      .expect("Couldn't get page_num from notebook_clone");
            notebook_clone.remove_page(Some(index));
        });


*/
        notebook.set_hexpand(true);
        notebook.set_vexpand(false);
        notebook.set_size_request(800, 480);
        notebook.show();
        EnvironEditor {
            notebook: notebook,
            environ_view: environ_view,
            //relm: relm.clone(),
        }
    }
}
