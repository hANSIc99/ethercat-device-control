use clap::Subcommand;
use clap::{Args, Parser, ValueEnum};

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq)]
#[derive(Subcommand)]
#[command(rename_all = "lower")]
pub enum EcState {

    Init            = 0x0001,
    PreOp           = 0x0002,
    Boot            = 0x0003,
    SafeOp          = 0x0004,
    Op              = 0x0008
}

impl From<u8> for EcState {
    fn from(x: u8) -> Self {
        match x {
            0x01 => EcState::Init,
            0x02 => EcState::PreOp,
            0x03 => EcState::Boot,
            0x04 => EcState::SafeOp,
            0x08 => EcState::Op,
            _ => panic!("Invalid value: {x}")
        }
    }
}


#[derive(Debug)]
#[derive(PartialEq)]
pub enum EcErrState {
    Ok              = 0x00,
    Err             = 0x10,
    VprsErr         = 0x20,
    InitErr         = 0x40,
    Disabled        = 0x80
}

impl From<u8> for EcErrState {
    fn from(x: u8) -> Self {
        match x {
            0x00 => EcErrState::Ok,
            0x10 => EcErrState::Err,
            0x20 => EcErrState::VprsErr,
            0x40 => EcErrState::InitErr,
            0x80 => EcErrState::Disabled,
            _ => panic!("Invalid value: {x}")
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum EcLinkState {
    Ok              = 0x0000,
    NotPresent      = 0x0100,
    LinkError       = 0x0200,
    MissLink        = 0x0400,
    UnexpectedLink  = 0x0800,
}

impl From<u8> for EcLinkState {
    fn from(x: u8) -> Self {
        let word: u16 = (x as u16) << 8;

        match word {
            0x0000 => EcLinkState::Ok,
            0x0100 => EcLinkState::NotPresent,
            0x0200 => EcLinkState::LinkError,
            0x0400 => EcLinkState::MissLink,
            0x0800 => EcLinkState::UnexpectedLink,
            _ => panic!("Invalid value: {x}")
        }
    }
    
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum EcLinkPort {
    None            = 0x0000,
    ComPortA        = 0x1000,
    ComPortB        = 0x2000,
    ComPortC        = 0x4000,
    ComPortD        = 0x8000
}

impl From<u8> for EcLinkPort {
    fn from(x: u8) -> Self {
        let word: u16 = (x as u16) << 8;

        match word {
            0x0000 => EcLinkPort::None,
            0x1000 => EcLinkPort::ComPortA,
            0x2000 => EcLinkPort::ComPortB,
            0x4000 => EcLinkPort::ComPortC,
            0x8000 => EcLinkPort::ComPortD,
            _ => panic!("Invalid value: {x}")
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct EcSDeviceError {
    pub ec_state        : EcState,
    pub ec_err_state    : EcErrState,
    pub link_state      : EcLinkState,
    pub link_port       : EcLinkPort
}