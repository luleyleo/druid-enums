use match_derive::Matcher;

#[derive(Matcher)]
#[matcher(matcher_name = App)]
enum AppState {
    #[matcher(builder_name = login_different)]
    Login(LoginState),
    Main(MainState),
}

struct LoginState;
struct MainState;

fn main() {}
