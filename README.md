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

## 📦 설치 및 실행 방법 (Getting Started)

### 1. 저장소 클론
```bash
git clone [https://github.com/your-username/rust-multi-chat.git](https://github.com/your-username/rust-multi-chat.git)
cd rust-multi-chat
```

<br>

## 🔒 동시성 제어 및 Race Condition 해결 (Concurrency & Safety)

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


🚀 Rust 멀티채팅 서버 1주일 완성 파이프라인
Day 1: 기반 공사 및 단일 통신 구현 (Setup & Echo)
목표: 프로젝트 세팅 및 클라이언트-서버 간 1:1 텍스트 송수신 확인

작업 내용:

cargo new chat-server 및 프로젝트 구조 세팅 (src/bin/server.rs, client.rs 분리)

필수 의존성 추가: tokio (features = ["full"]), futures

tokio::net::TcpListener를 이용해 기본 서버 포트 바인딩 및 연결 대기 루프 작성

클라이언트가 접속하면 터미널 입력을 서버로 보내고, 서버가 그대로 돌려주는 Echo 서버 구현

체크포인트: cargo run --bin server와 cargo run --bin client로 텍스트 송수신이 되는가?

Day 2: 멀티채팅의 핵심, 브로드캐스트 (Broadcasting)
목표: 다수의 클라이언트 접속 허용 및 메시지 전체 전파

작업 내용:

tokio::spawn을 사용하여 각 클라이언트 연결마다 독립적인 비동기 태스크(Task) 할당

tokio::sync::broadcast 채널 생성

클라이언트 A가 보낸 메시지를 채널 tx(Sender)로 보내고, 접속한 모든 클라이언트의 rx(Receiver)가 이를 받아 화면에 출력하도록 연결

체크포인트: 터미널 창을 3~4개 띄워 클라이언트를 접속시키고, 한 곳에서 친 채팅이 모두에게 보이는가?

Day 3: 동시성 제어 및 상태 관리 적용 (Concurrency Safety)
목표: README에 명시된 Race Condition 방지 로직 적용

작업 내용:

방법 A(RwLock) 또는 방법 B(mpsc) 중 택 1하여 구현

클라이언트 접속 시 닉네임이나 ID를 부여하고, 접속자 목록(Shared State)에 추가

클라이언트 연결 종료(Drop) 시 목록에서 안전하게 제거

이 과정에서 발생할 수 있는 Race Condition을 Arc<RwLock<HashMap>> 등을 통해 제어

체크포인트: 누군가 접속/종료할 때 "User_X님이 입장/퇴장했습니다" 메시지가 정상적으로 모두에게 브로드캐스트 되는가?

Day 4: 대규모 테스트 봇(Dummy Client) 제작
목표: 500명 접속 및 메시지 난사 환경 구축

작업 내용:

src/bin/test_bot.rs 생성

tokio::spawn을 루프문으로 돌려 500개의 TCP 연결을 동시에 서버로 맺기

각 봇이 고유 ID와 시퀀스 번호(예: Bot#12-Msg1)가 포함된 메시지를 무작위 간격으로 서버에 전송하는 로직 작성

체크포인트: 서버가 죽지 않고 500개의 연결을 모두 받아내는가? (이때 OS의 ulimit 에러가 발생할 수 있음)

Day 5: 메시지 검증 및 성능 튜닝 (Verification & Tuning)
목표: 유실률 0% 증명 및 병목 현상 해결

작업 내용:

Test Bot이 자신이 보낸 메시지가 다시 서버로부터 브로드캐스트 되어 돌아오는지 카운트하여 유실률 검증 코드 추가

파일 디스크립터 한계(Too many open files) 발생 시 OS ulimit -n 설정 조정

네트워크 패킷 낭비를 줄이기 위해 tokio::io::BufReader, BufWriter 적용 (I/O 버퍼링)

체크포인트: 500명이 동시에 메시지를 10번씩 보냈을 때, 총 5000개의 메시지가 단 1건의 유실도 없이 처리되는가?

Day 6: 코드 리팩토링 및 예외 처리 (Refactoring)
목표: 중간고사 평가를 위한 코드 품질 향상 및 안전성 확보

작업 내용:

에러 핸들링 고도화 (클라이언트가 비정상 종료되었을 때 서버가 패닉에 빠지지 않도록 Result 처리)

코드 분리 (server.rs가 너무 길어졌다면 네트워크 로직, 상태 관리 로직 분리)

Race condition이 왜 발생할 뻔했고 어떻게 막았는지 코드 내 주석(Comment) 상세히 작성

Day 7: 최종 README 작성 및 제출 준비 (Documentation)
목표: 중간고사를 대체할 완벽한 문서화

작업 내용:

작성해주신 README 템플릿에 맞춰 실제 구현 내용 채워넣기

Day 5에서 측정한 부하 테스트 결과(소요 시간, 유실률 0% 등)를 스크린샷과 함께 문서에 첨부

전체 테스트 사이클(서버 실행 -> 봇 500개 실행 -> 검증 완료) 최종 점검