use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6379";

    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Servidor rodando em: {addr}");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // O segundo item ('_'), o não atribuído, contém o endereço do novo cliente
        let (socket, _) = listener.accept().await.unwrap();

        // Em Rust, é importante ressaltar o conceito de dono único da variável. Caso 'db'
        // fosse passada diretamente para a função process, ela não poderia ser utilizada
        // em outras threads. Portando, é necessário clonar a variável 'db', fazendo com que
        // todas as tasks possam acessar as próprias instâncias do banco de dados simultaneamente.
        let db = db.clone();

        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

use mini_redis::{Connection, Frame};
use tokio::net::TcpStream;

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // Trava as tasks para garantir que apenas uma task possa acessar o banco de dados de cada vez.
                // Isso é importante para evitar condições de corrida e garantir a consistência dos dados.
                let mut db = db.lock().unwrap();

                // O valor é armazenado no banco de dados como um vetor de bytes (Vec<u8>)
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();

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
}
