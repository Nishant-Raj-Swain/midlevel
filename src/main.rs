mod browser;
mod start_page;
mod storage;
mod url_utils;

use browser::BrowserApp;
use gtk::prelude::*;

fn main() {
    let application = gtk::Application::builder()
        .application_id("com.midlevel.browser")
        .build();

    application.connect_activate(|app| {
        let _browser = BrowserApp::new(app);
    });

    application.run();
}