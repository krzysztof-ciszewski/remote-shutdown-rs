use std::process::Command;
use std::time::Duration;
use std::fs;

use rocket::{build, get, launch, routes, Build, Rocket};
use slint::{Timer, TimerMode};

const DEFAULT_DELAY: u32 = 60;
const SECRET_FILE: &str = "secret";
const APP_ID: &str = "remote_shutdown";

#[get("/<secret>/shutdown?<delay>")]
async fn shutdown(secret: String, delay: Option<u32>) {
    if get_secret() != secret {
        return;
    }

    render_window(delay.unwrap_or(DEFAULT_DELAY));
}

fn render_window(delay: u32) {
    let win = MainWindow::new().unwrap();
    let win_weak = win.as_weak();
    win.global::<Global>().on_timer_ended(move || {
        win_weak.upgrade().unwrap().hide().unwrap();
        shutdown_system();
    });
    win.set_time_left((delay * 1000).into());
    let win_weak = win.as_weak();
    win.on_button_pressed(move || {
        let win = win_weak.upgrade().unwrap();
        win.hide().unwrap();
    });

    let timer = Timer::default();
    {
        let main_window_weak = win.as_weak();
        timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let main_window = main_window_weak.unwrap();
            main_window.invoke_tick(1000);
        });
    }

    win.run().unwrap();
}

slint::slint! {
    import { AboutSlint, Button, VerticalBox } from "std-widgets.slint";
    export global Global  {
        callback timer_ended();
    }
    export component MainWindow inherits Window {
        in-out property <duration> time-left: 60s;
        callback button_pressed <=> button.clicked;
        callback tick(duration);
        tick(passed-time) => {
            root.time-left += (-passed-time);
            root.time-left = max(root.time-left, 0);
            if (root.time-left == 0s) {
                Global.timer_ended();
            }
        }

        VerticalBox {
            alignment: start;
            Text {
                text: "Shutdown in " + (root.time-left / 1s) + "s";
                font-size: 24px;
                horizontal-alignment: center;
            }
            HorizontalLayout { alignment: center; button := Button { text: "Abort"; } }
        }
    }
}

fn get_secret() -> String {
    let dirs = xdg::BaseDirectories::with_prefix(APP_ID).unwrap();
    let mut path = dirs.find_config_file(SECRET_FILE);
    if path.is_none() {
        path = Some(
            dirs.place_config_file(SECRET_FILE)
                .expect("Cannot create config dir"),
        );
        fs::write(path.clone().unwrap(), "secret").unwrap();
    }
    fs::read_to_string(path.unwrap()).unwrap()
}

fn shutdown_system() {
    Command::new("shutdown")
        .args(["-h", "now"])
        .output()
        .expect("Failed to execute shutdown");
}

#[launch]
async fn rocket() -> Rocket<Build> {
    build().mount("/", routes![shutdown])
}
