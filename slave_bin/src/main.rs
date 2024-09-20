
use serde::{Deserialize, Serialize};
use serde_derive;
use serde_hex;
use defaults::Defaults;
use serde_hex::{SerHex,StrictPfx};
use tokio::{self, task};
use mini_redis::{client, Result};
use core::result::Result as ResultC;
use tokio::runtime::Runtime;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use futures::{stream::StreamExt, SinkExt};
use futures_channel::mpsc;
use tracing::{info, trace, warn, error};
use tokio_serial::{SerialPort, SerialPortBuilderExt, StopBits};
use tokio_util::codec::{Decoder, Encoder};
use slaveapi::{self, LineCodec, Packet};
use rppal::gpio::Gpio;
#[cfg(unix)]
// const SERIAL_DEVICE: &'static str = env!("SERIAL_DEVICE");
const SERIAL_DEVICE: &'static str = "/dev/ttyAMA2";
// const SERIAL_DEVICE : "/dev/ttyACM1";


const UP_BT: u8 = 2;
const DOWN_BT: u8 = 3;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::new()
    ).expect("setting default subscriber failed");
    
    let up_pin = Arc::new(Mutex::new(Gpio::new().unwrap().get(UP_BT).unwrap().into_input_pullup()));
    let down_pin = Arc::new(Mutex::new(Gpio::new().unwrap().get(DOWN_BT).unwrap().into_input_pullup()));
    // let mut pin = Gpio::new().unwrap().get(DOWN_BT)?.into_output();
    
    //=============시리얼설정=============
    let mut port = tokio_serial::new(SERIAL_DEVICE, 115200).open_native_async().unwrap();
    #[cfg(unix)]
    port.set_stop_bits(StopBits::One).unwrap();
    
    let (mut writer, mut reader) = LineCodec.framed(port).split();
    //=============시리얼 수신 스레드=============
    
    thread::spawn(move||{
        let rt  = Runtime::new().unwrap();
        rt.block_on(async {
            while let Some(line_result) = reader.next().await {
                if let Ok(packet)=line_result{
                    match packet.command{
                        0x01=>info!("READ [REQUEST]: {:?}",packet),
                        0x02=>info!("READ [RESPONSE]: {:?}",packet),
                        0x03=>info!("READ [REPORT]: {:?}",packet),
                        _=>{}
                    }
                    
                }
            }
        });
            
        });
    //=============시리얼 송신 스레드=============
    thread::spawn(move||{
        let rt  = Runtime::new().unwrap();
        rt.block_on(async {
            let mut flag1 = 0;
            let mut flag2 = 0;
            loop{
                if up_pin.lock().unwrap().is_low(){
                    if flag1 ==0{
                        flag1=1;
                        let mut req = Packet::default();
                        req.command = 0x01;
                        req.remote=0b0000_0001;
                        if let Ok(_)=req.add_checksum(){
                            if let Ok(_)=req.is_checksum(){
                                if let Ok(_)=writer.send(req).await{
                                    info!("SEND [REQUEST]: {:?}",req);
                                }
                            }
                        }
                    }
                }else {
                    flag1 =0;
                }
                if down_pin.lock().unwrap().is_low(){
                    if flag2==0{
                        flag2=1;
                        let mut req = Packet::default();
                        req.command = 0x01;
                        req.remote=0b0000_0010;
                        if let Ok(_)=req.add_checksum(){
                            if let Ok(_)=req.is_checksum(){
                                if let Ok(_)=writer.send(req).await{
                                    
                                    info!("SEND [REQUEST]: {:?}",req);
                                }
                            }
                        }
                    }
                }else{
                    flag2=0;
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
        
    });
    loop{
        
        // println!("Main Loop");
        
        // if up_pin.is_low(){
        //     info!("UP BOTTON IS CLICKED");
        // }
        // if down_pin.is_low(){
        //     info!("DOWN BOTTON IS CLICKED");
        // }
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
