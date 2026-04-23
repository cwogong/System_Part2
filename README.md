# System_Part2

# 🚀 Rust Multi-Chat Server (도전과제 프로젝트 2)

단일 단톡방 서버에 수백 명(500명 이상)의 클라이언트가 동시에 접속하여 통신할 수 있는 고성능 멀티채팅 시스템입니다. 
Rust의 비동기 I/O 런타임(`tokio`)을 활용하여 동시성 문제를 안전하고 빠르게 처리하도록 설계되었습니다.

## ✨ 주요 기능 및 요구사항 충족

- **대규모 단톡방 지원**: 단일 서버로 500명 이상의 클라이언트 동시 접속 및 메시지 브로드캐스팅 처리
- **경량화된 클라이언트**: 비즈니스 로직에 집중하기 위해 텍스트 모드(CLI) 기반의 간단한 Front-end 구현
- **메시지 무결성 검증**: 수많은 클라이언트가 각각 다른 메시지를 보낼 때, 유실 없이 제대로 송수신되는지 검증하는 로직 포함
- **성능 테스트 및 최적화**: 대규모 트래픽 상황에서의 병목 현상 파악 및 성능 개선 과정 문서화

<br>

## 🛠 기술 스택 (Tech Stack)

- **Language**: Rust (Edition 2021)
- **Async Runtime**: `tokio` (네트워크 I/O 및 태스크 스케줄링)
- **Channel**: `tokio::sync::broadcast` (효율적인 메시지 브로드캐스팅)
- **Testing**: `tokio-test`, 자체 제작한 Dummy Client 스크립트

<br>

## 📦 설치 및 실행 방법 (Getting Started)

### 1. 저장소 클론
```bash
git clone [https://github.com/your-username/rust-multi-chat.git](https://github.com/your-username/rust-multi-chat.git)
cd rust-multi-chat
