use crate::start_page::get_start_page_html;
use crate::storage::StorageManager;
use crate::url_utils::build_target_url;

use gdk::keys::constants as keys;
use glib::Propagation;
use gtk::pango;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::WebView;
use webkit2gtk::WebViewExt;

pub struct BrowserState {
    pub storage: StorageManager,
    pub notebook: gtk::Notebook,
    pub address_entry: gtk::Entry,
    pub back_btn: gtk::Button,
    pub fwd_btn: gtk::Button,
    pub reload_btn: gtk::Button,
    pub bookmark_btn: gtk::Button,
    pub theme_btn: gtk::Button,
    pub window: gtk::ApplicationWindow,
}

pub struct BrowserApp {
    state: Rc<RefCell<BrowserState>>,
}

impl BrowserApp {
    pub fn new(app: &gtk::Application) -> Self {
        let storage = StorageManager::new();

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("MidLevel")
            .default_width(1100)
            .default_height(750)
            .build();

        let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        toolbar.set_margin_start(6);
        toolbar.set_margin_end(6);
        toolbar.set_margin_top(6);
        toolbar.set_margin_bottom(6);

        let back_btn = gtk::Button::with_label("◀");
        let fwd_btn = gtk::Button::with_label("▶");
        let reload_btn = gtk::Button::with_label("↻");
        let home_btn = gtk::Button::with_label("🏠");

        let address_entry = gtk::Entry::new();
        address_entry.set_placeholder_text(Some("Search Google or enter address"));

        let bookmark_btn = gtk::Button::with_label("☆");
        let history_btn = gtk::Button::with_label("📜");
        let initial_theme_icon = if storage.data.dark_mode { "☀️" } else { "🌙" };
        let theme_btn = gtk::Button::with_label(initial_theme_icon);
        let new_tab_btn = gtk::Button::with_label("+");

        toolbar.pack_start(&back_btn, false, false, 0);
        toolbar.pack_start(&fwd_btn, false, false, 0);
        toolbar.pack_start(&reload_btn, false, false, 0);
        toolbar.pack_start(&home_btn, false, false, 0);
        toolbar.pack_start(&address_entry, true, true, 0);
        toolbar.pack_start(&bookmark_btn, false, false, 0);
        toolbar.pack_start(&history_btn, false, false, 0);
        toolbar.pack_start(&theme_btn, false, false, 0);
        toolbar.pack_start(&new_tab_btn, false, false, 0);

        main_vbox.pack_start(&toolbar, false, false, 0);

        let notebook = gtk::Notebook::new();
        notebook.set_scrollable(true);

        main_vbox.pack_start(&notebook, true, true, 0);
        window.add(&main_vbox);

        let dark_mode = storage.data.dark_mode;
        let state = Rc::new(RefCell::new(BrowserState {
            storage,
            notebook,
            address_entry,
            back_btn,
            fwd_btn,
            reload_btn,
            bookmark_btn,
            theme_btn,
            window,
        }));

        let browser_app = Self { state };
        browser_app.apply_theme(dark_mode);
        browser_app.setup_signals(new_tab_btn, home_btn, history_btn);

        browser_app.add_new_tab(None);

        browser_app.state.borrow().window.show_all();
        browser_app
    }

    fn apply_theme(&self, dark_mode: bool) {
        if let Some(settings) = gtk::Settings::default() {
            settings.set_property("gtk-application-prefer-dark-theme", dark_mode);
        }
    }

    fn get_current_webview(&self) -> Option<WebView> {
        let state = self.state.borrow();
        let page_num = state.notebook.current_page()?;
        let child = state.notebook.nth_page(Some(page_num))?;
        child.downcast::<WebView>().ok()
    }

    pub fn add_new_tab(&self, initial_url: Option<&str>) {
        let webview = WebView::new();

        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let tab_title = gtk::Label::new(Some("New Tab"));
        tab_title.set_ellipsize(pango::EllipsizeMode::End);
        tab_title.set_max_width_chars(15);

        let close_btn = gtk::Button::with_label("✕");
        close_btn.set_relief(gtk::ReliefStyle::None);

        tab_box.pack_start(&tab_title, true, true, 0);
        tab_box.pack_start(&close_btn, false, false, 0);
        tab_box.show_all();

        let state = self.state.borrow();
        let page_index = state.notebook.append_page(&webview, Some(&tab_box));
        state.notebook.set_tab_reorderable(&webview, true);
        webview.show();

        let state_clone = self.state.clone();
        let webview_clone = webview.clone();
        close_btn.connect_clicked(move |_| {
            let state = state_clone.borrow();
            if let Some(page_num) = state.notebook.page_num(&webview_clone) {
                state.notebook.remove_page(Some(page_num));
                if state.notebook.n_pages() == 0 {
                    state.window.close();
                }
            }
        });

        let state_clone = self.state.clone();
        let tab_title_clone = tab_title.clone();
        webview.connect_title_notify(move |view| {
            let title = view.title().map(|s| s.to_string()).unwrap_or_else(|| "Untitled".into());
            tab_title_clone.set_text(&title);

            let mut state = state_clone.borrow_mut();
            if let Some(current) = state.notebook.nth_page(state.notebook.current_page()) {
                if current == *view {
                    if let Some(uri) = view.uri() {
                        state.storage.add_history(title, uri.to_string());
                    }
                }
            }
        });

        let state_clone = self.state.clone();
        webview.connect_uri_notify(move |view| {
            let state = state_clone.borrow();
            if let Some(current) = state.notebook.nth_page(state.notebook.current_page()) {
                if current == *view {
                    if let Some(uri) = view.uri() {
                        let uri_str = uri.to_string();
                        if !uri_str.starts_with("data:text/html") {
                            state.address_entry.set_text(&uri_str);
                            let is_bm = state.storage.is_bookmarked(&uri_str);
                            state.bookmark_btn.set_label(if is_bm { "★" } else { "☆" });
                        } else {
                            state.address_entry.set_text("about:midlevel");
                            state.bookmark_btn.set_label("☆");
                        }
                    }
                    state.back_btn.set_sensitive(view.can_go_back());
                    state.fwd_btn.set_sensitive(view.can_go_forward());
                }
            }
        });

        if let Some(url) = initial_url {
            let target = build_target_url(url);
            webview.load_uri(&target);
        } else {
            self.load_start_page(&webview);
        }

        state.notebook.set_current_page(Some(page_index));
    }

    fn load_start_page(&self, webview: &WebView) {
        let state = self.state.borrow();
        let html = get_start_page_html(state.storage.data.dark_mode, &state.storage.data.bookmarks);
        webview.load_html(&html, Some("about:midlevel"));
    }

    fn setup_signals(&self, new_tab_btn: gtk::Button, home_btn: gtk::Button, history_btn: gtk::Button) {
        let app_self = self.clone();
        new_tab_btn.connect_clicked(move |_| { app_self.add_new_tab(None); });

        let app_self = self.clone();
        home_btn.connect_clicked(move |_| {
            if let Some(wv) = app_self.get_current_webview() {
                app_self.load_start_page(&wv);
            }
        });

        let app_self = self.clone();
        self.state.borrow().address_entry.connect_activate(move |entry| {
            let text = entry.text();
            let target = build_target_url(text.as_str());
            if let Some(wv) = app_self.get_current_webview() {
                if target == "about:midlevel" {
                    app_self.load_start_page(&wv);
                } else {
                    wv.load_uri(&target);
                }
            }
        });

        let app_self = self.clone();
        self.state.borrow().back_btn.connect_clicked(move |_| {
            if let Some(wv) = app_self.get_current_webview() {
                if wv.can_go_back() { wv.go_back(); }
            }
        });

        let app_self = self.clone();
        self.state.borrow().fwd_btn.connect_clicked(move |_| {
            if let Some(wv) = app_self.get_current_webview() {
                if wv.can_go_forward() { wv.go_forward(); }
            }
        });

        let app_self = self.clone();
        self.state.borrow().reload_btn.connect_clicked(move |_| {
            if let Some(wv) = app_self.get_current_webview() { wv.reload(); }
        });

        let app_self = self.clone();
        self.state.borrow().bookmark_btn.connect_clicked(move |btn| {
            if let Some(wv) = app_self.get_current_webview() {
                if let Some(uri) = wv.uri() {
                    let title = wv.title().map(|s| s.to_string()).unwrap_or_else(|| "Untitled".into());
                    let mut state = app_self.state.borrow_mut();
                    let is_now_bookmarked = state.storage.toggle_bookmark(title, uri.to_string());
                    btn.set_label(if is_now_bookmarked { "★" } else { "☆" });
                }
            }
        });

        let app_self = self.clone();
        history_btn.connect_clicked(move |_| { app_self.show_history_dialog(); });

        let app_self = self.clone();
        self.state.borrow().theme_btn.connect_clicked(move |btn| {
            let dark_mode = {
                let mut state = app_self.state.borrow_mut();
                state.storage.toggle_dark_mode()
            };
            app_self.apply_theme(dark_mode);
            btn.set_label(if dark_mode { "☀️" } else { "🌙" });

            if let Some(wv) = app_self.get_current_webview() {
                if wv.uri().map_or(true, |u| u.starts_with("data:text/html") || u == "about:midlevel") {
                    app_self.load_start_page(&wv);
                }
            }
        });

        let state_clone = self.state.clone();
        self.state.borrow().notebook.connect_switch_page(move |_, page, _| {
            if let Ok(wv) = page.clone().downcast::<WebView>() {
                let state = state_clone.borrow();
                state.back_btn.set_sensitive(wv.can_go_back());
                state.fwd_btn.set_sensitive(wv.can_go_forward());
                if let Some(uri) = wv.uri() {
                    let uri_str = uri.to_string();
                    if !uri_str.starts_with("data:text/html") {
                        state.address_entry.set_text(&uri_str);
                        let is_bm = state.storage.is_bookmarked(&uri_str);
                        state.bookmark_btn.set_label(if is_bm { "★" } else { "☆" });
                    } else {
                        state.address_entry.set_text("about:midlevel");
                        state.bookmark_btn.set_label("☆");
                    }
                }
            }
        });

        let app_self = self.clone();
        self.state.borrow().window.connect_key_press_event(move |_, key_event| {
            let state = key_event.state();
            let keyval = key_event.keyval();

            if state.contains(gdk::ModifierType::CONTROL_MASK) {
                match keyval {
                    keys::t => { app_self.add_new_tab(None); Propagation::Stop }
                    keys::w => {
                        let state = app_self.state.borrow();
                        if let Some(page_num) = state.notebook.current_page() {
                            state.notebook.remove_page(Some(page_num));
                            if state.notebook.n_pages() == 0 { state.window.close(); }
                        }
                        Propagation::Stop
                    }
                    keys::l => { app_self.state.borrow().address_entry.grab_focus(); Propagation::Stop }
                    keys::r => { if let Some(wv) = app_self.get_current_webview() { wv.reload(); } Propagation::Stop }
                    keys::h => { app_self.show_history_dialog(); Propagation::Stop }
                    keys::b => {
                        if let Some(wv) = app_self.get_current_webview() {
                            if let Some(uri) = wv.uri() {
                                let title = wv.title().map(|s| s.to_string()).unwrap_or_else(|| "Untitled".into());
                                let mut state = app_self.state.borrow_mut();
                                let is_bm = state.storage.toggle_bookmark(title, uri.to_string());
                                state.bookmark_btn.set_label(if is_bm { "★" } else { "☆" });
                            }
                        }
                        Propagation::Stop
                    }
                    _ => Propagation::Proceed,
                }
            } else { Propagation::Proceed }
        });
    }

    fn show_history_dialog(&self) {
        let state = self.state.borrow();
        let dialog = gtk::Dialog::with_buttons(
            Some("Browsing History"),
            Some(&state.window),
            gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
            &[("Close", gtk::ResponseType::Close)],
        );
        dialog.set_default_size(500, 400);

        let content_area = dialog.content_area();
        let scroll = gtk::ScrolledWindow::new(Option::<&gtk::Adjustment>::None, Option::<&gtk::Adjustment>::None);
        let list_box = gtk::ListBox::new();

        for item in state.storage.data.history.iter().rev() {
            let row_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
            row_box.set_margin_start(10);
            row_box.set_margin_end(10);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);

            let title_lbl = gtk::Label::new(Some(&item.title));
            title_lbl.set_halign(gtk::Align::Start);

            let url_lbl = gtk::Label::new(Some(&item.url));
            url_lbl.set_halign(gtk::Align::Start);

            row_box.pack_start(&title_lbl, false, false, 0);
            row_box.pack_start(&url_lbl, false, false, 0);

            let list_row = gtk::ListBoxRow::new();
            list_row.add(&row_box);
            list_box.add(&list_row);
        }

        scroll.add(&list_box);
        content_area.pack_start(&scroll, true, true, 0);
        dialog.show_all();

        let app_self = self.clone();
        let dialog_clone = dialog.clone();
        let history_items = state.storage.data.history.clone();

        list_box.connect_row_activated(move |_, row| {
            let idx = history_items.len().saturating_sub(1 + row.index() as usize);
            if let Some(item) = history_items.get(idx) {
                if let Some(wv) = app_self.get_current_webview() {
                    wv.load_uri(&item.url);
                }
                dialog_clone.close();
            }
        });

        dialog.connect_response(|d, _| d.close());
    }
}

impl Clone for BrowserApp {
    fn clone(&self) -> Self {
        Self { state: self.state.clone() }
    }
}