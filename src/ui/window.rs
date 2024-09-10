// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]
use std::time::SystemTime;

use relm4::component::Connector;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender};

use reqwest;

use crate::helpers::httpparser;
use crate::ui::environments::{EnvironmentsMsg, EnvironmentsOutput};
use crate::ui::request_editor::{RequestMsg, RequestOutput};
use crate::ui::response_body::{ResponseBody, ResponseBodyMsg};
use crate::ui::sidebar::SideBarOutput;
use crate::ui::traffic_log::{TrafficLog, TrafficLogMsg};

use super::super::models::{Environment, Workspace, USER_AGENT};
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
    SaveHttpRequest(usize, String),
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

impl App {
    fn refresh_environment(&mut self) -> Environment {
        let mut environ = Environment::default();
        if let Some(env_id) = self.environments.model().environment_id() {
            let environ_string = self.environments.model().get_environment();
            self.workspace
                .set_environ_payload(env_id, environ_string.as_str());
            environ = self.workspace.environment(env_id).unwrap().clone();
        }
        environ
    }
    fn refresh_request(&mut self) -> Vec<String> {
        let mut template = String::new();
        if let Some(request_id) = self.request_editor.model().request_id() {
            let request_editor = self.request_editor.widgets();
            template = request_editor.get_template();
            self.workspace
                .set_request_template(request_id, template.as_str());
        }
        httpparser::split_template(template.as_str())
    }
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
                    RequestOutput::SaveHttpRequest(id, template) => {
                        AppMsg::SaveHttpRequest(id, template)
                    }
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
            AppMsg::SaveHttpRequest(id, template) => {
                self.workspace.set_request_template(id, template.as_str());
                self.workspace.safe_sync();
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
                let environ = self.refresh_environment();
                let req_templates = self.refresh_request();
                for req_template in req_templates.iter() {
                    debug!("Processing {:?}", req_template);
                    let request_parsed = httpparser::load_template(req_template.as_str(), &environ);
                    if let Err(rustaman_err) = request_parsed {
                        let error = format!("{:?}", rustaman_err);
                        self.response_body
                            .emit(ResponseBodyMsg::ReceivingError(error));
                        return;
                    }

                    let httpreq = request_parsed.unwrap();
                    let mut cbuilder =
                        reqwest::blocking::ClientBuilder::new().user_agent(USER_AGENT);
                    if !httpreq.verify_cert() {
                        cbuilder = cbuilder.danger_accept_invalid_certs(true);
                    }
                    let cli = cbuilder.build().unwrap();
                    let mut req = cli.request(httpreq.method(), httpreq.url());
                    for (key, val) in httpreq.headers() {
                        req = req.header(key, val);
                    }
                    if let Some(body) = httpreq.body() {
                        req = req.body(body.to_string());
                    }

                    let time = SystemTime::now();

                    let obfuscated_frame = httpreq.obfuscate(&environ).http_frame().to_string();
                    self.traffic_log
                        .emit(TrafficLogMsg::SendingHttpRequest(obfuscated_frame));

                    self.traffic_log
                        .emit(TrafficLogMsg::RequestSent(httpreq.http_frame().len()));

                    let req_response = req.send();
                    if let Err(err) = req_response {
                        self.response_body
                            .emit(ResponseBodyMsg::ReceivingError(err.to_string()));
                        self.traffic_log
                            .emit(TrafficLogMsg::ReceivingError(err.to_string()));
                        return;
                    }
                    let response = req_response.unwrap();
                    let mut resp = String::new();
                    // response.read_to_string(&mut resp).unwrap();
                    let version = format!("{:?}", response.version());
                    resp.push_str(version.as_str());
                    resp.push(' ');

                    resp.push_str(response.status().as_str());
                    resp.push(' ');
                    resp.push_str(response.status().canonical_reason().unwrap_or(""));
                    resp.push_str("\r\n");
                    for (key, hval) in response.headers() {
                        resp.push_str(key.to_string().as_str());
                        resp.push_str(": ");
                        resp.push_str(&String::from_utf8_lossy(hval.as_bytes()));
                        resp.push_str("\r\n");
                    }
                    resp.push_str("\r\n");
                    resp.push_str(&response.text().unwrap());

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
