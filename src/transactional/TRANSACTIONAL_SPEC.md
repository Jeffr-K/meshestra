# Transactional Implementation Specification

## 1. 개요 (Overview)
`Transactional` 모듈은 비즈니스 로직의 원자성(Atomicity)을 보장하기 위한 핵심 기능입니다. Spring Framework의 선언적 트랜잭션 관리와 유사한 경험을 Rust 환경에서 제공하는 것을 목표로 합니다.
단순히 트랜잭션을 열고 닫는 것을 넘어, **격리 수준(Isolation Level)**, **전파 속성(Propagation)**, 그리고 **컨텍스트 관리(Context Management)**를 포함한 "똑똑한" 트랜잭션 시스템을 설계합니다.

## 2. 핵심 기능 설계

### 2.1 트랜잭션 옵션 (Transaction Options)
매크로를 통해 세밀한 트랜잭션 제어가 가능해야 합니다.

```rust
#[transactional(
    isolation = "SERIALIZABLE",
    propagation = "REQUIRES_NEW",
    read_only = true,
    timeout = 30
)]
pub async fn critical_operation(&self) -> Result<()> { ... }
```

1.  **Isolation Level (격리 수준)**
    *   `READ_UNCOMMITTED`
    *   `READ_COMMITTED` (Default)
    *   `REPEATABLE_READ`
    *   `SERIALIZABLE`
2.  **Propagation (전파 속성)**
    *   `REQUIRED` (Default): 이미 트랜잭션이 있으면 참여하고, 없으면 새로 만듭니다.
    *   `REQUIRES_NEW`: 항상 새로운 트랜잭션을 만듭니다. 기존 트랜잭션은 일시 중지됩니다.
    *   `SUPPORTS`: 트랜잭션이 있으면 참여하고, 없으면 없이 실행합니다.
    *   `MANDATORY`: 트랜잭션이 반드시 있어야 합니다. 없으면 에러 발생.
    *   `NESTED`: Savepoint를 사용하여 중첩 트랜잭션을 수행합니다.
3.  **Read Only**: 읽기 전용 힌트를 제공하여 DB 성능을 최적화합니다.
4.  **Timeout**: 지정된 시간 내에 트랜잭션이 완료되지 않으면 강제 롤백합니다.

### 2.2 스마트 컨텍스트 관리 (Smart Context Management)
매직 변수(`let tx = ...`) 방식은 명시적이지만, 호출 깊이가 깊어지면 `tx` 객체를 계속 전달해야 하는 불편함이 있습니다(`Drilling`).
이를 해결하기 위해 `tokio::task::LocalKey` (Task Local Storage)를 활용하여 **암시적 컨텍스트 전파**를 구현합니다.

*   **원리**: Rust의 `Future` 실행 컨텍스트 내에서 트랜잭션 핸들을 공유합니다.
*   **장점**: Repository 계층에서 `tx` 인자를 받지 않아도, 내부적으로 현재 컨텍스트의 트랜잭션을 찾아 쿼리를 실행할 수 있습니다.

```rust
// Repository
impl UserRepository {
    pub async fn save(&self, user: User) -> Result<()> {
        // 현재 Task Context에서 활성 트랜잭션을 찾음
        let tx = TransactionContext::current().expect("No active transaction!");
        tx.execute("INSERT INTO ...").await
    }
}

// Service
#[transactional] // 여기서 TaskLocal에 트랜잭션을 주입
pub async fn create_user(&self, user: User) -> Result<()> {
    self.user_repo.save(user).await // tx 인자 전달 불필요!
}
```

## 3. 인터페이스 설계

### 3.1 TransactionManager Trait 확장

```rust
#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self, options: TransactionOptions) -> Result<Box<dyn Transaction>>;
}

#[derive(Default)]
pub struct TransactionOptions {
    pub isolation: Option<IsolationLevel>,
    pub read_only: bool,
    pub timeout: Option<Duration>,
}
```

### 3.2 Transaction Trait 확장

```rust
#[async_trait]
pub trait Transaction: Send + Sync {
    async fn commit(&mut self) -> Result<()>;
    async fn rollback(&mut self) -> Result<()>;
    
    // Savepoint 지원 (Nested 트랜잭션용)
    async fn savepoint(&mut self, name: &str) -> Result<()>;
    async fn rollback_to_savepoint(&mut self, name: &str) -> Result<()>;
}
```

## 4. 매크로 구현 상세 (Macro Logic)

`#[transactional]` 매크로는 다음과 같은 코드를 생성해야 합니다.

```rust
pub async fn my_service_method(&self) -> Result<()> {
    // 1. 옵션 파싱
    let options = TransactionOptions { ... };

    // 2. 트랜잭션 매니저 획득 (self에서 가져오거나 DI 컨테이너에서 조회)
    let tm = &self.transaction_manager;

    // 3. Propagation 로직 처리
    if tm.has_active_transaction() && propagation == REQUIRES_NEW {
        // 기존 트랜잭션 suspend 로직...
    }

    // 4. 트랜잭션 시작
    let mut tx = tm.begin(options).await?;
    
    // 5. Task Local에 주입 및 실행 (Scope 설정)
    let result = TRANSACTION_CONTEXT.scope(tx.clone(), async {
        // 원본 함수 바디 실행
        let res = (async { ... }).await;
        res
    }).await;

    // 6. 결과에 따른 커밋/롤백
    if result.is_ok() {
        tx.commit().await?;
    } else {
        tx.rollback().await?;
    }

    result
}
```

## 5. 고려사항 및 제약

1.  **Send + Sync**: `tokio::task_local!`에 저장되는 객체는 스레드 안전해야 합니다. `Arc<Mutex<Box<dyn Transaction>>>` 형태가 적합합니다.
2.  **데이터베이스 호환성**: 모든 DB 드라이버(Postgres, MySQL, SQLite)가 Savepoint나 모든 격리 수준을 지원하지 않을 수 있습니다. 트레이트는 일반화하되, 구현체에서 지원 여부를 체크해야 합니다.
3.  **비동기 런타임 의존성**: `tokio` 런타임에 강하게 의존하게 됩니다.

## 6. 구현 로드맵

1.  [ ] `TransactionOptions`, `IsolationLevel` 등 열거형(Enum) 정의
2.  [ ] `TransactionManager` 트레이트 고도화 (옵션 지원)
3.  [ ] `tokio::task_local!` 기반의 `TransactionContext` 구현
4.  [ ] `#[transactional]` 매크로가 옵션 파라미터를 파싱하도록 개선
5.  [ ] 매크로에서 `TransactionContext::scope`를 사용하도록 코드 생성 로직 수정
