use leptos::prelude::*;
use opensauna_ui::app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
