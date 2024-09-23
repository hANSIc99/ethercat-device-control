mod ec_device;
mod misc;

use ads_client::AdsError;
use ec_device::EtherCATDevice; 
use misc::EcState;
use regex::Regex;
use clap::{Args, Parser, Subcommand, ValueEnum};

pub type Result<T> = std::result::Result<T, AdsError>;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
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
    fwupdate,
    setstate(SetStateArgs),

}


#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct SetStateArgs {
    #[command(subcommand)]
    command: EcStateCmd,
}

#[derive(Debug, Subcommand)]
pub enum EcStateCmd {
    init,
    preop,
    boot,
    safeop,
    op
}




#[tokio::main]
async fn main() -> Result<()> {

    let args = Cli::parse();

    let re_net_id = Regex::new(r"^(\d{1,3}\.){5}\d{1,3}$").unwrap();
    if (re_net_id.is_match(&args.AmsNetId)){
        println!("Valid AmsNetId found!");
        println!("AmsNetId: {:?}", args.AmsNetId);
    } else {
        println!("Invalid AmsNetId");
    };

    
    let re_device_id = Regex::new(r"^(100[1-9]|10[1-9]\d|1[1-9]\d{2}|[2-9]\d{3})$").unwrap();

    match args.command {
        Commands::fwupdate => {
            println!("Firmware Update!");
        }
        Commands::setstate(args) => {
            //let stash_cmd = args.command.unwrap_or(EcStateCmd::op(stash.args));
            //let ec_state_arg = args.command;
            match args.command {
                EcStateCmd::op => {
                    println!("Set EtherCAT device to Op");
                },
                EcStateCmd::preop => {
                    println!("Set EtherCAT device to PreOp");
                },
                EcStateCmd::safeop => {
                    println!("Set EtherCAT device to SafeOp");
                },
                EcStateCmd::init => {
                    println!("Set EtherCAT device to Init");

                }EcStateCmd::boot => {
                    println!("Set EtherCAT device to Boot");
                }
            }
            let y = 3;
        }
    }

    // let ec_slave = EtherCATDevice::new("5.80.201.232.2.1").await?;
    // println!("ec_slave : {:?}", ec_slave); // 0x08
    // let x = 2;

    Ok(())
}