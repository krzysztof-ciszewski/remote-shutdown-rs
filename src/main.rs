use std::{fs, thread};
use std::process::Command;
use std::time::Duration;

use gtk::{Application, ApplicationWindow, Button, gio, glib, Label, Orientation};
use gtk::glib::clone;
use gtk::prelude::*;
use rocket::{build, Build, get, launch, Rocket, routes};

const DEFAULT_DELAY: u8 = 60;
const SECRET_FILE: &str = "secret";
const APP_ID: &str = "remote_shutdown";

#[get("/<secret>/shutdown?<delay>")]
async fn shutdown(secret: String, delay: Option<u8>) {
    if get_secret() != secret {
        return;
    }

    let app = Application::builder().application_id(APP_ID).build();
    let delay = delay.unwrap_or(DEFAULT_DELAY);

    app.connect_activate(move |application: &Application| build_popup(application, delay));
    app.run();
}

fn get_secret() -> String {
    let dirs = xdg::BaseDirectories::with_prefix(APP_ID).unwrap();
    let mut path = dirs.find_config_file(SECRET_FILE);
    if path.is_none() {
        path = Some(dirs.place_config_file(SECRET_FILE).expect("Cannot create config dir"));
        fs::write(path.clone().unwrap(), "secret").unwrap();
    }
    fs::read_to_string(path.unwrap()).unwrap()
}

fn build_popup(app: &Application, delay: u8) {
    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let text = Label::builder()
        .label(format!("Shutdown in {}s", delay))
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let button = Button::builder()
        .label("Abort")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let (abort_sender, abort_receiver) = async_channel::bounded(1);
    button.connect_clicked(move |_| {
        abort_sender.send_blocking(true).expect("channel is not open");
    });

    gtk_box.append(&text);
    gtk_box.append(&button);

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_ID)
        .child(&gtk_box)
        .build();

    let (update_label_sender, update_label_receiver) = async_channel::bounded(1);
    let (close_window_sender, close_window_receiver) = async_channel::bounded(1);

    window.connect_show(move |_| {
        let update_label_sender = update_label_sender.clone();
        let abort_receiver = abort_receiver.clone();
        let close_window_sender = close_window_sender.clone();
        gio::spawn_blocking(move || {
            update_label_sender
                .send_blocking(delay)
                .expect("The channel needs to be open.");
            let mut secs: u8 = delay;
            let mut miliseconds: u16 = 0;
            let mut shutdown = true;
            while secs > 0 {
                let abort = abort_receiver.try_recv();
                if abort.is_ok() {
                    shutdown = false;
                    break;
                }

                thread::sleep(Duration::from_millis(100));
                miliseconds += 100;

                if miliseconds == 1000 {
                    miliseconds = 0;
                    secs -= 1;
                    update_label_sender
                        .send_blocking(secs)
                        .expect("The channel needs to be open.");
                }

            }
            if shutdown {
                shutdown_system();
            }
            close_window_sender.send_blocking(true).expect("channel needs to be open");
        });
    });

    glib::spawn_future_local(clone!(@weak window => async move {
        while let Ok(close_win) = close_window_receiver.recv().await {
            if close_win {
                window.close();
            }
        }
    }));

    glib::spawn_future_local(clone!(@weak text => async move {
        while let Ok(secs) = update_label_receiver.recv().await {
            text.set_label(format!("Shutdown in {}s", secs).as_str());
        }
    }));

    window.present();
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
