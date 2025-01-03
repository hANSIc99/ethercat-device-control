mod ec_device;
mod misc;

use std::{fs::File, io::Read};
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use ads_client::AdsError;
use ec_device::EtherCATDevice; 
use misc::EcState;
use regex::Regex;
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{trace, debug, info, error};
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

pub type Result<T> = std::result::Result<T, AdsError>;

#[derive(Parser)]
#[command(version, about, long_about = "Long about instead of None")]

// command attributes https://docs.rs/clap/latest/clap/_derive/index.html

struct Cli {

    #[clap(value_parser, value_name = "AmsNetId", required = true, help = "AmsNetId of the EtherCAT Master")]
    ams_net_id: String,
    #[clap(value_parser, value_name = "DeviceAddress", required = true, help = "EtherCAT device number")]
    device_address: String,
    #[clap(value_parser, value_name = "LogLevel", required = false)]
    log_level : Option<LogLevel>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Debug, Clone)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

#[derive(Subcommand)]
enum Commands {
    #[command(rename_all = "lower" )]
    FwUpdate(FwUpdateArgs),
    #[command(rename_all = "lower" )]
    SetState(SetStateArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct FwUpdateArgs {
    #[clap(value_name = "FileName")]
    file_name : String
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true, after_help = "after help string", after_long_help = "after long help")]
struct SetStateArgs {
    #[command(subcommand)]
    command: EcState,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {

    let args = Cli::parse();

    let loglvl = args.log_level.map_or_else(
        || log::LevelFilter::Error, |lvl| {
        match lvl {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace
        }
    });

    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f)} - {l} - {f}:{L}- {m}{n}")))
        .build();

    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Trace)))
                .build("stderr", Box::new(stderr)))
        .build(
            Root::builder()
                .appender("stderr")
                .build(loglvl),
        )
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
    let re_net_id = Regex::new(r"^(\d{1,3}\.){5}\d{1,3}$").unwrap();

    if re_net_id.is_match(&args.ams_net_id){
        debug!("Valid AmsNetId found: {:?}", args.ams_net_id);
    } else {
        error!("Invalid AmsNetId");
        return Ok(())
    };
    
    let re_device_id = Regex::new(r"^(100[1-9]|10[1-9]\d|1[1-9]\d{2}|[2-9]\d{3})$").unwrap();
    
    if re_device_id.is_match(&args.device_address) {
        debug!("Valid Device Address found: {:?}", args.device_address);  
    } else {
        error!("Invalid Device Address");
        return Ok(())
    }

    let port = args.device_address.parse::<u32>().unwrap();
    let mut ec_device = EtherCATDevice::new(&args.ams_net_id, port).await?;

    trace!("EtherCAT Device: {:?}", ec_device);

    match args.command {
        Commands::FwUpdate(arg) => {
            trace!("Specified filename: {:?}", arg.file_name);
            let fw_path = Path::new(&arg.file_name);
            match fw_path.try_exists() {
                Ok(res) =>{
                    if res {debug!("Path check suceeded: {:?}", fw_path);}
                    else {
                        error!("Path check failed - path does no exist");
                        return Err(AdsError{n_error : 1804, s_msg : String::from("Not found files")});
                    }
                },
                Err(_e) => {
                    error!("Path check failed");
                    return Err(AdsError{n_error : 1792, s_msg : String::from("General device error")});
                }
            }

            let filename = fw_path  .file_stem()
                                            .ok_or_else(|| AdsError{n_error : 0x706, s_msg : String::from("Invalid Data - Filename not specified")})?;

            let f_str = filename.to_str()
                                            .ok_or_else(|| AdsError{n_error : 0x706, s_msg : String::from("Invalid Data - Filename not specified")})?;

            // Read file from disk
            let f = File::open(fw_path)?;
            let f_length = f.metadata().unwrap().len();
            info!("File length: {:?}", f_length);
            let mut f_reader = BufReader::new(f);

            ec_device.request_ec_state(EcState::Boot).await?;

            // File Open
            let f_hdl = ec_device.ec_foe_open_wr(f_str).await?;

            if f_hdl <= 0 {
                error!("No file handle");
                return Ok(())
            } else {
                trace!("File handle created: {:?}", f_hdl);
            }

            let mut chunk : [u8; 0x4000] = [0; 0x4000];
            let mut sum_read = 0;

            loop {
                let bytes_read = f_reader.read(&mut chunk)?;
                sum_read += bytes_read;

                // Calculate process
                let bar : f32 = (sum_read as f32) / (f_length as f32) * 100.0;

                print!("\rLoad [%]: {:2.1}", bar);
                let _ = std::io::stdout().flush();
                if bar >= 100.0 {
                    print!("\n Waiting for write process to be finished - this can take several minutes...");
                }

                if bytes_read > 0 {
                    ec_device.ec_foe_write(f_hdl, &chunk[..bytes_read]).await?;
                } else {
                    break;
                }

            }
            println!("\r\n");    
            println!("FW update done - device need power cycle");
            ec_device.ec_foe_close(f_hdl).await
        }
        Commands::SetState(args) => {
            return ec_device.request_ec_state(args.command).await;
        }
    }
}