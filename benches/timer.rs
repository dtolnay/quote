use std::io::Write;
use std::time::Instant;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const ITERATIONS: u32 = 1000;

pub fn time<T>(name: &'static str, function: impl Fn() -> T) {
    let begin = Instant::now();
    for _ in 0..ITERATIONS {
        _ = function();
    }
    let micros = (begin.elapsed() / ITERATIONS).as_micros();
    let mode = ["release", "debug"][cfg!(debug_assertions) as usize];
    let mut writer = StandardStream::stderr(ColorChoice::Auto);
    _ = writer.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
    _ = writeln!(&mut writer, "{} in {} mode: {} micros", name, mode, micros);
}
