// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]
use std::time::SystemTime;

use relm4::component::Connector;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, gtk::gio, ComponentParts, ComponentSender};

use crate::helpers::{self, http};
use crate::ui::environments::{EnvironmentsMsg, EnvironmentsOutput};
use crate::ui::request_editor::{RequestMsg, RequestOutput};
use crate::ui::response_body::{ResponseBody, ResponseBodyMsg};
use crate::ui::sidebar::SideBarOutput;
use crate::ui::traffic_log::{TrafficLog, TrafficLogMsg};

use super::super::models::{Environment, Workspace};
use super::environments::EnvironmentsTabs;
use super::request_editor::RequestEditor;
use super::sidebar::{SideBar, SideBarMsg};
use super::status_line::{StatusLine, StatusLineMsg};

#[derive(Debug, Clone)]
pub enum AppMsg {
    TogglingRequest(usize),
    DeleteRequest(usize),
    ToggleOff,
    RunHttpRequest,
    RenameRequest(usize, String),
    NewRequest,
    SearchingRequest,
    CreateEnvironment(String),
    RenameEnvironment(usize, String),
    DeleteEnvironment(usize),
}

pub struct App {
    workspace: Workspace,
    sidebar: Controller<SideBar>,
    request_editor: Controller<RequestEditor>,
    environments: Controller<EnvironmentsTabs>,
    response_body: Connector<ResponseBody>,
    traffic_log: Connector<TrafficLog>,
    status_line: Connector<StatusLine>,
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
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let sidebar = SideBar::builder()
            .launch(workspace.requests().into())
            .forward(sender.input_sender(), |msg| match msg {
                SideBarOutput::NewRequest => AppMsg::NewRequest,
                SideBarOutput::DeleteRequest(request_id) => AppMsg::DeleteRequest(request_id),
                SideBarOutput::TogglingRequest(request_id) => AppMsg::TogglingRequest(request_id),
                SideBarOutput::RenameRequest(request_id, name) => {
                    AppMsg::RenameRequest(request_id, name)
                }
                SideBarOutput::ToggleOff => AppMsg::ToggleOff,
            });

        let request_editor =
            RequestEditor::builder()
                .launch(None)
                .forward(sender.input_sender(), |msg| match msg {
                    RequestOutput::RunHttpRequest => AppMsg::RunHttpRequest,
                });

        let environments = EnvironmentsTabs::builder()
            .launch(workspace.environments().to_vec())
            .forward(sender.input_sender(), |msg| match msg {
                EnvironmentsOutput::RunHttpRequest => AppMsg::RunHttpRequest,
                EnvironmentsOutput::CreateEnvironment(name) => AppMsg::CreateEnvironment(name),
                EnvironmentsOutput::RenameEnvironment(env_id, name) => {
                    AppMsg::RenameEnvironment(env_id, name)
                }
                EnvironmentsOutput::DeleteEnvironment(env_id) => AppMsg::DeleteEnvironment(env_id),
            });

        let response_body = ResponseBody::builder().launch(());
        let traffic_log = TrafficLog::builder().launch(());
        let status_line = StatusLine::builder().launch(());
        let status_line_widget = status_line.widget();
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
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,
                set_hexpand: true,
                set_vexpand: true,
                #[local_ref]
                status_line_widget -> gtk::Box {
                    set_hexpand: true,
                    set_vexpand: false,
                },
                gtk::Paned::new(gtk::Orientation::Vertical) {
                    set_start_child: Some(response_body.widget()),
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

        let root_sender = sender.input_sender().clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_evt, key, _code, mask| {
            if mask != gtk::gdk::ModifierType::CONTROL_MASK {
                return false.into();
            }
            match key {
                gtk::gdk::Key::n => {
                    root_sender.emit(AppMsg::NewRequest);
                    true.into()
                }
                gtk::gdk::Key::p | gtk::gdk::Key::k => {
                    root_sender.emit(AppMsg::SearchingRequest);
                    true.into()
                }
                _ => false.into(),
            }
        });
        root.add_controller(controller);

        relm4::view! {
            #[local_ref]
            root -> gtk::ApplicationWindow {
                set_hexpand: true,
                set_vexpand: true,
                gtk::Paned::new(gtk::Orientation::Horizontal) {
                    set_wide_handle: true,
                    set_position: 250,
                    set_start_child: Some(sidebar.widget()),
                    set_end_child: Some(&workspace_box),
                }
            }
        }

        ComponentParts {
            model: App {
                workspace,
                sidebar,
                request_editor,
                environments,
                traffic_log,
                status_line,
                response_body,
            },
            widgets: Widgets {},
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AppMsg::NewRequest => {
                debug!("Creating new request");
                let request = self.workspace.create_request();
                self.sidebar
                    .emit(SideBarMsg::CreateRequest(request.clone()));
            }
            AppMsg::TogglingRequest(request_id) => {
                info!("toggling request {:?}", request_id);
                if let Some(request) = self.workspace.request(request_id) {
                    self.request_editor
                        .emit(RequestMsg::RequestChanged(request.clone()));
                }
            }
            AppMsg::ToggleOff => {
                self.request_editor.emit(RequestMsg::ToggleOff);
            }
            AppMsg::RenameRequest(request_id, name) => {
                self.workspace.set_request_name(request_id, name.as_str());
                self.workspace.safe_sync();
                self.sidebar
                    .emit(SideBarMsg::RequestRenamed(request_id, name))
            }
            AppMsg::DeleteRequest(request_id) => {
                self.workspace.delete_request(request_id);
                self.workspace.safe_sync();
                self.sidebar.emit(SideBarMsg::RequestDeleted(request_id))
            }
            AppMsg::SearchingRequest => self.sidebar.emit(SideBarMsg::SearchingRequest),
            AppMsg::CreateEnvironment(name) => {
                let env = self.workspace.create_environment(name.as_str());
                self.environments
                    .emit(EnvironmentsMsg::EnvironmentCreated(env.clone()));
                self.workspace.safe_sync();
            }
            AppMsg::RenameEnvironment(environment_id, name) => {
                self.workspace
                    .set_environment_name(environment_id, name.as_str());
                self.environments
                    .emit(EnvironmentsMsg::EnvironmentRenamed(environment_id, name));
            }
            AppMsg::DeleteEnvironment(environment_id) => {
                self.workspace.delete_environment(environment_id);
                self.environments
                    .emit(EnvironmentsMsg::EnvironmentDeleted(environment_id));
            }
            AppMsg::RunHttpRequest => {
                let mut environ = Environment::default();
                let mut template = String::new();
                if let Some(env_id) = self.environments.model().environment_id() {
                    let environ_string = self.environments.model().get_environment();
                    self.workspace
                        .set_environ_payload(env_id, environ_string.as_str());
                    environ = self.workspace.environment(env_id).unwrap().clone();
                }
                if let Some(request_id) = self.request_editor.model().request_id() {
                    let request_editor = self.request_editor.widgets();
                    template = request_editor.get_template();
                    self.workspace
                        .set_request_template(request_id, template.as_str());
                }

                let req_templates = http::split_template(template.as_str());
                for req_template in req_templates.iter() {
                    debug!("Processing {:?}", req_template);
                    let request_parsed = http::load_template(req_template.as_str(), &environ);
                    if let Err(rustaman_err) = request_parsed {
                        let error = format!("{:?}", rustaman_err);
                        self.response_body
                            .emit(ResponseBodyMsg::ReceivingError(error));
                        return;
                    }

                    let httpreq = request_parsed.unwrap();

                    let mut default_port = 80;
                    let client = gio::SocketClient::new();
                    if httpreq.scheme == helpers::http::Scheme::HTTPS {
                        client.set_tls(true);
                        client.set_tls_validation_flags(httpreq.tls_flags);
                        default_port = 443;
                    }
                    let cancellable: Option<&gio::Cancellable> = None;

                    let time = SystemTime::now();
                    let host_and_port = httpreq.host_and_port();
                    debug!("Connecting to {:?}", host_and_port);
                    self.traffic_log
                        .emit(TrafficLogMsg::Connecting(host_and_port.clone()));

                    let socket_con_result =
                        client.connect_to_host(host_and_port.as_str(), default_port, cancellable);

                    if let Err(gsocket_err) = socket_con_result {
                        let error = format!("Connection failed: {:?}", gsocket_err);
                        self.response_body
                            .emit(ResponseBodyMsg::ReceivingError(error));
                        return;
                    }

                    let socket_con = socket_con_result.unwrap();
                    let stream: gio::IOStream = socket_con.upcast();
                    let writer = stream.output_stream();
                    let reader = stream.input_stream();

                    let http_frame = httpreq.http_frame().to_string();
                    debug!("Sending {:?}", http_frame);

                    let obfuscated_frame = httpreq.obfuscate(&environ).http_frame().to_string();
                    self.traffic_log
                        .emit(TrafficLogMsg::SendingHttpRequest(obfuscated_frame));

                    let written = writer.write(http_frame.into_bytes().as_slice(), cancellable);
                    match written {
                        Ok(len) => {
                            self.traffic_log.emit(TrafficLogMsg::RequestSent(len));
                        }
                        Err(err) => {
                            self.response_body
                                .emit(ResponseBodyMsg::ReceivingError(format!("{:?}", err)));
                        }
                    }

                    let mut response: Vec<u8> = Vec::new();
                    let mut buf = vec![0; 1024];
                    loop {
                        let read_size = reader.read_all(buf.as_mut_slice(), cancellable).unwrap();
                        if read_size.0 == 0 {
                            debug!("no more bytes");
                            break;
                        }
                        if let Some(err) = read_size.1 {
                            let error = format!("Socket Reading Error: {:?}", err);
                            self.response_body
                                .emit(ResponseBodyMsg::ReceivingError(error));
                            return;
                        }
                        debug!("{} bytes received", read_size.0);
                        response.extend_from_slice(&buf[0..read_size.0]);
                    }
                    let resp = String::from_utf8(response).unwrap();

                    let duration = time.elapsed().unwrap(); // SystemTimeError!
                    debug!("Response: {}", resp);
                    self.status_line
                        .emit(StatusLineMsg::ReceivingHttpResponse(resp.clone(), duration));
                    self.response_body
                        .emit(ResponseBodyMsg::ReceivingHttpResponse(resp.clone()));
                    self.traffic_log
                        .emit(TrafficLogMsg::ReceivingHttpResponse(resp));
                    debug!("Done with the request");
                }
                debug!("Done with all the requests")
            }
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}
