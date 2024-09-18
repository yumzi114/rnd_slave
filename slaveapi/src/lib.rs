

use core::str;
use std::io::{self, Error, ErrorKind};
use serde_derive::{Serialize, Deserialize};
use bytes::{buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use serde_with::{self, serde_as};
use defaults::Defaults;

#[cfg(unix)]
// const SERIAL_DEVICE: &'static str = env!("SERIAL_DEVICE");
const SERIAL_DEVICE: &'static str = "/dev/ttyAMA2";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";


pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // let newline = src.as_ref().iter().position(|b| *b == b'j');
        let start = src.as_ref().iter().position(|x| *x == 0xFC);
        if let Some(n) = start {
            let line = src.split_to(n+1);
            let line_list = line.to_vec();
            if line_list.len()==14&&line_list[0]==0xAF&&line_list[1]==11{
                let mut packet = Packet::default();
                if let Ok(_)=packet.parser(line_list){
                    if let Ok(_)=packet.is_checksum(){
                        return Ok(Some(packet));
                    }
                }
                // if line_list[3]==00{
                //     return Err(Error::other("Device S/N Error"));
                // }
                // return Ok(Some(line_list));
            }
            else {
                return Ok(None)
                // return Err(Error::new(ErrorKind::NotConnected, "Device Not Connected"));
            }

        }
        // if let Some(n) = newline {
        //     let line = src.split_to(n + 1);
        //     return match str::from_utf8(line.as_ref()) {
        //         Ok(s) => Ok(Some(s.to_string())),
        //         Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
        //     };
        // }
        Ok(None)
    }
}

impl Encoder<Vec<u8>> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, buf: &mut BytesMut) -> Result<(), Self::Error> {
        
        for i in item{
            buf.put_u8(i);
        }
        
        Ok(())
    }
}

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize,Defaults,Clone,Copy)]
pub struct Packet {
    #[def = "0xAF"]
    start: u8,
    #[def = "0x0B"]
    length: u8,
    #[def = "0x0000"]
    reserved: u16,
    #[def = "0x00"]
    pub command: u8,
    #[def = "0x0000"]
    pub pannel_up: i16,
    #[def = "0x0000"]
    pub pannel_down: i16,
    #[def = "0x0000"]
    pub overload: i16,
    #[def = "0x00"]
    pub sensor_state: u8,
    #[def = "0x00"]
    pub checksum: u8,
    #[def = "0xFC"]
    end: u8,
}
impl Packet{
    pub fn parser(&mut self, buf:Vec<u8>)->Result<(),io::Error>{
        if buf.len()==14{
            self.start = buf[0];
            self.length = u8::from_be_bytes([buf[1]]);
            self.reserved = u16::from_be_bytes([buf[2],buf[3]]);
            self.command = u8::from_be_bytes([buf[4]]);
            self.pannel_up = i16::from_be_bytes([buf[5],buf[6]]);
            self.pannel_down = i16::from_be_bytes([buf[7],buf[8]]);
            self.overload = i16::from_be_bytes([buf[9],buf[10]]);
            self.sensor_state = u8::from_be_bytes([buf[11]]);
            self.checksum = u8::from_be_bytes([buf[12]]);
            self.end = u8::from_be_bytes([buf[13]]);

            return Ok(())    
        }
        else{
            return Err(io::Error::new(ErrorKind::Other, "Fail Check buf"));
        }
    }
    pub fn is_checksum (&self)->Result<(),String>{
        let mut sumdata:u128=0;
        self.reserved.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.command.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.pannel_up.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.pannel_down.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.overload.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.sensor_state.to_be_bytes().map(|x|sumdata+=u128::from(x));
        let hex_str = format!("{:#x}",sumdata);
        let check_sum =hex::decode(&hex_str[hex_str.len()-2..]);
        if let Ok(data)=check_sum{
            if self.checksum!=data[0]{
                return Err("Fail checksum Err".to_string());
            }
            return Ok(());
        }
        else{
            let hex_str = hex_str.trim_start_matches("0x");
            let checksum=u8::from_str_radix(hex_str,16).unwrap();
            if self.checksum!=checksum{
                return Err("Fail checksum Err".to_string());
            }
            return Ok(());
        }
    }
}