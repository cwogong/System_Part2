use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{sleep, Duration};

// 테스트할 봇의 수 (컴퓨터 성능에 맞춰 500으로 설정 가능)
const CLIENT_COUNT: usize = 10; 

#[tokio::main]
async fn main() {
    println!("🚀 {}명의 봇이 접속을 시작합니다. 모든 수신 메시지를 실시간으로 출력합니다.", CLIENT_COUNT);
    let mut handles = vec![];

    for id in 0..CLIENT_COUNT {
        let handle = tokio::spawn(async move {
            // 1. 서버 접속
            let stream = match TcpStream::connect("127.0.0.1:8080").await {
                Ok(s) => s,
                Err(_) => return, // 서버 연결 실패 시 종료
            };

            // 소켓을 읽기 전용과 쓰기 전용으로 분리
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);

            // 2. 접속 즉시 인삿말 전송 ("bot 1 hi" 형식)
            let greeting = format!("bot {} hi\n", id);
            let _ = writer.write_all(greeting.as_bytes()).await;

            // 3. 실시간 수신 및 출력 루프
            // 이제 다른 메시지를 기다리지 않고, 서버에서 오는 모든 것을 즉시 화면에 뿌립니다.
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("❌ [Bot {:03}] 서버 연결 종료", id);
                        break;
                    }
                    Ok(_) => {
                        // 서버로부터 브로드캐스트된 모든 메시지를 출력[cite: 3, 4]
                        println!("📩 [Bot {:03} 수신]: {}", id, line.trim());
                    }
                    Err(_) => break,
                }
            }
        });
        
        // 500개가 한꺼번에 접속하면 OS의 파일 핸들 제한에 걸릴 수 있으므로 미세한 간격을 둡니다.
        if id % 10 == 0 {
            sleep(Duration::from_millis(50)).await;
        }
        handles.push(handle);
    }

    // 모든 봇이 종료될 때까지 대기
    for handle in handles {
        let _ = handle.await;
    }
}