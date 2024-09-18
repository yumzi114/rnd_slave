
use serde::{Deserialize, Serialize};
use serde_derive;
use serde_hex;
use defaults::Defaults;
use serde_hex::{SerHex,StrictPfx};
use tokio::{self};
use mini_redis::{client, Result};
use core::result::Result as ResultC;
use std::{thread, time::Duration};
use futures::{stream::StreamExt, SinkExt};
use futures_channel::mpsc;
use tracing::{info, trace, warn, error};
use tokio_serial::{SerialPort, SerialPortBuilderExt, StopBits};
use tokio_util::codec::{Decoder, Encoder};
use slaveapi::{self, LineCodec};
#[cfg(unix)]
// const SERIAL_DEVICE: &'static str = env!("SERIAL_DEVICE");
const SERIAL_DEVICE: &'static str = "/dev/ttyAMA2";
// const SERIAL_DEVICE : "/dev/ttyACM1";

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize,Defaults,Clone,Copy)]
struct Responese{
    #[serde(with = "SerHex::<StrictPfx>")]
    #[def = "0xAF"]
    start: u8,
    #[serde(with = "SerHex::<StrictPfx>")]
    #[def = "0x03"]
    length: u8,
    #[serde(with = "SerHex::<StrictPfx>")]
    #[def = "0x00"]
    command: u8,
    #[serde(with = "SerHex::<StrictPfx>")]
    #[def = "0x00"]
    data: u8,
    #[serde(with = "SerHex::<StrictPfx>")]
    #[def = "0x00"]
    checksum: u8,
    #[serde(with = "SerHex::<StrictPfx>")]
    #[def = "0xFC"]
    end: u8,
}
impl Responese {
    fn parser (&mut self, buf : &Vec<u8>)->ResultC<(),String>{
        if buf.len()==6{
            // let mut resp=Responese::default();
            self.start = u8::from_be_bytes([buf[0]]);
            self.length = u8::from_be_bytes([buf[1]]);
            self.command = u8::from_be_bytes([buf[2]]);
            self.data = u8::from_be_bytes([buf[3]]);
            self.checksum = u8::from_be_bytes([buf[4]]);
            self.end = u8::from_be_bytes([buf[5]]);
            return Ok(())
        }
        else {
            return Err("Fail Parsing".to_string())
        }
    }
    fn is_checksum(&self)->ResultC<(),String>{
        let mut sumdata:u128=0;
        // for i in list[1..3].iter() {
        //     let num = *i;
        //     num.to_be_bytes().map(|x|sumdata+=u128::from(x));
        // }
        self.length.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.command.to_be_bytes().map(|x|sumdata+=u128::from(x));
        self.data.to_be_bytes().map(|x|sumdata+=u128::from(x));

        let hex_str = format!("{:#x}",sumdata);
        let check_sum =hex::decode(&hex_str[hex_str.len()-2..]);
        if let Ok(data)=check_sum{
            if self.checksum!=data[0]{
                return Err("Fail checksum Err".to_string());
            }
            let num = data[0].to_string();
            return Ok(());
        }
        else{
            let hex_str = hex_str.trim_start_matches("0x");
            let checksum=u8::from_str_radix(hex_str,16).unwrap();
            if self.checksum!=checksum{
                return Err("Fail checksum Err".to_string());
            }
            let num = checksum.to_string();
            return Ok(());
        }
    }
    fn to_vec(&self)->ResultC<Vec<u8>,String>{
        let mut list = vec![];
        list.push(self.start);
        list.push(self.length);
        list.push(self.command);
        list.push(self.data);
        list.push(self.checksum);
        list.push(self.end);
        Ok(list)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::new()
    ).expect("setting default subscriber failed");
    let mut port = tokio_serial::new(SERIAL_DEVICE, 115200).open_native_async().unwrap();
    #[cfg(unix)]
    port.set_stop_bits(StopBits::One).unwrap();
    let (mut writer, mut reader) = LineCodec.framed(port).split();
    let serial_handle = tokio::task::Builder::new()
        .name("Serial Reciver")
        .spawn(async move{
            // let mut port = tokio_serial::new(SERIAL_DEVICE, 115200).open_native_async().unwrap();
            #[cfg(unix)]
            // port.set_stop_bits(StopBits::One).unwrap();
            // let mut reader =LineCodec.framed(port);
            while let Some(line_result) = reader.next().await {
                if let Ok(packet)=line_result{
                    match packet.command{
                        0x01=>info!("READ [Request]: {:?}",packet),
                        0x02=>info!("READ [RESPONSE]: {:?}",packet),
                        0x03=>info!("READ [REPORT]: {:?}",packet),
                        _=>{}
                    }
                    
                }
                // let mut respon = Responese::default();
                // if let Ok(line)=line_result{
                //     if let Ok(_)=respon.parser(&line){
                //         match respon.is_checksum() {
                //             Ok(_)=>println!("{:?}", respon),
                //             Err(_)=>println!("CheckSum Error"),
                //         }
                //         // println!("{:?}", respon);    
                //     }
                // }
            }
        });

    loop{
        // println!("Main Loop");
        thread::sleep(Duration::from_millis(1000));
        // let test = Responese::default();
        // if let Ok(list) = test.to_vec(){
        //     if let Ok(send_data)=writer.send(list).await{
        //         println!("SEND : {:?}",test);
        //     }
        // }
    }

    // let mut port = tokio_serial::new(SERIAL_DEVICE, 115200).open_native_async().unwrap();
    // #[cfg(unix)]
    // port.set_stop_bits(StopBits::One).unwrap();
    // let (writer, mut reader) = LineCodec.framed(port).split();
    // while let Some(line_result) = reader.next().await {
    //     let line = line_result.expect("Failed to read line");
    //     println!("{}", line);
    // }
    
    Ok(())
}
