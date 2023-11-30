use maelstrom_echo::protocol::Message;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> maelstrom_echo::Result<()> {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let stdin = io::stdin();
        let reader = io::BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await.expect("stdin read error") {
            let msg = serde_json::from_str::<Message>(&line).expect("json parse error");
            tx.send(msg).await.unwrap();
        }
    });

    while let Some(msg) = rx.recv().await {
        let reply = msg.reply().await;
        println!(
            "{}",
            serde_json::to_string(&reply).expect("json serialize error")
        );
    }
    Ok(())
}
