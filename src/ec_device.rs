
use ads_client::{Client, AdsTimeout, Result};
use crate::misc::{EcState, EcErrState, EcLinkState, EcLinkPort, EcSDeviceError};

type EtherCATSlaveState = std::result::Result<EcState, EcSDeviceError>;


#[derive(Debug)]
pub struct EtherCATDevice {
    state           : EtherCATSlaveState,
    ads_ec_master   : Client,
    device_addr     : u32
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
                device_addr     : device_no
            },
        )
        } else {
            Ok(EtherCATDevice {
                state           : EtherCATSlaveState::Ok(ec_state),
                ads_ec_master   : ads_client,
                device_addr     : device_no
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

    pub async fn request_ec_state(&mut self, ec_state : EcState) -> Result<()>{

        
        if self.state.as_ref().is_ok_and(|ecstate| ec_state.eq(ecstate)){
            println!("EtherCAT device already in state {:?}", ec_state);
            return Ok(())
        }
        
        self.update_ec_state().await?;

        //Self::get_ec_state().await?;
        Ok(())
    }

}