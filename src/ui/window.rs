// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, gtk::gio, ComponentParts, ComponentSender};

use crate::helpers::{self, http};
use crate::models::Request;
use crate::ui::environments::EnvironmentsMsg;
use crate::ui::menu_item::MenuItemOutput;
use crate::ui::request_editor::{RequestMsg, RequestOutput};
use crate::ui::response_body::ResponseBody;
use crate::ui::traffic_log::TrafficLog;

use super::super::models::Workspace;
use super::environments::EnvironmentsTabs;
use super::menu_item::MenuItem;
use super::request_editor::RequestEditor;

#[derive(Debug, Clone)]
pub enum AppMsg {
    CreateRequest(Request),
    TogglingRequest(usize, bool),
    DeleteRequest(usize),
    RunHttpRequest,
}

pub struct App {
    workspace: Workspace,
    menu_items: FactoryVecDeque<MenuItem>,
    request_editor: Controller<RequestEditor>,
    environments: Controller<EnvironmentsTabs>,
}

pub struct Widgets {}

impl Component for App {
    type Init = Workspace;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = Widgets;
    type Root = gtk::ApplicationWindow;

    fn init_root() -> Self::Root {
        relm4::view! {
            window = gtk::ApplicationWindow {
                set_title: Some("Rustaman Vibration")
            }
        }
        window
    }

    fn init(
        workspace: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let menu_items =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output| match output {
                    MenuItemOutput::DeleteRequest(request_id) => AppMsg::DeleteRequest(request_id),
                    MenuItemOutput::TogglingRequest(request_id, active) => {
                        AppMsg::TogglingRequest(request_id, active)
                    }
                });

        for request in workspace.requests() {
            if request.active() {
                sender
                    .input_sender()
                    .send(AppMsg::CreateRequest(request.clone()))
                    .unwrap();
            }
        }

        let menu_items_container: &gtk::Box = menu_items.widget();
        menu_items_container.set_orientation(gtk::Orientation::Vertical);

        let left_menu = gtk::Box::new(gtk::Orientation::Vertical, 0);
        left_menu.set_hexpand(true);
        left_menu.set_vexpand(true);
        left_menu.append(menu_items_container);

        let request_editor =
            RequestEditor::builder()
                .launch(None)
                .forward(sender.input_sender(), |msg| match msg {
                    RequestOutput::RunHttpRequest => AppMsg::RunHttpRequest,
                });

        let environments = EnvironmentsTabs::builder()
            .launch(workspace.environments().to_vec())
            .forward(sender.input_sender(), |msg| match msg {
                EnvironmentsMsg::RunHttpRequest => AppMsg::RunHttpRequest,
            });

        let resp_body = ResponseBody::builder().launch(());
        let traffic_log = TrafficLog::builder().launch(());

        relm4::view! {
            request_box = gtk::Box {
                set_spacing: 20,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Paned::new(gtk::Orientation::Vertical) {
                    set_start_child: Some(request_editor.widget()),
                    set_end_child: Some(environments.widget()),
                }
            }
        }

        relm4::view! {
            response_box = gtk::Box {
                set_spacing: 20,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Paned::new(gtk::Orientation::Vertical) {
                    set_start_child: Some(resp_body.widget()),
                    set_end_child: Some(traffic_log.widget()),
                }
            }
        }

        relm4::view! {
            workspace_box = gtk::Box {
                set_spacing: 20,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Paned::new(gtk::Orientation::Horizontal) {
                    set_wide_handle: true,
                    set_position: 800,
                    set_start_child: Some(&request_box),
                    set_end_child: Some(&response_box),
                }
            }
        }

        relm4::view! {
            #[local_ref]
            root -> gtk::ApplicationWindow {
                set_hexpand: true,
                set_vexpand: true,
                gtk::Paned::new(gtk::Orientation::Horizontal) {
                    set_wide_handle: true,
                    set_position: 250,
                    set_start_child: Some(&left_menu),
                    set_end_child: Some(&workspace_box),
                }
            }
        }

        ComponentParts {
            model: App {
                workspace,
                menu_items,
                request_editor,
                environments,
            },
            widgets: Widgets {},
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let mut menu_items_guard = self.menu_items.guard();
        match message {
            AppMsg::CreateRequest(request) => {
                menu_items_guard.push_back(request);
            }
            AppMsg::TogglingRequest(request_id, active) => {
                info!("toggling request {:?}. active {}", request_id, active);
                if active {
                    if let Some(request) = self.workspace.request(request_id) {
                        self.request_editor
                            .emit(RequestMsg::RequestChanged(request.clone()));
                    }

                    for item in menu_items_guard.iter_mut() {
                        if item.id() == request_id {
                            item.set_selected(true);
                        }
                        if item.selected() && item.id() != request_id {
                            item.set_selected(false);
                        }
                    }
                }
            }
            AppMsg::RunHttpRequest => {
                let mut environ = String::new();
                let mut template = String::new();
                if let Some(env_id) = self.environments.widgets().environment_id() {
                    environ = self.environments.widgets().get_environment();
                    self.workspace.set_environ_payload(env_id, environ.as_str());
                }
                if let Some(request_id) = self.request_editor.model().request_id() {
                    let request_editor = self.request_editor.widgets();
                    template = request_editor.get_template();
                    self.workspace
                        .set_request_template(request_id, template.as_str());
                }

                let req_templates = http::parse_template(template.as_str());
                for req_template in req_templates.iter() {
                    let request = helpers::handlebars::render_template(
                        req_template.as_str(),
                        environ.as_str(),
                    )
                    .unwrap();
                    let httpreq = http::parse_request(request.as_str()).unwrap();

                    let mut default_port = 80;
                    let client = gio::SocketClient::new();
                    if httpreq.scheme == helpers::http::Scheme::HTTPS {
                        client.set_tls(true);
                        client.set_tls_validation_flags(httpreq.tls_flags);
                        default_port = 443;
                    }
                    let cancellable: Option<&gio::Cancellable> = None;

                    let host_and_port = httpreq.host_and_port();
                    debug!("Connecting to {:?}", host_and_port);
                    let socket_con = client
                        .connect_to_host(host_and_port.as_str(), default_port, cancellable)
                        .unwrap();

                    let stream: gio::IOStream = socket_con.upcast();
                    let writer = stream.output_stream();
                    let reader = stream.input_stream();

                    let http_frame = httpreq.http_frame().to_string();
                    debug!("Sending {:?}", http_frame);
                    writer
                        .write(http_frame.into_bytes().as_slice(), cancellable)
                        .unwrap();

                    let mut buf = vec![0; 1024];
                    let read_size = reader.read_all(buf.as_mut_slice(), cancellable).unwrap();
                    let resp =
                        String::from_utf8(buf.iter().take(read_size.0).copied().collect()).unwrap();

                    info!("Response: {}", resp)
                }
            }
            _ => (),
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
