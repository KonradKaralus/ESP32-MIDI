use std::{iter, sync::Arc, thread::sleep, time::Duration};

use eframe::egui::mutex::Mutex;
use io_bluetooth::bt::{self, BtStream};

use crate::{command::Command, ADDRESS, NUM_PEDALS};

const CONNECTION_ATTEMPTS: usize = 3;
const READ_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(PartialEq)]
pub enum ConnectionStatus {
    Connected,
    TryConnect,
    Disconnected,
    BTNotAvailable,
}

impl ConnectionStatus {
    pub fn fmt(&self) -> String {
        let res = match self {
            ConnectionStatus::Connected => "Connected.",
            ConnectionStatus::TryConnect => "Trying to connect...",
            ConnectionStatus::Disconnected => "Not connected.",
            ConnectionStatus::BTNotAvailable => "Bluetooth not available...",
        };

        res.to_string()
    }
}

pub struct IConnection {
    pub socket: Option<BtStream>,
    pub status: ConnectionStatus,
    pub new_connect: bool,
    pub try_connect: bool,
}

pub type Connection = Arc<Mutex<IConnection>>;

impl Default for IConnection {
    fn default() -> Self {
        Self {
            socket: Option::None,
            status: ConnectionStatus::Disconnected,
            new_connect: false,
            try_connect: false,
        }
    }
}

impl IConnection {
    pub fn req_cfg(&self) -> Option<Vec<u8>> {
        let cfg_req: Vec<u8> = vec![0x00];
        let socket = match &self.socket {
            None => return None,
            Some(s) => s,
        };
        socket.send(&cfg_req).unwrap();

        let mut cfg_buffer: Vec<u8> = vec![0; NUM_PEDALS + NUM_PEDALS * size_of::<Command>()];

        socket.recv(&mut cfg_buffer).unwrap();

        Some(cfg_buffer) // TODO: does this mean, we can set to disconnected?
    }

    pub fn send_cfg(&self, mut buf: Vec<u8>) {
        buf.insert(0, 0x01);

        let socket = match &self.socket {
            None => return,
            Some(s) => s,
        };
        socket.send(&buf).unwrap();
    }

    pub fn heartbeat(&self) -> bool {
        let buf: Vec<u8> = vec![0x02];
        let socket = match &self.socket {
            None => return false,
            Some(s) => s,
        };
        socket.send(&buf).unwrap();

        let mut hb_buffer: Vec<u8> = vec![0; 4];
        let res = socket.recv(&mut hb_buffer);

        if res.is_ok() {
            hb_buffer[0] == 2
        } else {
            false
        }
    }
}

pub fn get_connection() -> Connection {
    Arc::new(Mutex::new(IConnection::default()))
}

pub fn check_connection_status(connection: Connection) {
    let mut first = true;
    'outer: loop {
        if connection.lock().status == ConnectionStatus::Connected {
            let hb = connection.lock().heartbeat();
            if hb {
                continue 'outer;
            } else {
                connection.lock().status = ConnectionStatus::Disconnected;
            }
        } else if first || connection.lock().try_connect {
            connection.lock().try_connect = false;
            first = false;
            connection.lock().status = ConnectionStatus::TryConnect;
            println!("trying conn");

            for _ in 0..CONNECTION_ATTEMPTS {
                let mut devices = match bt::discover_devices() {
                    Ok(d) => d,
                    Err(_) => {
                        connection.lock().status = ConnectionStatus::BTNotAvailable;
                        continue 'outer;
                    }
                };

                println!("trying conn in");

                devices.retain(|d| *d.to_string() == *ADDRESS);

                if devices.len() == 1 {
                    let socket =
                        BtStream::connect(iter::once(&devices[0]), bt::BtProtocol::RFCOMM).unwrap();

                    socket.set_read_timeout(Some(READ_TIMEOUT)).unwrap();

                    match socket.peer_addr() {
                        Ok(_) => {}
                        Err(err) => {
                            println!("An error occured while retrieving the peername: {:?}", err)
                        }
                    }

                    match socket.local_addr() {
                        Ok(_) => {}
                        Err(err) => {
                            println!("An error occured while retrieving the sockname: {:?}", err)
                        }
                    }
                    {
                        let mut conn = connection.lock();
                        conn.socket = Option::from(socket);
                        conn.status = ConnectionStatus::Connected;
                        conn.new_connect = true;
                    }
                    continue 'outer;
                }
            }
            connection.lock().status = ConnectionStatus::Disconnected;
        }

        sleep(Duration::from_millis(500));
    }
}
