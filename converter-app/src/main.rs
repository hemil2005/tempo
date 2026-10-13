mod app;
mod job;
mod settings;
mod worker;

#[cfg(test)]
mod logic_tests;

use app::App;

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size((960.0, 780.0))
        .run()
}
