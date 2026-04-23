slint::slint! {
    import { Button, LineEdit, ScrollView } from "std-widgets.slint";

    export component AppWindow inherits Window {
        title: "Rust Multi-Chat Front-end";
        width: 500px;
        height: 600px;

        // 부모(Rust)에게 메시지를 전달하는 통로
        callback send_message(string);
        // 서버에서 받은 메시지들이 저장될 변수
        in-out property <string> chat_log: "";

        VerticalLayout {
            padding: 20px;
            spacing: 10px;

            Text {
                text: "🚀 대규모 접속 채팅 테스트";
                font-size: 24px;
                horizontal-alignment: center;
            }

            // 채팅 내역이 표시되는 영역 (스크롤 가능)
            ScrollView {
                viewport-height: 400px;
                Rectangle {
                    background: #f0f0f0;
                    Text {
                        text: root.chat_log;
                        wrap: word-wrap;
                        color: black;
                        font-family: "Consolas";
                    }
                }
            }

            HorizontalLayout {
                spacing: 10px;
                input := LineEdit {
                    placeholder-text: "메시지를 입력하세요...";
                    accepted => { root.send_message(self.text); self.text = ""; }
                }
                Button {
                    text: "전송";
                    primary: true;
                    clicked => { root.send_message(input.text); input.text = ""; }
                }
            }
        }
    }
}

use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use slint::ComponentHandle;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. GUI 창 생성 및 핸들 확보
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // 2. 서버 연결 (서버가 8080에서 켜져 있어야 함)
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = stream.into_split(); // 소유권 분리

    // 3. [채널 생성] GUI -> 서버로 메시지를 보낼 통로
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);

    // ---------------------------------------------------------
    // 4. [백그라운드 태스크 A] 서버로부터 메시지 읽기 (수신 전담)
    // ---------------------------------------------------------
    let ui_copy = ui_handle.clone();
    tokio::spawn(async move {
        let mut network_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            // 서버에서 한 줄 읽어옴
            if let Ok(n) = network_reader.read_line(&mut line).await {
                if n == 0 { break; } // 연결 종료

                let new_msg = line.clone();
                let ui = ui_copy.clone();
                
                // GUI 스레드에 화면 업데이트 요청
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui.upgrade() {
                        let current_log = ui.get_chat_log();
                        ui.set_chat_log(current_log + &new_msg);
                    }
                });
            } else {
                break;
            }
        }
    });

    // ---------------------------------------------------------
    // 5. [백그라운드 태스크 B] 서버로 메시지 쓰기 (전송 전담)
    // ---------------------------------------------------------
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // 채널에서 받은 메시지를 실제 서버 소켓에 씀
            if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // ---------------------------------------------------------
    // 6. [GUI 콜백] 사용자가 전송 버튼을 눌렀을 때
    // ---------------------------------------------------------
    let tx_for_ui = tx.clone();
    ui.on_send_message(move |msg| {
        let tx = tx_for_ui.clone();
        let formatted_msg = format!("{}\n", msg);
        
        // 버튼 클릭 이벤트를 비동기 태스크로 넘겨 채널에 던짐
        tokio::spawn(async move {
            let _ = tx.send(formatted_msg).await;
        });
    });

    // 7. GUI 실행 (창이 닫힐 때까지 여기서 대기)
    ui.run()?;

    Ok(())
}