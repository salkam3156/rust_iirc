use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

static LOCAL_HOST: &str = "127.0.0.1:55999";
static MESSAGE_SIZE: usize = 1024;

fn sleep() {
    thread::sleep(std::time::Duration::from_millis(100));
}

fn main() {
    let chat_server = TcpListener::bind(LOCAL_HOST).expect("Failed to bind server");
    chat_server
        .set_nonblocking(true)
        .expect("Failed to set non-blocking on server");

    let mut clients = vec![];

    let (sender, receiver) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = chat_server.accept() {
            println!("Client {:#?} connected", addr);

            let sender_clone = sender.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop {
                let mut buffer = vec![0; MESSAGE_SIZE];

                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let message = buffer
                            .into_iter()
                            .take_while(|&byte| byte != 0)
                            .collect::<Vec<_>>();
                        let message = String::from_utf8(message).expect("Invalid utf-8 chars");

                        println!("{}: {:#?}", addr, message);

                        sender_clone
                            .send(message)
                            .expect("Failed to send the message to receiver");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }

        if let Ok(message) = receiver.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut message_buffer = message.clone().into_bytes();
                    message_buffer.resize(MESSAGE_SIZE, 0);

                    client.write_all(&message_buffer).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
    }
}
