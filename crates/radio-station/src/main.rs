#![warn(rust_2018_idioms)]

#[macro_use]
extern crate log;

mod radio_station;
mod input;

use std::str::FromStr;

use radio_station::RadioStation;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .try_init()
        .unwrap();

    let matches = clap::App::new("dcs-radio-station")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::with_name("frequency")
                .short("f")
                .long("freq")
                .default_value("251000000")
                .help("Sets the SRS frequency (in Hz, e.g. 255000000 for 255MHz)")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("server")
                .short("s")
                .long("server")
                .help("Server and Port of SRS Server, for example: srs.taw.rocks:5002")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required
    let server = matches.value_of("server").unwrap();
    let freq = matches.value_of("frequency").unwrap();
    let freq = if let Ok(n) = u64::from_str(freq) {
        n
    } else {
        error!("The provided frequency is not a valid number");
        return Ok(());
    };

    let mut station = RadioStation::new("DCS Radio Station");
    station.set_frequency(freq);
    station.set_position(0.0, 0.0, 8000.);
    station.set_port(5002);

    info!("Start playing ...");
    station.play(server).await?;

    Ok(())
}
