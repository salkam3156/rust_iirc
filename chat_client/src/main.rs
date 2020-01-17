use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL_HOST: &str = "127.0.0.1:55999";
const MESSAGE_SIZE: usize = 1024;

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn main() {
    let mut client = TcpStream::connect(LOCAL_HOST).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to set non-blocking on the Client");

    let (sender, receiver) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut message_buffer = vec![0; MESSAGE_SIZE];
        match client.read_exact(&mut message_buffer) {
            Ok(_) => {
                let message = message_buffer
                    .into_iter()
                    .take_while(|&character| character != 0)
                    .collect::<Vec<_>>();
                println!("Message received {:?}", message);
            }
            Err(ref error) if error.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("The connection with Server was sewered");
                break;
            }
        }

        match receiver.try_recv() {
            Ok(message) => {
                let mut message_buffer = message.clone().into_bytes();
                message_buffer.resize(MESSAGE_SIZE, 0);

                client
                    .write_all(&mut message_buffer)
                    .expect("Writing to socket failed");

                println!("Message sent {:?}", message);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        sleep();
    });

    println!("Write a message: ");
    loop {
        let mut message_buffer = String::new();
        io::stdin()
            .read_line(&mut message_buffer)
            .expect("Reading from stdin failed");

        let message = message_buffer.trim().to_string();
        if message == ":quit" || sender.send(message).is_err() {
            break;
        }
    }
}
