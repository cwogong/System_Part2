use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 서버 오픈
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("🚀 비동기 채팅 서버가 127.0.0.1:8080에서 실행 중입니다.");

    // 2. 채널 개설 (1024 버퍼)
    let (tx, _rx) = broadcast::channel::<String>(1024);

    loop {
        // 3. 클라이언트 접속 대기
        let (mut socket, addr) = listener.accept().await?;
        println!("✅ 유저 입장: {}", addr);

        // 4. 새 유저를 위한 송신기와 수신기 복사본 생성
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // 5. 각 유저를 독립된 비동기 태스크로 분리
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                // 💡 핵심: tokio::select! 를 통한 비동기 동시 대기
                tokio::select! {
                    // [이벤트 A] 클라이언트로부터 메시지 수신
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => {
                                // 정상적인 연결 종료 (EOF)
                                println!("🚪 유저 퇴장: {}", addr);
                                break; 
                            }
                            Ok(_) => {
                                let msg = format!("{}: {}", addr, line);
                                print!("📩 [LOG] {}", msg); // 서버 터미널 모니터링 출력
                                
                                // 방송국 전파탑(tx)에 메시지 전송
                                let _ = tx.send(msg); 
                                line.clear();
                            }
                            Err(e) => {
                                // 예기치 않은 소켓 읽기 에러 처리 (기존 unwrap() 패닉 방지)
                                eprintln!("❌ [Error] 클라이언트({}) 읽기 에러: {}", addr, e);
                                break;
                            }
                        }
                    }
                    
                    // [이벤트 B] 방송국에서 다른 유저의 메시지 수신
                    result = rx.recv() => {
                        match result {
                            Ok(msg) => {
                                // 클라이언트 소켓으로 브로드캐스트 메시지 전달
                                if writer.write_all(msg.as_bytes()).await.is_err() {
                                    eprintln!("❌ [Error] 클라이언트({}) 쓰기 실패", addr);
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped_count)) => {
                                // 💡 PR 리뷰 반영: 단순 무시(continue)가 아닌 모니터링 로그 추가
                                eprintln!("⚠️ [Warning] 수신자 지연으로 인해 {}개의 메시지 유실 후 통신 재개", skipped_count);
                                continue;
                            }
                            Err(_) => break, // 채널 닫힘 등 기타 에러
                        }
                    }
                }
            }
        });
    }
}
