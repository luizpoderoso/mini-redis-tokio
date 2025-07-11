use mini_redis::{Result, client};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:6379";

    // Fazer a conexão com o servidor Redis
    let mut client = client::connect(addr).await?;

    // Definir um par chave valor no redis {hello: world}
    client.set("hello", "world".into()).await?;

    // Obter o valor associado à chave "hello"
    let result = client.get("hello").await?;

    println!("O valor obtido do servidor é: {:?}", result);

    Ok(())
}
