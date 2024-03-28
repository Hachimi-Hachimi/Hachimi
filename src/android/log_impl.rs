use android_logger::FilterBuilder;

pub fn init(filter_level: log::LevelFilter) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(filter_level)
            .with_filter(
                FilterBuilder::new()
                    .filter_level(filter_level)
                    .filter_module("sqlparser", log::LevelFilter::Off) // annoying
                    .build()
            )
            .with_tag("Hachimi")
    );
}