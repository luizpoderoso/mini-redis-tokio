use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6379";

    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        // O segundo item ('_'), o não atribuído, contém o endereço do novo cliente
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    let mut db = HashMap::new();

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // O valor é armazenado no banco de dados como um vetor de bytes (Vec<u8>)
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // Eu creio que vou entender melhor como funciona o Frame::Bulk mais para frente.
                    // Só sei que ele é uma série de bytes (Vec<u8>), ou seja, faz sentido usá-lo para
                    // representar o valor armazenado no banco de dados. Segundo o tutorial, o into()
                    // converte o valor em um Frame::Bulk.
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => {
                panic!("Erro não implementado: {:?}", cmd);
            }
        };

        connection.write_frame(&response).await.unwrap()
    }

    // Com esse exemplo eu comecei a entender melhor as sintaxes de 'if let' e 'while let'.
    // Também reparei como funciona o 'read_frame' e o 'write_frame'. Obviamente já havia
    // entendido o que eles fazem, mas agora tenho uma noção mais clara de como eles são usados.
}
