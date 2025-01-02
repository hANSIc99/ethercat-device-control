#![allow(dead_code)]
#![allow(unused)]
use std::thread::{self, sleep};
use std::time;
use log::{trace, debug, info, warn, error};
use ads_client::{Client, AdsTimeout, Result, AdsError};
use crate::misc::{EcState, EcErrState, EcLinkState, EcLinkPort, EcSDeviceError};

type EtherCATSlaveState = std::result::Result<EcState, EcSDeviceError>;


#[derive(Debug)]
pub struct EtherCATDevice {
    state           : EtherCATSlaveState,
    ads_ec_master   : Client,
    pub ads_ec_device   : Client,
    device_addr     : u32,
    net_id          : String
}

impl EtherCATDevice {
    //pub fn new(value: [u8; 2] ) -> Self {
    pub async fn new(addr :&str, device_no: u32) -> Result<Self> {
        
        let ads_client_master = Client::new(addr, 0xFFFF, AdsTimeout::DefaultTimeout).await?;
        let ads_client_device = Client::new(addr, device_no.try_into().unwrap(), AdsTimeout::CustomTimeout(200)).await?;
        let mut ec_state_raw : [u8; 2] = [0; 2];
        let n_bytes_read = ads_client_master.read(0x00000009, device_no, &mut ec_state_raw).await?;

        if n_bytes_read < 2 {
            error!("Error - ADS response less than 2 bytes");
            return Err(AdsError{n_error : 1798, s_msg : String::from("Invalid data values")});
        }

        let ec_state = EcState::from(ec_state_raw[0] & 0x0F);
        trace!("EcStateMachine state {:?}", ec_state);

        let ec_err_state = EcErrState::from(ec_state_raw[0] & 0xF0);
        trace!("EcErrorState {:?}", ec_err_state);

        let link_state = EcLinkState::from(ec_state_raw[1] & 0x0F);
        trace!("EcLinkState {:?}", link_state);

        let link_port = EcLinkPort::from(ec_state_raw[1] & 0xF0);
        trace!("EcLinkPort {:?}", link_port);

        if ec_err_state != EcErrState::Ok || link_state != EcLinkState::Ok {     
            Ok(EtherCATDevice {
                state : Err(EcSDeviceError {
                    ec_state : ec_state,
                    ec_err_state : ec_err_state,
                    link_state  : link_state,
                    link_port   : link_port
                }),
                ads_ec_master   : ads_client_master,
                ads_ec_device   : ads_client_device,
                device_addr     : device_no,
                net_id          : String::from(addr)
            },
        )
        } else {
            Ok(EtherCATDevice {
                state           : EtherCATSlaveState::Ok(ec_state),
                ads_ec_master   : ads_client_master,
                ads_ec_device   : ads_client_device,
                device_addr     : device_no,
                net_id          : String::from(addr)
            })
        }
    }

    pub async fn update_ec_state(&mut self) -> Result<()>{

        let mut ec_state_raw : [u8; 2] = [0; 2];
        let n_bytes_read = self.ads_ec_master.read(0x00000009, self.device_addr, &mut ec_state_raw).await?;

        if n_bytes_read < 2 {
            error!("Error - ADS response less than 2 bytes");
            return Err(AdsError{n_error : 1798, s_msg : String::from("Invalid data values")});
        }

        let ec_state = EcState::from(ec_state_raw[0] & 0x0F);
        trace!("EcStateMachine state {:?}", ec_state);

        let ec_err_state = EcErrState::from(ec_state_raw[0] & 0xF0);
        trace!("EcErrorState {:?}", ec_err_state);

        let link_state = EcLinkState::from(ec_state_raw[1] & 0x0F);
        trace!("EcLinkState {:?}", link_state);

        let link_port = EcLinkPort::from(ec_state_raw[1] & 0xF0);
        trace!("EcLinkPort {:?}", link_port);

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
        
        self.ads_ec_master.write(0x00000009, self.device_addr, &(req_ec_state as u16).to_ne_bytes() ).await?;

        loop {
            self.update_ec_state().await?;
            if self.state.as_ref().is_ok_and( |ec_state| req_ec_state.eq(ec_state) ) {break}
            if self.state.is_err() {break}
            sleep(std::time::Duration::from_millis(100));
        }

        match &self.state {
            Ok(ec_state) => {println!("State switched to {:?}", ec_state);},
            Err(err_state) => {
                println!("EC Error - State: {:?}, Error: {:?}, Link: {:?}, Port: {:?}", err_state.ec_state, err_state.ec_err_state, err_state.link_state, err_state.link_port);
            }
        };
        
        Ok(())
    }

    pub async fn ec_foe_open_wr(&self, f_name : &str) -> Result<u32>{
        let mut rd_raw : [u8; 199] = [0; 199];

        trace!("f_name B {:?}", f_name);
        let rd_length = self.ads_ec_device.read_write(0xF402, 0, &mut rd_raw, f_name.as_bytes()).await?;
        
        Ok(u32::from_ne_bytes(rd_raw[0..4].try_into().unwrap()))
    }

    pub async fn ec_foe_close(&self, hdl : u32) -> Result<()>{
        let mut rd_raw : [u8; 199] = [0; 199];
        let wr_raw : [u8; 0] = [0; 0];

        self.ads_ec_device.read_write(0xF403, hdl, &mut rd_raw, &wr_raw).await?;
        Ok(())
    }

    pub async fn ec_foe_write(&self, f_hdl : u32, data : &[u8]) -> Result<()>{
        let mut rd_raw : [u8; 199] = [0; 199];
        self.ads_ec_device.read_write(0xF405, f_hdl.try_into().unwrap(), &mut rd_raw, data).await?;
        Ok(())
    }

}