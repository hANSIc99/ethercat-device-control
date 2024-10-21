mod ec_device;
mod misc;

use std::path::Path;
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
    fwupdate(FwUpdateArgs),
    setstate(SetStateArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct FwUpdateArgs {
    fileName : String
}


#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct SetStateArgs {
    #[command(subcommand)]
    command: EcState,
}




#[tokio::main(flavor = "current_thread")]
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
        Commands::fwupdate(arg) => {
            println!("Firmware Update: {:?}", arg.fileName);
            let fw_path = Path::new(&arg.fileName);
            match fw_path.try_exists() {
                Ok(_r) =>{println!("Path exist")},
                Err(_e) => {println!("Path does not exist")}
            }



            let filename = fw_path  .file_stem()
                                            .ok_or_else(|| AdsError{n_error : 0x706, s_msg : String::from("Invalid Data - Filename not specified")})?;


            let f_str = filename.to_str().ok_or_else(|| AdsError{n_error : 0x706, s_msg : String::from("Invalid Data - Filename not specified")})?;
            println!("f_str A {:?}", f_str);
            let wr_hdl = ec_device.ec_foe_open_wr(f_str).await?;

            println!("Write handle: {:?}", wr_hdl);

            //fw_path.try_exists().map_or_else(|v| {println!("Path does not exist")}, |x| {println!("Path exist")});
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