
use ads_client::{Client, AdsTimeout, Result};
use crate::misc::{EcState, EcErrState, EcLinkState, EcLinkPort, EcSDeviceError};

type EtherCATSlaveState = std::result::Result<EcState, EcSDeviceError>;


#[derive(Debug)]
pub struct EtherCATDevice {
    state           : EtherCATSlaveState,
    ads_ec_master   : Client,
    device_addr     : u32,
    net_id          : String
}

impl EtherCATDevice {
    //pub fn new(value: [u8; 2] ) -> Self {
    pub async fn new(addr :&str, device_no: u32) -> Result<Self> {
        
        let ads_client = Client::new(addr, 0xFFFF, AdsTimeout::DefaultTimeout).await?; // Zu EtherCATDevice ??
        let mut ec_state_raw : [u8; 2] = [0; 2];
        let n_bytes_read = ads_client.read(0x00000009, device_no, &mut ec_state_raw).await?;

        if n_bytes_read < 2 {
            print!("Error"); // TODO
        }

        let ec_state = EcState::from(ec_state_raw[0] & 0x0F);
        println!("EcStateMachine state {:?}", ec_state);

        let ec_err_state = EcErrState::from(ec_state_raw[0] & 0xF0);
        println!("EcErrorState {:?}", ec_err_state);

        let link_state = EcLinkState::from(ec_state_raw[1] & 0x0F);
        println!("EcLinkState {:?}", link_state);

        let link_port = EcLinkPort::from(ec_state_raw[1] & 0xF0);
        println!("EcLinkPort {:?}", link_port);

        if ec_err_state != EcErrState::Ok || link_state != EcLinkState::Ok {     
            Ok(EtherCATDevice {
                state : Err(EcSDeviceError {
                    ec_state : ec_state,
                    ec_err_state : ec_err_state,
                    link_state  : link_state,
                    link_port   : link_port
                }),
                ads_ec_master   : ads_client,
                device_addr     : device_no,
                net_id          : String::from(addr)
            },
        )
        } else {
            Ok(EtherCATDevice {
                state           : EtherCATSlaveState::Ok(ec_state),
                ads_ec_master   : ads_client,
                device_addr     : device_no,
                net_id          : String::from(addr)
            })
        }
    }

    // async fn read_state(&self) -> Result<()>{

    // }

    pub async fn update_ec_state(&mut self) -> Result<()>{

        let mut ec_state_raw : [u8; 2] = [0; 2];
        let n_bytes_read = self.ads_ec_master.read(0x00000009, self.device_addr, &mut ec_state_raw).await?;

        if n_bytes_read < 2 {
            print!("Error"); // TODO
        }

        let ec_state = EcState::from(ec_state_raw[0] & 0x0F);
        println!("EcStateMachine state {:?}", ec_state);

        let ec_err_state = EcErrState::from(ec_state_raw[0] & 0xF0);
        println!("EcErrorState {:?}", ec_err_state);

        let link_state = EcLinkState::from(ec_state_raw[1] & 0x0F);
        println!("EcLinkState {:?}", link_state);

        let link_port = EcLinkPort::from(ec_state_raw[1] & 0xF0);
        println!("EcLinkPort {:?}", link_port);

        if ec_err_state != EcErrState::Ok || link_state != EcLinkState::Ok {  
            self.state = Err(EcSDeviceError {
                ec_state : ec_state,
                ec_err_state : ec_err_state,
                link_state  : link_state,
                link_port   : link_port
            });
        } else {
            self.state = Ok(ec_state);
        }

        Ok(())
    }

    // pub async fn set_ec_state(&mut self, req_ec_state : EcState) -> Result<()> {

    //     self.ads_ec_master.write(0x00000009, self.device_addr, &(req_ec_state as u16).to_ne_bytes() ).await?;
    //     Ok(())
    // }

    pub async fn request_ec_state(&mut self, req_ec_state : EcState) -> Result<()>{

        self.update_ec_state().await?;

        if self.state.as_ref().is_ok_and(|ecstate| req_ec_state.eq(ecstate)){
            println!("EtherCAT device already in state {:?}", req_ec_state);
            return Ok(())
        }
        
        //let ec_state_raw = (req_ec_state as u16).to_ne_bytes();
        self.ads_ec_master.write(0x00000009, self.device_addr, &(req_ec_state as u16).to_ne_bytes() ).await
        // Todo: While schleife bis ECState erreicht ist oder Error
    }

    pub async fn ec_foe_open_wr(&self, f_name : &str) -> Result<u32>{
        let mut rd_raw : [u8; 199] = [0; 199];

        println!("f_name B {:?}", f_name);
        // Read length: 199
        // Write length: 29 (Filename Size)

        // Neuer ADS Client: Port = Slave ADDR
        // FoE Open Write
        let foe_ads = Client::new(&self.net_id, self.device_addr.try_into().unwrap(), AdsTimeout::DefaultTimeout).await?;

        let rd_length = foe_ads.read_write(0xF402, 0, &mut rd_raw, f_name.as_bytes()).await?;
        //self.ads_ec_master.read_write(0x4F02, , read_data, write_data)
        
        Ok(u32::from_ne_bytes(rd_raw[0..4].try_into().unwrap()))
    }

}