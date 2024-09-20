
use ads_client::{Client, AdsTimeout, Result};
use crate::misc::{EcState, EcErrState, EcLinkState, EcLinkPort, EcSDeviceError};

type EtherCATSlaveState = std::result::Result<EcState, EcSDeviceError>;


#[derive(Debug)]
pub struct EtherCATDevice {
    state : EtherCATSlaveState,
}

impl EtherCATDevice {
    //pub fn new(value: [u8; 2] ) -> Self {
    pub async fn new(addr :&str) -> Result<Self> {
        
        let ads_client = Client::new(addr, 0xFFFF, AdsTimeout::DefaultTimeout).await?;
        let mut ec_state_raw : [u8; 2] = [0; 2];
        let rd_result = ads_client.read(0x00000009, 1002, &mut ec_state_raw).await?;

        // println!("value[0] : {:?}", value[0]); // 0x00
        // println!("value[1] : {:?}", value[1]); // 0x08

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
                    link_state : link_state,
                    link_port : link_port
                })
            })
        } else {
            Ok(EtherCATDevice {
                state : EtherCATSlaveState::Ok(ec_state)
            })
        }
    }
}