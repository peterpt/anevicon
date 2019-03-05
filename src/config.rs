/* anevicon: The most powerful UDP-based load generator, written in Rust.
 * Copyright (C) 2019  Temirkhan Myrzamadi <gymmasssorla@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * For more information see <https://github.com/Gymmasssorla/anevicon>.
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;
use std::num::{NonZeroUsize, ParseIntError};
use std::path::PathBuf;
use std::time::Duration;

use humantime::parse_duration;
use structopt::clap::ArgGroup;
use structopt::StructOpt;

#[derive(Debug, Clone, Eq, PartialEq, StructOpt)]
#[structopt(
    author = "Temirkhan Myrzamadi <gymmasssorla@gmail.com>",
    about = "An UDP-based server stress-testing tool, written in Rust.",
    after_help = "For more information see <https://github.com/Gymmasssorla/anevicon>.",
    set_term_width = 80
)]
pub struct ArgsConfig {
    /// A receiver of generated traffic, specified as an IP-address and a
    /// port number, separated by a colon.
    #[structopt(
        short = "r",
        long = "receiver",
        takes_value = true,
        value_name = "SOCKET-ADDRESS",
        required = true
    )]
    pub receiver: SocketAddr,

    /// A sender of generated traffic, specified as an IP-address and a
    /// port number, separated by a colon.
    #[structopt(
        short = "s",
        long = "sender",
        takes_value = true,
        value_name = "SOCKET-ADDRESS",
        default_value = "0.0.0.0:0"
    )]
    pub sender: SocketAddr,

    /// A waiting time span before a test execution used to prevent a
    /// launch of an erroneous (unwanted) test.
    #[structopt(
        short = "w",
        long = "wait",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "5secs",
        parse(try_from_str = "parse_duration")
    )]
    pub wait: Duration,

    /// A periodicity of sending packets. By default, all packets will be
    /// sent momentarily (without any periodicity).
    #[structopt(
        long = "send-periodicity",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "0secs",
        parse(try_from_str = "parse_duration")
    )]
    pub send_periodicity: Duration,

    /// A count of packets per displaying test summaries. It is highly
    /// recommended to not set a too small value (say, 6).
    #[structopt(
        long = "display-periodicity",
        takes_value = true,
        value_name = "PACKETS",
        default_value = "300",
        parse(try_from_str = "parse_non_zero_usize")
    )]
    pub display_periodicity: NonZeroUsize,

    /// A timeout of sending every single packet. If a timeout is reached,
    /// an error will be printed.
    #[structopt(
        long = "send-timeout",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "10secs",
        parse(try_from_str = "parse_duration")
    )]
    pub send_timeout: Duration,

    /// A name of a future test. This option lets produce the program
    /// beautiful output and doesn't make any sense on test performing.
    #[structopt(
        short = "n",
        long = "test-name",
        takes_value = true,
        value_name = "STRING",
        default_value = "unnamed"
    )]
    pub test_name: String,

    #[structopt(flatten)]
    pub logging_config: LoggingConfig,

    #[structopt(flatten)]
    pub stop_conditions_config: StopConditionsConfig,

    #[structopt(flatten)]
    pub packet_config: PacketConfig,
}

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
pub struct LoggingConfig {
    /// A file for redirecting all user messages (notifications, warnings,
    /// and errors) without debugging information.
    #[structopt(
        short = "o",
        long = "output",
        takes_value = true,
        value_name = "FILENAME"
    )]
    pub output: Option<PathBuf>,

    /// Enable the debugging mode
    #[structopt(short = "d", long = "debug", takes_value = false)]
    pub debug: bool,
}

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
pub struct StopConditionsConfig {
    /// A count of packets for sending. When this limit is reached, then
    /// the program will exit.
    #[structopt(
        short = "p",
        long = "packets-count",
        takes_value = true,
        value_name = "POSITIVE-INTEGER",
        default_value = "18446744073709551615",
        parse(try_from_str = "parse_non_zero_usize")
    )]
    pub packets_count: NonZeroUsize,

    /// A whole test duration. By default, a test will be performed until
    /// you explicitly stop the process.
    #[structopt(
        long = "test-duration",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "64years 64hours 64secs",
        parse(try_from_str = "parse_duration")
    )]
    pub test_duration: Duration,
}

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
pub struct PacketConfig {
    /// A count of packets for sending. When this limit is reached, then
    /// the program will exit. The default value is 65000.
    #[structopt(
        short = "l",
        long = "packet-length",
        takes_value = true,
        value_name = "POSITIVE-INTEGER",
        parse(try_from_str = "parse_non_zero_usize")
    )]
    pub packet_length: Option<NonZeroUsize>,

    /// A file for sending instead of random-generated packets. You cannot
    /// use this option and the `--packet-length` together.
    #[structopt(
        short = "f",
        long = "send-file",
        takes_value = true,
        value_name = "FILENAME"
    )]
    pub send_file: Option<PathBuf>,
}

impl ArgsConfig {
    pub fn setup() -> ArgsConfig {
        let matches = ArgsConfig::clap()
            .group(ArgGroup::with_name("message").args(&["send_file", "packet_length"]))
            .get_matches();

        let mut args_config = ArgsConfig::from_clap(&matches);

        // If an user hasn't specified a file, then set the default packet
        // length
        if !matches.is_present("send_file") {
            args_config.packet_config.packet_length =
                Some(unsafe { NonZeroUsize::new_unchecked(65000) });
        }

        args_config
    }
}

pub fn parse_non_zero_usize(number: &str) -> Result<NonZeroUsize, NonZeroUsizeError> {
    let number: usize = number
        .parse()
        .map_err(|error| NonZeroUsizeError::InvalidFormat(error))?;

    NonZeroUsize::new(number).ok_or(NonZeroUsizeError::ZeroValue)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonZeroUsizeError {
    InvalidFormat(ParseIntError),
    ZeroValue,
}

impl Display for NonZeroUsizeError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            NonZeroUsizeError::InvalidFormat(error) => write!(fmt, "{}", error),
            NonZeroUsizeError::ZeroValue => write!(fmt, "The value equals to zero"),
        }
    }
}

impl Error for NonZeroUsizeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_non_zero_usize() {
        unsafe {
            // Check that ordinary values are parsed correctly
            assert_eq!(
                parse_non_zero_usize("1"),
                Ok(NonZeroUsize::new_unchecked(1))
            );
            assert_eq!(
                parse_non_zero_usize("3"),
                Ok(NonZeroUsize::new_unchecked(3))
            );
            assert_eq!(
                parse_non_zero_usize("26655"),
                Ok(NonZeroUsize::new_unchecked(26655))
            );
            assert_eq!(
                parse_non_zero_usize("+75"),
                Ok(NonZeroUsize::new_unchecked(75))
            );
        }
    }

    #[test]
    fn parses_invalid_non_zero_usize() {
        let panic_if_invalid = |string| {
            if let Ok(_) = parse_non_zero_usize(string) {
                panic!("Parses invalid formatted usize correctly");
            }
        };

        // Invalid numbers must produce the invalid format error
        panic_if_invalid("   ");

        panic_if_invalid("abc5653odr!");
        panic_if_invalid("6485&02hde");

        panic_if_invalid("-565642");
        panic_if_invalid(&"2178".repeat(50));

        // Check that the zero value is not allowed
        assert_eq!(parse_non_zero_usize("0"), Err(NonZeroUsizeError::ZeroValue));
    }
}
