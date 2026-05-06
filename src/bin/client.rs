slint::slint! {
    import { Button, LineEdit, ScrollView } from "std-widgets.slint";

    export component AppWindow inherits Window {
        title: "Rust Multi-Chat Client";
        width: 500px;
        height: 600px;

        // 양방향 통신을 위한 콜백 및 프로퍼티 정의
        callback send_message(string);
        in-out property <string> chat_log: "";

        VerticalLayout {
            padding: 20px;
            spacing: 12px;

            Text {
                text: "🚀 대규모 접속 채팅 테스트";
                font-size: 22px;
                font-weight: 700;
                horizontal-alignment: center;
            }

            // 채팅 로그 출력 영역 (UI 안전성 확보)
            Rectangle {
                background: #f8f9fa;
                vertical-stretch: 1;
                border-radius: 8px;
                padding: 12px; 

                ScrollView {
                    viewport-height: chat_text.preferred-height;

                    chat_text := Text {
                        text: root.chat_log;
                        wrap: word-wrap;
                        color: #212529;
                        font-family: "Consolas";
                        font-size: 14px;
                    }
                }
            }

            // 메시지 입력 및 전송 영역
            HorizontalLayout {
                spacing: 10px;
                height: 40px;
                
                input := LineEdit {
                    placeholder-text: "메시지를 입력하세요...";
                    font-size: 14px;
                    accepted => { 
                        if (self.text != "") {
                            root.send_message(self.text); 
                            self.text = ""; 
                        }
                    }
                }
                
                Button {
                    text: "전송";
                    primary: true;
                    clicked => { 
                        if (input.text != "") {
                            root.send_message(input.text); 
                            input.text = ""; 
                        }
                    }
                }
            }
        }
    }
}

use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use slint::ComponentHandle;

// 💡 실무 관례: 하드코딩 방지를 위한 상수 분리
const SERVER_ADDR: &str = "127.0.0.1:8080";
const MPSC_CHANNEL_CAPACITY: usize = 32;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. GUI 인스턴스 생성 및 안전한 참조(Weak) 확보
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // 2. 서버 연결 (네트워크 에러 시 명확한 패닉 메시지 출력)
    println!("⏳ 서버({})에 연결을 시도합니다...", SERVER_ADDR);
    let stream = TcpStream::connect(SERVER_ADDR).await.map_err(|e| {
        format!("서버 연결 실패. 서버가 실행 중인지 확인하세요: {}", e)
    })?;
    println!("✅ 서버에 성공적으로 접속했습니다!");
    
    // 소유권 완전 분리를 통해 읽기/쓰기 독립 보장
    let (reader, mut writer) = stream.into_split(); 

    // 3. [내부 채널] 메인 GUI 스레드 -> 네트워크 전송 태스크 간 메시지 릴레이
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(MPSC_CHANNEL_CAPACITY);

    // ---------------------------------------------------------
    // 4. [Background Task A] 서버 메시지 수신 및 GUI 렌더링
    // ---------------------------------------------------------
    let ui_copy = ui_handle.clone();
    tokio::spawn(async move {
        let mut network_reader = BufReader::new(reader);
        let mut line = String::new();

        // 스트림을 끝까지 읽는 Event Loop
        while let Ok(bytes_read) = network_reader.read_line(&mut line).await {
            if bytes_read == 0 {
                eprintln!("🔌 [System] 서버와의 연결이 정상적으로 종료되었습니다.");
                break;
            }

            let new_msg = line.clone();
            let ui_weak = ui_copy.clone();
            
            // 💡 핵심: 스레드 안전성 보장 (이벤트 루프에 UI 렌더링 위임)
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let current_log = ui.get_chat_log();
                    ui.set_chat_log(current_log + &new_msg);
                }
            });
            line.clear();
        }
    });

    // ---------------------------------------------------------
    // 5. [Background Task B] 클라이언트 메시지 서버 전송
    // ---------------------------------------------------------
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // 네트워크 쓰기 실패 시 에러 로깅 후 종료
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                eprintln!("❌ [Error] 서버로 메시지 전송 실패: {}", e);
                break;
            }
            // 💡 개선점: 버퍼링된 데이터를 즉시 푸시하여 지연 방지
            let _ = writer.flush().await; 
        }
    });

    // ---------------------------------------------------------
    // 6. [GUI Event Callback] 사용자 입력 처리
    // ---------------------------------------------------------
    let tx_for_ui = tx.clone();
    ui.on_send_message(move |msg| {
        let tx = tx_for_ui.clone();
        let formatted_msg = format!("{}\n", msg);
        
        // MPSC 채널 송신을 비동기 태스크로 분리하여 메인 스레드 프리징 완벽 차단
        tokio::spawn(async move {
            if let Err(e) = tx.send(formatted_msg).await {
                eprintln!("⚠️ [Warning] 내부 릴레이 채널 전송 실패: {}", e);
            }
        });
    });

    // 7. GUI 앱 실행 (블로킹 포인트)
    ui.run()?;

    Ok(())
}
