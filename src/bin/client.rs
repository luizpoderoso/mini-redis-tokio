use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6379";

    // Os canais (nesse caso o mpsc) são usados para enviar e receber dados entre as threads,
    // já que, por si só, seria impossível passar o client para duas threads simultaneamente e
    // manter a concorrência (multi-threading). O 32 simboliza a quantidade máxima de mensagens
    // que podem ser armazenadas na fila antes de bloquear o produtor.
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let mut client = client::connect(addr).await.unwrap();

        // Receber as mensagens/comandos
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Clone do canal para enviar comandos, permitindo que mais uma thread envie comandos ao servidor
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        // Aqui é criado um canal de mão única para receber a resposta do servidor. A diferença dele para o mpsc
        // é que o mpsc permite que vários consumidores consumam a mesma mensagem, enquanto o oneshot permite apenas um consumidor.
        let (resp_tx, resp_rx) = oneshot::channel();

        // Prepara o comando GET
        let cmd = Command::Get {
            key: ("foo".to_string()),
            resp: resp_tx,
        };

        // Envia o GET
        tx.send(cmd).await.unwrap();

        // Espera pela resposta do servidor e a imprime
        let resp = resp_rx.await.unwrap();
        println!("Resposta GET: {:?}", resp);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        // Prepara o comando SET
        let cmd = Command::Set {
            key: ("foo".to_string()),
            val: ("bar".into()),
            resp: resp_tx,
        };

        tx2.send(cmd).await.unwrap();

        let resp = resp_rx.await.unwrap();
        println!("Resposta SET: {:?}", resp);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
