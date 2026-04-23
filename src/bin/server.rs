use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 서버 오픈
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("🚀 비동기 채팅 서버가 127.0.0.1:8080에서 실행 중입니다.");

    // 2. 단 하나의 방송국(Channel) 개설! (최대 1024개 메시지 버퍼링)
    // tx: 송신기(전파탑), _rx: 수신기 (여기선 안 쓰고 클라이언트 접속 시 복사해 줌)
    let (tx, _rx) = broadcast::channel::<String>(1024);

    loop {
        // 3. 클라이언트 접속 대기
        let (mut socket, addr) = listener.accept().await?;
        println!("✅ 유저 입장: {}", addr);

        // 4. 새 유저를 위한 송신기와 수신기 복사본 생성
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // 5. 각 유저를 독립된 비동기 태스크(Task)로 분리 (OS 스레드가 아님!)
        tokio::spawn(async move {
            // 소켓을 읽기(reader)와 쓰기(writer)로 반갈죽(?) 합니다.
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                // 💡 핵심: tokio::select! 
                // 유저가 채팅을 치는 것(수신)과, 남의 채팅을 전달받는 것(송신)을 동시에 기다립니다!
                tokio::select! {
                    // [이벤트 A] 현재 유저가 서버로 메시지를 보냈을 때
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            println!("❌ 유저 퇴장: {}", addr);
                            break; // 0바이트를 읽었다면 연결이 끊긴 것
                        }
                        
                        // 받은 메시지 앞에 유저의 주소를 예쁘게 붙입니다.
                        let msg = format!("{}: {}", addr, line);

                        // ⭐ 이 코드를 추가하세요!
                        print!("📩 [LOG] {}", msg); // 서버 터미널에 메시지 출력 (line에 \n이 포함되어 있어 print 사용)
                        
                        // 🌟 방송국 전파탑(tx)에 메시지를 쏩니다! (그러면 모든 rx에게 뿌려짐)
                        let _ = tx.send(msg); 
                        line.clear();
                    }
                    
                    // [이벤트 B] 방송국에서 다른 유저의 메시지가 날아왔을 때
                    result = rx.recv() => {
                        match result {
                            Ok(msg) => {
                                // 내 소켓(writer)을 통해 유저 화면에 출력해 줍니다.
                                if writer.write_all(msg.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                // 💡 의도된 함정 해결법!
                                // 메시지가 너무 많이 쏟아져서 큐가 꽉 차 일부를 놓쳤을 때 발생하는 에러.
                                // 프로그램이 터지지 않도록 무시(continue)하고 다음 메시지를 받습니다.
                                continue;
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });
    }
}