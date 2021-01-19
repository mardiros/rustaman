use gdk;
use gdk::keys::constants;
use gtk::{
    self, prelude::*, ButtonsType, DialogFlags, GtkWindowExt, MessageDialog, MessageType,
    Orientation, WindowPosition, WindowType,
};

use relm::{connect, connect_stream, Component, ContainerWidget, Relm, Update, Widget};
use serde_yaml;

use super::super::helpers::http::{Http, HttpRequest, Msg as HttpMsg};
use super::super::models::{Environment, Request, Template, Workspace};
use super::environ_editor::{EnvironEditor, Msg as EnvironMsg};
use super::helpbox::{HelpBox, Msg as HelpBoxMsg};
use super::menu::{Menu, Msg as MenuMsg};
use super::request_editor::{Msg as EditorMsg, RequestEditor};
use super::request_logger::{Msg as RequestLoggerMsg, RequestLogger};
use super::response::{Msg as ResponseMsg, Response};
use super::response_status::{Msg as ResponseStatusMsg, ResponseStatus};

#[derive(Msg)]
pub enum Msg {
    CreatingRequest,
    TogglingRequest(usize, bool),
    Deleting(usize),
    Renaming(usize, String),
    RequestTemplateChanged(usize, Template),
    ExecutingCurrentRequestTemplate,
    TemplateFetched(Template),
    EnvironmentFetched(serde_yaml::Value),
    EnvironmentFetchedFailed(String),
    // The template has been rendered and will start the http request
    HttpRequestBeingExecuted(HttpRequest),
    // The request has been executed, we have a response
    HttpRequestExecuted(String),
    HttpCapturedRequestExecuted(String),
    DisplayError(String),

    SavingEnvironment(usize, String),
    DeletingEnvironment(usize),
    CreatingEnvironment(String),
    TogglingEnvironment(usize),
    Quitting,
    PressingKey(gdk::EventKey),
}

pub struct Model {
    workspace: Workspace,
    current_req: usize,
    current_env: usize,
    current_template: Template,
}

impl Model {
    pub fn name(&self) -> &str {
        self.workspace.name()
    }
    pub fn requests(&self) -> &[Request] {
        self.workspace.requests()
    }
    pub fn current_request(&self) -> Option<&Request> {
        self.workspace.request(self.current_req)
    }
    pub fn create_request(&mut self) -> &Request {
        self.workspace.create_request()
    }
    pub fn environments(&self) -> &[Environment] {
        self.workspace.environments()
    }
    pub fn create_environment(&mut self, name: &str) -> &Environment {
        self.workspace.create_environment(name)
    }
    pub fn current_environment(&self) -> Option<&Environment> {
        self.workspace.environment(self.current_env)
    }
}

pub struct Window {
    model: Model,
    menu: Component<Menu>,
    window: gtk::Window,
    relm: Relm<Window>,
    request_editor: Component<RequestEditor>,
    env_editor: Component<EnvironEditor>,
    help_box: Component<HelpBox>,
    response_status: Component<ResponseStatus>,
    response: Component<Response>,
    request_logger: Component<RequestLogger>,
}

impl Update for Window {
    type Model = Model;
    type ModelParam = Workspace;
    type Msg = Msg;

    fn model(_: &Relm<Self>, workspace: Workspace) -> Model {
        Model {
            workspace,
            current_req: 0,
            current_env: 0,
            current_template: "".to_owned(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::CreatingRequest => {
                let request = self.model.create_request();
                self.menu
                    .stream()
                    .emit(MenuMsg::CreatingRequest(request.clone()))
            }
            Msg::TogglingRequest(id, active) => {
                if self.model.current_req > 0 {
                    self.request_editor
                        .stream()
                        .emit(EditorMsg::RequestingSave(self.model.current_req));
                }
                if active {
                    self.model.current_req = id;
                    let req = self.model.current_request().unwrap(); // XXX
                    self.help_box.stream().emit(HelpBoxMsg::Hiding);
                    self.request_editor
                        .stream()
                        .emit(EditorMsg::TemplateChanged(req.template().to_owned()));
                } else if self.model.current_req == id {
                    self.request_editor.stream().emit(EditorMsg::Hiding);
                    self.help_box.stream().emit(HelpBoxMsg::Showing);
                    self.model.current_req = 0;
                }
            }
            Msg::Renaming(id, name) => {
                self.model.workspace.set_request_name(id, name.as_str());
            }
            Msg::Deleting(id) => {
                info!("Deleting template {}", id);
                self.model.workspace.delete_request(id);

                if self.model.current_req == id {
                    self.request_editor.stream().emit(EditorMsg::Hiding);
                    self.help_box.stream().emit(HelpBoxMsg::Showing);
                    self.model.current_req = 0;
                }

                self.menu.stream().emit(MenuMsg::Deleted(id));
            }
            Msg::RequestTemplateChanged(id, template) => {
                info!("Save Template Changes {} {}", id, template);
                self.model
                    .workspace
                    .set_request_template(id, template.as_str());
            }
            Msg::TemplateFetched(template) => {
                self.relm.stream().emit(Msg::RequestTemplateChanged(
                    self.model.current_req,
                    template.clone(),
                ));
                self.env_editor
                    .stream()
                    .emit(EnvironMsg::FetchingEnvironment);
                self.model.current_template = template;
            }
            Msg::ExecutingCurrentRequestTemplate => {
                self.request_editor
                    .stream()
                    .emit(EditorMsg::FetchingCurrentTemplate);
            }
            Msg::EnvironmentFetched(env) => {
                //let resp = self.runner.run_request(template.as_str());
                let params = (self.model.current_template.clone(), env);
                let http = relm::execute::<Http>(params);

                // Clean the status and response time
                connect_stream!(
                    http@HttpMsg::Writing(ref request), self.relm.stream(), Msg::HttpRequestBeingExecuted(request.clone()));

                connect_stream!(
                    http@HttpMsg::ReadDone(ref response), self.relm.stream(), Msg::HttpRequestExecuted(response.clone()));

                connect_stream!(
                    http@HttpMsg::ReadDone(ref response), self.relm.stream(), Msg::HttpCapturedRequestExecuted(response.clone()));

                connect_stream!(
                    http@HttpMsg::DisplayError(ref response), self.relm.stream(), Msg::DisplayError(response.to_string()));

                http.emit(HttpMsg::StartConsuming);
            }
            Msg::EnvironmentFetchedFailed(err) => {
                self.response.stream().emit(ResponseMsg::DisplayError(err));
            }
            Msg::HttpRequestBeingExecuted(request) => {
                self.response_status
                    .stream()
                    .emit(ResponseStatusMsg::ExecutingRequest(request.clone()));
                self.request_logger
                    .stream()
                    .emit(RequestLoggerMsg::ExecutingRequest(
                        request.obfuscate(self.model.current_environment().unwrap()),
                    ));
            }

            Msg::DisplayError(error) => {
                // Currently not called
                self.response
                    .stream()
                    .emit(ResponseMsg::DisplayError(error));
            }

            Msg::HttpCapturedRequestExecuted(response) => {
                self.response_status
                    .stream()
                    .emit(ResponseStatusMsg::RequestExecuted(response.clone()));
                self.request_logger
                    .stream()
                    .emit(RequestLoggerMsg::RequestExecuted(response));
            }

            Msg::HttpRequestExecuted(response) => {
                self.response_status
                    .stream()
                    .emit(ResponseStatusMsg::RequestExecuted(response.clone()));
                self.response
                    .stream()
                    .emit(ResponseMsg::RequestExecuted(response.clone()));
            }
            Msg::CreatingEnvironment(name) => {
                info!("Creating environment {}", name);
                let env = self.model.create_environment(name.as_str());
                self.env_editor
                    .stream()
                    .emit(EnvironMsg::EnvironmentCreated(env.clone()))
            }
            Msg::TogglingEnvironment(id) => {
                info!("Toggling environment {}", id);
                self.model.current_env = id;
            }
            Msg::SavingEnvironment(id, payload) => {
                info!("Save environment {} {:?}", id, payload);
                self.model
                    .workspace
                    .set_environ_payload(id, payload.as_str());
            }

            Msg::DeletingEnvironment(id) => {
                let dialog: MessageDialog = MessageDialog::new(
                    Some(&self.window),
                    DialogFlags::DESTROY_WITH_PARENT,
                    MessageType::Warning,
                    ButtonsType::OkCancel,
                    "Are you sure you want to delete?",
                );
                let result = dialog.run();

                if result == gtk::ResponseType::Ok {
                    info!("Deleting environment {}", id);
                    self.model.workspace.delete_environment(id);
                    self.env_editor
                        .stream()
                        .emit(EnvironMsg::EnvironmentDeleted(id))
                }
                dialog.close();
            }
            Msg::Quitting => gtk::main_quit(),
            Msg::PressingKey(key) => {
                let keyval = key.get_keyval();
                let keystate = key.get_state();

                if keystate.intersects(gdk::ModifierType::CONTROL_MASK) {
                    match keyval {
                        constants::w => self.relm.stream().emit(Msg::Quitting),
                        constants::n => self.relm.stream().emit(Msg::CreatingRequest),
                        constants::p => self.menu.stream().emit(MenuMsg::RequestingFilteringMenu),
                        constants::Return => self
                            .relm
                            .stream()
                            .emit(Msg::ExecutingCurrentRequestTemplate),
                        _ => {}
                    }
                } else if keyval == constants::F2 && self.model.current_req > 0 {
                    self.menu
                        .stream()
                        .emit(MenuMsg::RenamingRequest(self.model.current_req))
                }
            }
        }
    }
}

impl Widget for Window {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        let window = gtk::Window::new(WindowType::Toplevel);

        window.set_title(model.name());
        window.set_position(WindowPosition::Center);
        window.set_default_size(1280, 1024);

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quitting), Inhibit(false))
        );

        connect!(
            relm,
            window,
            connect_key_press_event(_, key),
            return (Msg::PressingKey(key.clone()), Inhibit(false))
        );

        let settings = gtk::Settings::get_default().unwrap();

        let use_dark = true;
        settings
            .set_property("gtk-application-prefer-dark-theme", &use_dark)
            .expect("Should switch to dark theme");

        let paned = gtk::Paned::new(Orientation::Horizontal);
        paned.set_wide_handle(true);
        paned.set_position(200);

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        let menubox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.set_hexpand(true);
        hbox.set_vexpand(true);
        let requests = model.requests().to_vec();
        let menu = menubox.add_widget::<Menu>(requests);
        window.set_hexpand(true);
        window.set_vexpand(true);
        hbox.pack_start(&menubox, true, true, 5);

        connect!(
            menu@MenuMsg::NewRequest,
            relm,
            Msg::CreatingRequest
        );

        connect!(
            menu@MenuMsg::TogglingRequest(id, active),
            relm,
            Msg::TogglingRequest(id, active)
        );

        connect!(
            menu@MenuMsg::Renaming(id, ref name),
            relm,
            Msg::Renaming(id, name.to_owned())
        );

        connect!(
            menu@MenuMsg::Deleting(id),
            relm,
            Msg::Deleting(id)
        );

        hbox.show_all();
        paned.pack1(&hbox, true, true);

        let main_box = gtk::Paned::new(Orientation::Horizontal);
        main_box.set_wide_handle(true);
        main_box.set_position(800);

        let editor_box = gtk::Paned::new(Orientation::Vertical);

        let req_editor_box = gtk::Box::new(Orientation::Vertical, 0);
        req_editor_box.set_hexpand(true);
        req_editor_box.set_vexpand(true);
        let request_editor = req_editor_box.add_widget::<RequestEditor>(());
        let help_box = req_editor_box.add_widget::<HelpBox>(());
        req_editor_box.show();
        connect!(
            request_editor@EditorMsg::Saving(id, ref template),
            relm,
            Msg::RequestTemplateChanged(id, template.to_owned())
        );
        connect!(
            request_editor@EditorMsg::NotifyingCurrentTemplate(ref template),
            relm,
            Msg::TemplateFetched(template.to_owned())
        );
        let envs = model.environments().to_vec();
        let env_editor_box = gtk::Box::new(Orientation::Vertical, 0);
        let env_editor = env_editor_box.add_widget::<EnvironEditor>(envs);

        connect!(
            env_editor@EnvironMsg::TogglingEnvironment(id),
            relm,
            Msg::TogglingEnvironment(id)
        );

        env_editor_box.show_all();

        editor_box.pack1(&req_editor_box, false, false);
        editor_box.pack2(&env_editor_box, false, false);
        editor_box.set_wide_handle(true);
        editor_box.set_position(550);
        editor_box.show();

        connect!(
            env_editor@EnvironMsg::FetchedEnvironment(ref result),
            relm,
            Msg::EnvironmentFetched(result.clone())
        );
        connect!(
            env_editor@EnvironMsg::FetchedEnvironmentFailed(ref err),
            relm,
            Msg::EnvironmentFetchedFailed(err.to_string())
        );
        connect!(
            env_editor@EnvironMsg::CreatingEnvironment(ref name),
            relm,
            Msg::CreatingEnvironment(name.to_owned())
        );

        connect!(
            env_editor@EnvironMsg::SavingEnvironment(id, ref env),
            relm,
            Msg::SavingEnvironment(id, env.clone())
        );

        connect!(
            env_editor@EnvironMsg::DeletingEnvironment(id),
            relm,
            Msg::DeletingEnvironment(id)
        );

        if model.environments().is_empty() {
            relm.stream()
                .emit(Msg::CreatingEnvironment("Dev".to_owned()));
        }

        main_box.pack1(&editor_box, false, false);

        let response_paned = gtk::Paned::new(Orientation::Vertical);
        let vbox = gtk::Box::new(Orientation::Vertical, 0);
        response_paned.set_hexpand(true);
        response_paned.set_vexpand(true);
        response_paned.set_position(800);
        response_paned.set_wide_handle(true);

        let response_status = vbox.add_widget::<ResponseStatus>(());
        let response = vbox.add_widget::<Response>(());

        response_paned.add(&vbox);
        let request_logger = response_paned.add_widget::<RequestLogger>(());

        response_paned.show_all();
        main_box.add2(&response_paned);

        editor_box.show();
        main_box.show();
        paned.pack2(&main_box, true, true);
        paned.show();
        window.add(&paned);

        window.show();
        Window {
            model,
            menu,
            window,
            request_editor,
            env_editor,
            help_box,
            response_status,
            response,
            request_logger,
            relm: relm.clone(),
        }
    }
}
