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
        .subscription(App::subscription)
        .window_size((900.0, 760.0))
        .run()
}
