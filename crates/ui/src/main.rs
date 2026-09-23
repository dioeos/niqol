slint::include_modules!();

#[allow(unused_variables)]
fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let weak = ui.as_weak();

    // ui.on_request_increase_value(move || {
    //     if let Some(ui) = weak.upgrade() {
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    ui.on_search_requested(move |string| {
        println!("search requested! {}", string);
    });

    ui.run()
}
