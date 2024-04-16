pub fn init(filter_level: log::LevelFilter) {
    if let Some(level) = filter_level.to_level() {
        windebug_logger::init_with_level(level).ok();
    }
}