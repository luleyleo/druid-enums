# Druid Enums

This allows deriving a `Matcher` for any Rust enum.
The `Matcher` allows matching a `druid::Widget` to each variant of the enum.

## Usage

It's not (yet) published on `crates.io`, thus it should be used through Git:
```toml
[dependencies]
druid-enums = { git = "https://github.com/finnerale/druid-enums" }
```

## Example

Just a sketch, but you can find the fully working example [here](./examples/login.rs).

```rust
#[derive(Clone, Data, Matcher)]
#[matcher(matcher_name = App)] // defaults to AppStateMatcher
enum AppState {
    Login(LoginState),
    #[matcher(builder_name = my_main)]
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
        .my_main(main_ui())
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
                .on_click(|_, state, _| state.count += 1),
        )
        .center()
}
```

