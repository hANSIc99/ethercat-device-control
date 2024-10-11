mod ec_device;
mod ec_state;
mod misc;

use ads_client::AdsError;
use ec_device::EtherCATDevice; 
use misc::EcState;
use regex::Regex;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tokio::runtime::Runtime;
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
        return Ok(())
    };

    
    let re_device_id = Regex::new(r"^(100[1-9]|10[1-9]\d|1[1-9]\d{2}|[2-9]\d{3})$").unwrap();
    if(re_device_id.is_match(&args.DeviceAddress)){
        println!("Valid Device Address found!");
        println!("Device Address: {:?}", args.DeviceAddress);
        
    } else {
        println!("Invalid Device Address");
        return Ok(())
    }

    let port = args.DeviceAddress.parse::<u32>().unwrap();

    //let rt = Runtime::new().unwrap();
    //let ec_device = rt.block_on( EtherCATDevice::new(&args.AmsNetId, port)).unwrap(); // TODO
    let mut ec_device = EtherCATDevice::new(&args.AmsNetId, port).await?;
    // match rt.block_on(ads_client.read_state()) {
    //     Ok(state) => println!("State: {:?}", state),
    //     Err(err) => println!("Error: {}", err.to_string())
    // }

    println!("EtherCAT Device: {:?}", ec_device);

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
                    //ec_device.request_ec_state(EcState::PreOp).await?;
                    let test = tokio::join!(ec_device.request_ec_state(EcState::PreOp));
                    println!("EtherCAT Device: {:?}", ec_device);
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


    Ok(())
}