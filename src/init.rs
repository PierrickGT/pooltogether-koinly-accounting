use colored::Colorize;
use eyre::Result;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use indoc::indoc;

pub fn print_banner() {
    let banner = indoc! {
r#"

██████╗  ██████╗  ██████╗ ██╗  ████████╗ ██████╗  ██████╗ ███████╗████████╗██╗  ██╗███████╗██████╗
██╔══██╗██╔═══██╗██╔═══██╗██║  ╚══██╔══╝██╔═══██╗██╔════╝ ██╔════╝╚══██╔══╝██║  ██║██╔════╝██╔══██╗
██████╔╝██║   ██║██║   ██║██║     ██║   ██║   ██║██║  ███╗█████╗     ██║   ███████║█████╗  ██████╔╝
██╔═══╝ ██║   ██║██║   ██║██║     ██║   ██║   ██║██║   ██║██╔══╝     ██║   ██╔══██║██╔══╝  ██╔══██╗
██║     ╚██████╔╝╚██████╔╝███████╗██║   ╚██████╔╝╚██████╔╝███████╗   ██║   ██║  ██║███████╗██║  ██║
╚═╝      ╚═════╝  ╚═════╝ ╚══════╝╚═╝    ╚═════╝  ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝

██╗  ██╗ ██████╗ ██╗███╗   ██╗██╗  ██╗   ██╗     █████╗  ██████╗ ██████╗ ██████╗ ██╗   ██╗███╗   ██╗████████╗██╗███╗   ██╗ ██████╗
██║ ██╔╝██╔═══██╗██║████╗  ██║██║  ╚██╗ ██╔╝    ██╔══██╗██╔════╝██╔════╝██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝██║████╗  ██║██╔════╝
█████╔╝ ██║   ██║██║██╔██╗ ██║██║   ╚████╔╝     ███████║██║     ██║     ██║   ██║██║   ██║██╔██╗ ██║   ██║   ██║██╔██╗ ██║██║  ███╗
██╔═██╗ ██║   ██║██║██║╚██╗██║██║    ╚██╔╝      ██╔══██║██║     ██║     ██║   ██║██║   ██║██║╚██╗██║   ██║   ██║██║╚██╗██║██║   ██║
██║  ██╗╚██████╔╝██║██║ ╚████║███████╗██║       ██║  ██║╚██████╗╚██████╗╚██████╔╝╚██████╔╝██║ ╚████║   ██║   ██║██║ ╚████║╚██████╔╝
╚═╝  ╚═╝ ╚═════╝ ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝       ╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝
"#};

    log::info!("{}", format!("{}", banner.green().bold()));
}

pub fn setup_logger(multi: MultiProgress) -> Result<()> {
    let colors = ColoredLevelConfig {
        trace: Color::Cyan,
        debug: Color::Magenta,
        info: Color::Green,
        warn: Color::Red,
        error: Color::BrightRed,
    };

    let (level, logger) = Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .level(log::LevelFilter::Error)
        .level_for("pooltogether_koinly_accounting", log::LevelFilter::Info)
        .into_log();

    LogWrapper::new(multi, logger).try_init().unwrap();
    log::set_max_level(level);

    Ok(())
}
