mod ec_device;
mod misc;

use ads_client::AdsError;
use ec_device::EtherCATDevice; 


pub type Result<T> = std::result::Result<T, AdsError>;

#[tokio::main]
async fn main() -> Result<()> {

    let ec_slave = EtherCATDevice::new("5.80.201.232.2.1").await?;
    println!("ec_slave : {:?}", ec_slave); // 0x08
    let x = 3;

    Ok(())
}