use std::env::var;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::{thread, vec};

const LOCAL: &str = "127.0.0.1:6000";
//Max buffer size of messages
const MSG_SIZE: usize = 32;

fn sleep(){
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    //Instantiating server that binds the listener to the local port.
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    
    //Sets server into non-blocking mode to constantly check for messages
    server.set_nonblocking(true).expect("failed to initialize non-blocking");

    let mut clients = vec![];
    //Instatiates the channel
    let (tx,rx) = mpsc::channel::<String>();

    loop {
        // Deconstructs result of server.accept that allows us to accept connections to the server
        if let Ok((mut socket, addr)) = server.accept(){
            println!("Client {} conneceted", addr);
            let tx = tx.clone();
            
            //Push socket to the client vector. If the socket can't be cloned -> panic!
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop{
                let mut buff = vec![0; MSG_SIZE];
                
                // Will read the message into the buffer
                match socket.read_exact(&mut buff){
                    Ok(_) => {
                        //we're going to take the message we are receiving, and collect everything but the white space
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        // Take that slice of string into an actual string
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("Failed to send msg to rx");
                    },
                    // If it's an error that would block, then just continue
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }
        // This if let statement allows us to send the msg back as bytes to the client!
        if let Ok(msg) = rx.try_recv(){
            clients = clients.into_iter().filter_map(|mut client| {
                // Convert the message into bytes
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }
        sleep();
    }

}
