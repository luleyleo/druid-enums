use druid::{
    widget::{Button, Controller, Flex, Label, TextBox},
    AppLauncher, Data, Env, Event, EventCtx, Lens, PlatformError, Selector, Widget, WidgetExt,
    WindowDesc,
};
use druid_enums::Matcher;

const LOGIN: Selector<MainState> = Selector::new("druid-enums.basic.login");

#[derive(Clone, Data, Matcher)]
#[matcher(matcher_name = App)] // defaults to AppStateMatcher
enum AppState {
    Login(LoginState),
    Main(MainState),
}

#[derive(Clone, Data, Lens, Default)]
struct LoginState {
    user: String,
}

#[derive(Clone, Data, Lens)]
struct MainState {
    user: String,
    count: u32,
}

fn main() -> Result<(), PlatformError> {
    let window = WindowDesc::new(ui).title("Druid Enums");
    let state = AppState::Login(LoginState::default());
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(state)
}

fn ui() -> impl Widget<AppState> {
    // AppState::matcher() or
    App::new()
        .login(login_ui())
        .main(main_ui())
        .controller(LoginController)
}

fn login_ui() -> impl Widget<LoginState> {
    fn login(ctx: &mut EventCtx, state: &mut LoginState, _: &Env) {
        ctx.submit_command(LOGIN.with(MainState::from(state.clone())), None)
    }

    Flex::row()
        .with_child(TextBox::new().lens(LoginState::user))
        .with_spacer(5.0)
        .with_child(Button::new("Login").on_click(login))
        .center()
}

fn main_ui() -> impl Widget<MainState> {
    Flex::column()
        .with_child(Label::dynamic(MainState::welcome_label))
        .with_spacer(5.0)
        .with_child(
            Button::dynamic(MainState::count_label)
                .on_click(|_, state: &mut MainState, _| state.count += 1),
        )
        .center()
}

struct LoginController;
impl Controller<AppState, App> for LoginController {
    fn event(
        &mut self,
        child: &mut App,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(LOGIN) => {
                let main_state = cmd.get_unchecked(LOGIN).clone();
                *data = AppState::Main(main_state);
            }
            _ => {}
        }
        child.event(ctx, event, data, env)
    }
}

impl MainState {
    pub fn welcome_label(&self, _: &Env) -> String {
        format!("Welcome {}!", self.user)
    }

    pub fn count_label(&self, _: &Env) -> String {
        format!("clicked {} times", self.count)
    }
}

impl From<LoginState> for MainState {
    fn from(login: LoginState) -> Self {
        MainState {
            user: login.user,
            count: 0,
        }
    }
}
