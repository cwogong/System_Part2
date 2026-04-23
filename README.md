# System_Part2
<br>

# 🚀 Rust Multi-Chat Server (도전과제 프로젝트 2)

단일 단톡방 서버에 수백 명(500명 이상)의 클라이언트가 동시에 접속하여 통신할 수 있는 고성능 멀티채팅 시스템입니다. 
Rust의 비동기 I/O 런타임(`tokio`)을 활용하여 동시성 문제를 안전하고 빠르게 처리하도록 설계되었습니다.

<br>

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

## 📂 프로젝트 구조 (Directory Structure)

```text
📦 System_Part2 (Project Root)
├── 📜 README.md           # 프로젝트 설명 및 결과 보고서 (현재 파일)
└── 📁 src/
    ├── 📁 bin/            # 독립적으로 실행 가능한 다중 바이너리(타겟) 폴더
    │   ├── 📄 client.rs   # 사용자용 채팅 클라이언트 로직 (CLI/Web)
    │   ├── 📄 server.rs   # 메인 멀티채팅 서버 로직 (Broadcasting, 동시성 제어)
    │   └── 📄 tester.rs   # 500명 동시 접속 및 메시지 유실 검증용 부하 테스트 봇
```

<br>

## 📦 설치 및 실행 방법 (Getting Started)

### 1. 저장소 클론
```bash
git clone [https://github.com/cwogong/System_Part2](https://github.com/cwogong/System_Part2)
cd rust-multi-chat
```

<br>

## 🔒 동시성 제어 및 Race Condition 해결계획 (Concurrency & Safety)

본 프로젝트는 수백 명의 클라이언트가 동시에 접속하여 메시지를 주고받는 환경이므로, **동일한 자원(클라이언트 목록, 메시지 큐 등)에 대한 동시 접근(Concurrency)**이 필연적으로 발생합니다. 이 과정에서 발생할 수 있는 Race Condition을 식별하고 안전하게 해결했습니다.

### 1. 식별된 Race Condition (문제 상황)
- **상황**: 다수의 스레드(혹은 비동기 태스크)가 '현재 접속 중인 사용자 목록(Shared State)'에 동시에 접근하여 새로운 사용자를 추가하거나, 나간 사용자를 삭제하고, 메시지를 브로드캐스팅하기 위해 목록을 순회하는 상황.
- **위험성**: 스레드 A가 목록을 읽고 있는 도중에 스레드 B가 목록의 데이터를 수정해버리면, 데이터 불일치가 발생하거나 프로그램이 패닉(Panic)에 빠질 수 있는 전형적인 Race Condition이 발생합니다.

### 2. 해결 방안 (How to solve)
Rust의 강력한 소유권(Ownership) 모델과 동시성 제어 도구를 활용하여 메모리 안전성을 보장했습니다.

* **방법 A: 공유 상태의 안전한 락(Lock) 획득 (`Arc` + `Mutex` / `RwLock`)**
  - 접속자 목록을 `Arc<RwLock<HashMap<...>>>` 구조로 래핑하여 여러 태스크에서 안전하게 소유권을 공유(`Arc`)했습니다.
  - 읽기 작업(메시지 브로드캐스트)이 쓰기 작업(접속/종료)보다 압도적으로 많다는 점을 고려해 `Mutex` 대신 `RwLock`을 사용하여 다중 읽기(Multiple Readers)를 허용, 병목을 최소화하고 Race Condition을 원천 차단했습니다.
  
* **방법 B: 공유 상태 제거 (Actor Model & `tokio::sync::mpsc`)** *[적용하신 방법에 따라 A/B 중 택일]*
  - 상태를 공유하는 대신, 중앙 상태 관리자(Actor) 태스크를 하나 두고 `mpsc`(Multi-Producer, Single-Consumer) 채널을 통해 상태 변경 '요청'을 메시지로 전달하는 방식을 채택했습니다. 
  - 상태를 직접 수정하는 주체는 오직 하나의 큐 매니저뿐이므로, Race Condition 자체가 발생할 수 없는 구조를 설계했습니다.

### 3. 안전성 증명 및 테스트 (Testing & Verification)
위에서 적용한 동시성 제어가 실제로 안전하게 동작하는지 검증하기 위해 다음과 같은 테스트를 수행했습니다.

- **컴파일 타임 검증**: Rust 컴파일러의 `Send`와 `Sync` 트레잇(Trait) 규칙을 통과함으로써 **Data Race가 100% 존재하지 않음**을 언어적 차원에서 증명했습니다.
- **동시 접속 스트레스 테스트**:
  - `tokio::spawn`을 이용해 500개의 클라이언트 연결 요청과 메시지 전송을 밀리초(ms) 단위로 동시에(Concurrently) 발생시키는 통합 테스트 코드를 작성했습니다.
  - 500개의 세션이 동시에 생성 및 해제되는 과정에서 단 한 번의 Deadlock이나 데이터 유실 없이 정상 처리됨을 확인했습니다.

