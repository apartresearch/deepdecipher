//! Sets up logging.

use env_logger::{fmt::Color, Logger, Target};
use log::{Level, LevelFilter, Log};
use multi_log::MultiLogger;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::cli;

fn log_file_path<P>(dir: P, index: u32) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut result = PathBuf::new();
    let dir = dir.as_ref();
    result.push(dir.join(format!("{index}.log")));
    result
}

fn ensure_log_dir<P>(dir: P)
where
    P: AsRef<Path>,
{
    fs::create_dir_all(log_file_path(dir, 0).as_path().parent().unwrap())
        .expect("Could not create log dir.");
}

fn log_index<P>(dir: P) -> u32
where
    P: AsRef<Path>,
{
    (0..)
        .find(|&log_index| {
            let file_path = log_file_path(dir.as_ref(), log_index);
            match file_path.as_path().metadata() {
                Ok(_) => false,
                Err(error) => match error.kind() {
                    std::io::ErrorKind::NotFound => true,
                    _ => panic!("Could not create log file. Error: {:?}", error),
                },
            }
        })
        .unwrap()
}

fn create_write_logger<P>(dir: P) -> Box<Logger>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();

    ensure_log_dir(dir);

    let log_file_path = log_file_path(dir, log_index(dir));

    let log_file_target =
        Box::new(fs::File::create(log_file_path).expect("Could not create log file"));

    let write_logger = env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] {} {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                buf.timestamp_millis(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Debug)
        .target(Target::Pipe(log_file_target))
        .build();

    Box::new(write_logger)
}

fn create_stdout_logger() -> Box<Logger> {
    let stdout_logger = env_logger::builder()
        .format(|buf, record| {
            let mut style = buf.style();
            match record.level() {
                Level::Error => style.set_color(Color::Red),
                Level::Warn => style.set_color(Color::Yellow),
                Level::Info => style.set_color(Color::Green),
                Level::Debug => style.set_color(Color::Blue),
                Level::Trace => style.set_color(Color::Cyan),
            };
            style.set_bold(true);
            writeln!(buf, "[{}] {}", style.value(record.level()), record.args())
        })
        .filter_level(LevelFilter::Info)
        .target(Target::Stdout)
        .build();

    Box::new(stdout_logger)
}

pub fn log_init(log_path: Option<impl AsRef<Path>>) {
    // Log to stdout.
    let mut loggers: Vec<Box<dyn Log + 'static>> = vec![create_stdout_logger()];

    // Log to file.
    if let Some(log_path) = log_path {
        loggers.push(create_write_logger(log_path));
    }

    // Initialize multiple loggers.
    MultiLogger::init(loggers, Level::Trace).expect("Could not initialize logger(s).");

    // Log panics.
    log_panics::init();
}

pub fn log_init_config(config: &cli::ServerConfig) {
    log_init(config.log_path());
}
