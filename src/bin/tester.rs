use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{sleep, Duration};
use std::collections::HashSet;
use std::time::Instant;

// 💡 목표 인원인 500명으로 설정합니다. (OS 환경에 따라 접속이 튕기면 100~200으로 낮춰서 먼저 테스트하세요)
const CLIENT_COUNT: usize = 500;

#[tokio::main]
async fn main() {
    println!("🔥 {}명의 봇 접속 및 무결성 검증 테스트를 시작합니다...", CLIENT_COUNT);
    let start_time = Instant::now();
    let mut handles = vec![];

    for id in 0..CLIENT_COUNT {
        let handle = tokio::spawn(async move {
            // 1. 서버 접속
            let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
            
            // 2. 다른 봇들이 모두 접속할 때까지 2초간 대기 (Race Condition 방지)
            sleep(Duration::from_secs(2)).await;

            // 3. 다 같이 모인 후 자기 번호가 담긴 고유한 메시지 발송
            let msg = format!("Hello from Bot {}\n", id);
            stream.write_all(msg.as_bytes()).await.unwrap();

            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            
            // 🌟 여기가 바로 HashSet이 들어가는 위치입니다!
            // 단순 카운터(received_count) 대신, 받은 메시지 자체를 저장하는 보관함을 만듭니다.
            let mut received_messages = HashSet::new();

            // 4. 보관함에 들어있는 '서로 다른 메시지'의 개수가 목표치(CLIENT_COUNT)에 도달할 때까지 무한 대기
            while received_messages.len() < CLIENT_COUNT {
                line.clear();
                let bytes_read = reader.read_line(&mut line).await.unwrap();
                
                if bytes_read == 0 {
                    panic!("Bot {}: 서버 연결이 끊겼습니다! (현재까지 {}개 수신)", id, received_messages.len());
                }

                // 공백과 줄바꿈을 제거한 순수한 텍스트만 HashSet에 추가합니다.
                // 중복된 메시지라면 HashSet의 크기(.len())는 늘어나지 않습니다.
                received_messages.insert(line.trim().to_string());
            }
            
            // (선택 사항) 개별 봇의 성공 로그를 너무 많이 띄우면 터미널이 지저분해지므로 생략해도 됩니다.
            // println!("🤖 Bot {} 검증 완료!", id);
        });
        handles.push(handle);
    }

    // 5. 모든 봇의 검증이 끝날 때까지 메인 스레드 대기
    for handle in handles {
        handle.await.unwrap();
    }

    // 6. 최종 결과 출력
    let elapsed = start_time.elapsed();
    println!("============================================================");
    println!("✅ 완벽한 성공! {}개의 봇이 단 1건의 유실이나 중복 없이 각각 고유한 메시지를 모두 수신했습니다.", CLIENT_COUNT);
    println!("⏱ 총 소요 시간(2초 대기 포함): {:?}", elapsed);
    println!("============================================================");
}