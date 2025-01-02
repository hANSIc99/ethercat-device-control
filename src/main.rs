#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused)]
mod ec_device;
mod misc;

use std::{fs::File, io::Read};
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::thread::{self, sleep};
use std::time;
use ads_client::AdsError;
use ec_device::EtherCATDevice; 
use misc::EcState;
use regex::Regex;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tokio::runtime::Runtime;
pub type Result<T> = std::result::Result<T, AdsError>;

use log::{trace, debug, info, warn, error};
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};


#[derive(Parser)]
#[command(version, about, long_about = "Long about instead of None")]

// Todo: Log optionen

struct Cli {

    #[clap(value_parser, required = true, help = "AmsNetId of the EtherCAT Master")]
    AmsNetId: String,
    #[clap(value_parser, required = true, help = "EtherCAT device number")]
    DeviceAddress: String,
    // /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,

    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand)]
enum Commands {
    fwupdate(FwUpdateArgs),
    setstate(SetStateArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct FwUpdateArgs {
    fileName : String
}


#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true, after_help = "after help string", after_long_help = "after long help")]
struct SetStateArgs {
    #[command(subcommand)]
    command: EcState,
}




#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {

    // Define appender "stderr"
    let stderr = ConsoleAppender::builder()
        // https://github.com/estk/log4rs/blob/main/src/encode/pattern/mod.rs
        // https://docs.rs/chrono/latest/chrono/format/strftime/
        //.encoder(Box::new(PatternEncoder::new("{d:<35.35} - {l} - {f}:{L}- {m}{n}")))
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f)} - {l} - {f}:{L}- {m}{n}")))
        .build();
    
    // Define appender "logfile"
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f)} - {l} - {f}:{L}- {m}{n}")))
        .build("log/requests.log")
        .unwrap();

    let config = Config::builder()
        // .appender(
        //     Appender::builder()
        //         .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
        //         .build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
                .build("stderr", Box::new(stderr)))
        .build(
            Root::builder()
                //.appender("logfile")
                .appender("stderr")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

        let _handle = log4rs::init_config(config).unwrap();


    let args = Cli::parse();

    let re_net_id = Regex::new(r"^(\d{1,3}\.){5}\d{1,3}$").unwrap();
    if (re_net_id.is_match(&args.AmsNetId)){
        debug!("Valid AmsNetId found: {:?}", args.AmsNetId);
    } else {
        error!("Invalid AmsNetId");
        return Ok(())
    };

    
    let re_device_id = Regex::new(r"^(100[1-9]|10[1-9]\d|1[1-9]\d{2}|[2-9]\d{3})$").unwrap();
    if re_device_id.is_match(&args.DeviceAddress) {
        debug!("Valid Device Address found: {:?}", args.DeviceAddress);  
    } else {
        error!("Invalid Device Address");
        return Ok(())
    }

    let port = args.DeviceAddress.parse::<u32>().unwrap();

    let mut ec_device = EtherCATDevice::new(&args.AmsNetId, port).await?;

    trace!("EtherCAT Device: {:?}", ec_device);

    match args.command {
        Commands::fwupdate(arg) => {
            trace!("Specified filename: {:?}", arg.fileName);
            let fw_path = Path::new(&arg.fileName);
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


            let f_str = filename.to_str().ok_or_else(|| AdsError{n_error : 0x706, s_msg : String::from("Invalid Data - Filename not specified")})?;

            // Datei einlesen
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

            ec_device.ec_foe_close(f_hdl).await?;
            println!("FW update done - device need power cycle");
            Ok(())
        }
        Commands::setstate(args) => {
            //let stash_cmd = args.command.unwrap_or(EcStateCmd::op(stash.args));
            //let ec_state_arg = args.command;
            //let test = tokio::join!(ec_device.request_ec_state(args.command));
            return ec_device.request_ec_state(args.command).await;
        }
    }


    //Ok(())
}