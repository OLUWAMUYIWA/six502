use cursive::views::TextView;

fn main() {
    let mut siv = cursive::default();
    siv.add_global_callback('q', |c| c.quit());
    siv.add_layer(TextView::new("Welcome to 6502. press <q> to exit"));
    siv.run();
}
