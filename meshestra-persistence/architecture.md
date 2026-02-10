# Meshestra Persistence 아키텍처 설계

이 문서는 `meshestra-persistence` 라이브러리의 전체 아키텍처 설계를 정의합니다. 목표는 특정 데이터베이스 드라이버(예: `sqlx`)에 대한 의존성 없이, 고수준의 객체-관계 매핑(ORM) 기능을 제공하는 것입니다.

## 핵심 아키텍처 컴포넌트

아키텍처는 크게 6개의 주요 레이어로 구성되며, 총 25개의 핵심 컴포넌트로 나누어집니다. 각 레이어는 특정 역할을 수행하며, 서로 유기적으로 상호작용합니다.

| 레이어 | 핵심 컴포넌트 | 상세 개발 항목 (Implementation Details) |
| --- | --- | --- |
| **1. 추상화 엔진**<br/>(Core Interface) | **Driver Port** | `Connection`, `Pool`, `Transaction`, `Statement` 등 DB 통신을 위한 핵심 동작을 추상 트레이트로 정의합니다. |
| | **Unified Type System** | DB별 상이한 데이터 타입을 통합 관리하기 위한 공용 타입 `MeshestraValue` (예: `I32`, `Text`, `Decimal`, `JsonB`)를 설계합니다. |
| | **Row Mapper** | 드라이버가 반환하는 로우(Raw) 데이터를 `MeshestraValue` 세트로 변환하는 `ResultParser`를 구현합니다. |
| | **Dialect Manager** | PostgreSQL(`$1`), MySQL(`?`) 등 DB별 SQL 문법 차이(Placeholder, Quote, Keyword)를 처리하는 전략 기반의 처리기를 구현합니다. |
| | **Adapter Implementations** | `sqlx`, `native-libpq` 등 실제 라이브러리를 Meshestra 인터페이스에 맞게 래핑한 `sqlx-adapter`, `mock-adapter` 등을 구현합니다. |
| **2. 메타데이터**<br/>(The Brain) | **Metadata Registry** | 컴파일 타임에 `#[derive(Entity)]` 매크로를 통해 수집된 엔티티, 컬럼, 인덱스 정보를 런타임에 조회할 수 있는 중앙 저장소를 구축합니다. |
| | **Relation Registry** | `OneToOne`, `OneToMany`, `ManyToMany` 같은 엔티티 간의 관계 정의 및 조인 전략 데이터를 관리합니다. |
| | **Naming Strategy** | Rust의 `camelCase` 필드명을 데이터베이스의 `snake_case` 컬럼명으로 자동 변환하는 등 명명 규칙을 관리하는 엔진을 구현합니다. |
| | **Schema Inspector** | 실제 DB 스키마를 읽어와 코드상의 메타데이터와 비교하여 차이점을 감지하는 역공학 엔진을 구현합니다. |
| **3. 쿼리 빌더**<br/>(Query Engine) | **AST Engine** | `SelectNode`, `JoinNode`, `WhereNode` 등 특정 DB에 종속되지 않는 쿼리 추상 구문 트리(AST) 구조를 설계합니다. |
| | **SQL Compiler** | 구축된 AST를 특정 `Dialect`에 맞는 SQL 문자열과 바인딩 파라미터로 변환하는 컴파일러를 구현합니다. |
| | **Criteria API** | 타입 안정성을 보장하면서 동적 쿼리를 작성할 수 있는 Fluent API 인터페이스를 제공합니다. |
| | **Query Optimizer** | 불필요한 JOIN을 제거하거나 서브쿼리를 최적화하여 쿼리 성능을 향상시키는 변환기를 구현합니다. |
| **4. 객체 매핑**<br/>(ORM Logic) | **Hydrator** | 평면적인 DB 조회 결과를 중첩된 객체 구조(Entity Graph)로 조립하여 실제 Rust 구조체 인스턴스로 변환합니다. |
| | **Identity Map** | 동일한 트랜잭션 또는 세션 내에서 같은 PK를 가진 엔티티 인스턴스의 유일성을 보장하는 1차 캐시를 구현합니다. |
| | **Change Tracker** | 엔티티의 변경 사항을 감지하여 Dirty 상태를 추적하고, `UPDATE` 시 변경된 필드만 반영하는 Unit of Work 패턴을 구현합니다. |
| | **Lazy Loader** | 연관된 엔티티를 실제 참조하는 시점에 로딩하는 지연 로딩(Proxy) 시스템을 구현합니다. |
| **5. 영속성 컨텍스트**<br/>(Runtime) | **EntityManager** | `persist`, `merge`, `remove`, `flush` 등 엔티티의 생명주기를 관리하는 메인 인터페이스를 제공합니다. |
| | **Lifecycle Hooks** | `BeforeInsert`, `AfterLoad`, `PreUpdate` 등 특정 이벤트 발생 시 실행될 콜백(Hook) 시스템을 구현합니다. |
| | **Cascade Engine** | 부모 엔티티의 영속성 조작(저장, 삭제 등)이 자식 엔티티까지 전파되는 영속성 전파 로직을 구현합니다. |
| | **Validation Engine** | DB에 저장하기 전, 정의된 제약 조건을 검증하는 인터셉터를 구현합니다. |
| **6. 개발 도구**<br/>(Dev Experience) | **Proc-Macros** | `#[derive(Entity)]`, `#[repository]`, `#[transactional]` 등 선언적 프로그래밍을 지원하는 절차적 매크로를 구현합니다. |
| | **Schema Migrator** | 엔티티 메타데이터의 변경분을 감지하여 자동으로 `ALTER TABLE` 같은 마이그레이션 SQL을 생성합니다. |
| | **CLI Toolchain** | `migration:generate`, `migration:run` 등 데이터베이스 버전 관리를 위한 명령행 도구(CLI)를 개발합니다. |
| | **Mock Driver** | DB 연결 없이 단위 테스트를 수행할 수 있도록 메모리에서 동작하는 가짜 드라이버 어댑터를 제공합니다. |

## 아키텍처 패턴 결정

Meshestra Persistence가 어떤 설계 철학을 따르는지 명확히 정의합니다.

### 1. 주력 패턴: `Repository`와 `Data Mapper` 채택
- **결정**: Meshestra는 **리포지토리(Repository) 패턴**을 핵심 아키텍처로 채택합니다.
- **이유**:
    - **관심사 분리(Separation of Concerns)**: 도메인 로직(Entity)과 영속성 로직(DB 접근 코드)을 명확하게 분리할 수 있습니다. 엔티티는 순수한 데이터 구조(Plain Old Rust Object)로 유지되며, 어떻게 저장되는지 알 필요가 없습니다.
    - **테스트 용이성**: 리포지토리를 인터페이스(Trait)로 정의하므로, 비즈니스 로직을 테스트할 때 실제 DB 대신 Mock Repository를 쉽게 주입할 수 있습니다.
- **구현 방향**:
    - 사용자는 `trait UserRepository`와 같이 인터페이스만 정의합니다.
    - `#[repository]` 매크로는 이 인터페이스에 대한 실제 구현체(Data Mapper)를 자동으로 생성합니다. 이 구현체는 내부적으로 쿼리 빌더, Hydrator, EntityManager를 사용하여 DB와 통신합니다.

### 2. `Active Record` 패턴은 왜 배제하는가?
- **결정**: `Active Record` 패턴(예: `user.save()`)은 지원하지 않습니다.
- **이유**:
    - **단일 책임 원칙 위배**: 엔티티 모델이 비즈니스 로직과 영속성 책임을 모두 갖게 되어 결합도가 높아집니다.
    - **유연성 및 확장성 한계**: 도메인 모델이 특정 데이터베이스 스키마에 강하게 얽매이게 되어, 복잡한 비즈니스 로직을 표현하거나 여러 데이터 소스를 다루기 어려워집니다.
- Meshestra는 단순한 CRUD를 넘어 복잡하고 확장 가능한 애플리케이션을 지향하므로, `Repository` 패턴이 더 적합합니다.

### 3. `Unit of Work` 패턴의 역할 명확화
- `Unit of Work` 패턴은 Meshestra의 트랜잭션 관리와 상태 변경 추적의 핵심입니다.
- **동작 방식**:
    1. `#[transactional]` 스코프가 시작되면, 새로운 `Unit of Work`가 생성됩니다.
    2. 해당 스코프 내에서 조회된 모든 엔티티는 `Identity Map`에 의해 관리되며, 변경 사항은 `Change Tracker`에 의해 "dirty"로 표시됩니다.
    3. 스코프가 성공적으로 종료되면, `Unit of Work`는 "dirty" 엔티티 목록을 검토하여 필요한 `INSERT`, `UPDATE`, `DELETE` SQL을 단일 트랜잭션으로 실행(`flush`)합니다.
- 즉, `EntityManager`가 `Unit of Work`의 구현체 역할을 합니다.

### 4. `이벤트 소싱(Event Sourcing)` 패턴의 범위
- **결정**: **이벤트 소싱은 `meshestra-persistence`의 직접적인 목표가 아닙니다.**
- **이유**:
    - 이벤트 소싱은 현재 상태를 저장하는 전통적인 ORM과 근본적으로 다른 패러다임입니다. 상태를 일련의 이벤트 로그로 저장하고, 현재 상태는 이벤트를 재실행하여 계산합니다.
    - `meshestra-persistence`는 관계형 데이터베이스와의 객체-상태 매핑에 집중합니다.
- **여지**: 다만, `Ports and Adapters` 구조 덕분에 이론적으로는 `EventStore`를 위한 `Adapter`를 만들어 `meshestra-messaging` 같은 다른 모듈과 연동할 수는 있습니다. 하지만 이는 Core의 주력 기능이 아닙니다.

## 개발 로드맵 (MVP)

이 방대한 시스템을 한 번에 구축하는 것은 불가능하므로, 다음과 같은 우선순위에 따라 최소 기능 제품(MVP)을 개발합니다.

1.  **Step 1 (Core Abstraction):**
    *   `MeshestraValue` 열거형 정의 (DB 데이터 타입 추상화)
    *   `Connection` 트레이트 설계 (SQL 실행 및 결과 반환 통로)

2.  **Step 2 (First Driver):**
    *   `sqlx` 라이브러리를 기반으로 한 첫 번째 `Adapter` 구현

3.  **Step 3 (Basic Metadata):**
    *   `#[derive(Entity)]` 매크로를 구현하여 최소한 테이블 이름과 컬럼 정보라도 런타임에 인식

4.  **Step 4 (Basic Query Execution):**
    *   작성된 SQL을 직접 실행하고, 그 결과를 엔티티 객체로 변환하는 간단한 `Hydrator` 및 `RawSqlExecutor` 구현

이 MVP 단계를 통해 라이브러리의 가장 핵심적인 코어를 먼저 구축하고, 점진적으로 기능을 확장해 나갑니다.

## 의존성 분리: Ports and Adapters (Hexagonal Architecture)

`meshestra-persistence`의 핵심 설계 원칙은 **"Ports and Adapters"** 패턴을 적용하여 의존성을 제어하는 것입니다. 이를 통해 핵심 로직과 외부 구현 기술을 명확하게 분리합니다.

### 1. Meshestra Core (Port Layer)
- **설명**: `meshestra-persistence-core` 패키지에 해당하며, 외부 라이브러리 의존성이 전혀 없는 순수한 Rust 코드로만 구성됩니다.
- **구성 요소**:
    - **Ports**: `Connection`, `TransactionManager`, `Repository` 등 핵심 기능을 정의하는 인터페이스(Trait).
    - **Contracts**: `MeshestraValue`, `Row`, `Metadata` 등 시스템 전반에서 사용될 데이터 규격.
- **의존성 방향**: 모든 외부 모듈(Adapters)이 이 Core를 바라보게 됩니다 (Adapters -> Core).

### 2. Persistence Adapters (Implementation Layer)
- **설명**: 실제 DB 기술(`SQLx`, 네이티브 드라이버 등)을 사용하여 Core의 인터페이스를 구현하는 부분입니다.
- **구성 요소**:
    - **Adapter**: `SqlxPostgresAdapter`, `NativeLibpqAdapter`, `MockAdapter` 등.
- **역할**: Core에 정의된 `Connection` 트레이트의 실제 동작을 구현합니다.
- **의존성 방향**: Adapter가 Core의 인터페이스를 구현합니다 (Adapter -> Core).

### 데이터와 제어의 흐름

| 단계 | 주체 | 동작 내용 |
| :--- | :--- | :--- |
| 1. 요청 | Meshestra Core | "데이터를 저장해야 하니 `Connection.execute()`를 호출해줘" (Port 사용) |
| 2. 변환 | Adapter (SQLx) | Meshestra용 SQL과 데이터를 `sqlx::query()` 형식으로 변환 |
| 3. 실행 | External DB | 실제 데이터베이스에 쿼리 전송 및 결과 반환 |
| 4. 결과 매핑 | Adapter (SQLx) | `sqlx::Row`를 다시 `MeshestraValue`로 변환하여 Core에 전달 |

### 이점
- **플러그형 드라이버**: `cargo add meshestra-persistence-sqlx` 명령만으로 SQLx를 사용할 수 있고, 나중에 더 빠른 드라이버가 나오면 어댑터만 교체하여 사용할 수 있습니다.
- **완벽한 테스트**: 실제 DB 없이 `MockAdapter`를 주입하여 트랜잭션 관리, 엔티티 매핑 등 코어 로직을 100% 테스트할 수 있습니다.
- **컴파일 타임 최적화**: 사용자가 필요한 어댑터만 선택적으로 컴파일하므로, 불필요한 코드가 최종 바이너리에 포함되지 않습니다.

### 다음 설계: Core 인터페이스 구체화

이 구조를 실현하기 위해 `meshestra-persistence-core`에 정의될 `Connection` 트레이트의 명세를 구체화해야 합니다.

```rust
#[async_trait]
pub trait Connection: Send + Sync {
    // SQL 실행 (INSERT, UPDATE, DELETE)
    async fn execute(&self, sql: &str, params: Vec<MeshestraValue>) -> Result<u64, PersistenceError>;
    
    // 쿼리 실행 (SELECT)
    async fn fetch_all(&self, sql: &str, params: Vec<MeshestraValue>) -> Result<Vec<Row>, PersistenceError>;
    
    // 트랜잭션 시작
    async fn begin(&self) -> Result<Box<dyn Transaction>, PersistenceError>;
}
```

## `sqlx` 의존성 배제 및 Native Driver 직접 사용 전략

`sqlx`와 같은 중간 프레임워크를 걷어내고, `mysql_async`, `tokio-postgres`와 같은 검증된 로우 레벨 네이티브 드라이버를 직접 사용하는 전략을 채택합니다. 이 경우, `Meshestra Persistence` Core의 역할과 책임은 다음과 같이 확장됩니다.

### 1. 통합 커넥션 풀링 (Unified Connection Pooling)
로우 레벨 드라이버는 각기 다른 커넥션 풀링 메커니즘을 가집니다. Meshestra는 이를 통합 관리하는 공통 풀링 로직을 제공해야 합니다.
- **추상화**: `mysql_async::Pool`과 `tokio-postgres`의 커넥션 관리 방식을 `MeshestraPool`이라는 단일 인터페이스로 추상화합니다.
- **장점**: 사용자는 어떤 DB 드라이버를 사용하든 `max_connections`, `idle_timeout` 같은 설정을 동일한 구성으로 관리할 수 있습니다.

### 2. 선언적 쿼리 바인딩 (Declarative Parameter Mapping)
로우 레벨 드라이버들은 쿼리 파라미터를 처리하는 방식이 제각각입니다.
- **Type Mapper**: `#[repository]` 매크로가 생성한 인자들을 각 드라이버가 요구하는 네이티브 타입(예: `mysql_async::Value`)으로 자동 변환하는 `Type Mapper`를 제공합니다.
- **SQL Dialect**: `Dialect` 컴파일러가 `$1`, `$2`(Postgres) 형태의 파라미터를 `?`, `?`(MySQL)로 자동 치환하는 역할을 수행합니다.

### 3. 유닛 오브 워크 (Unit of Work & Change Tracking)
로우 레벨 드라이버는 객체의 변경 상태를 추적하지 않습니다. 이는 ORM의 핵심 기능이므로 Meshestra Core가 직접 구현해야 합니다.
- **Dirty Checking**: 엔티티가 로드된 시점의 상태를 스냅샷으로 저장하고, `flush()`가 호출될 때 변경된 필드만 감지하여 동적으로 `UPDATE` 쿼리를 생성합니다.

### 레이어별 상세 역할
| 기능 | 로우 레벨 드라이버의 역할 | Meshestra Persistence의 역할 |
| :--- | :--- | :--- |
| **네트워크** | DB 프로토콜 패킷 송수신 | (관여 안 함, 드라이버에 위임) |
| **인증** | DB 로그인 및 세션 유지 | 드라이버 설정(Config) 전달 및 세션 관리 |
| **트랜잭션** | `BEGIN`, `COMMIT` 명령 전송 | `@transactional` 어노테이션 기반의 선언적 트랜잭션 제어 로직 제공 |
| **객체 변환** | Raw 데이터 행(`Row`) 제공 | 로우 데이터를 엔티티 구조체로 조립 (**Hydration**) |
| **쿼리 생성** | (유저가 직접 SQL 작성) | AST 빌더를 통한 타입-세이프 동적 SQL 자동 생성 |

## 구체적 핵심 설계 (Detailed Core Designs)
아키텍처의 핵심 컴포넌트들을 실제로 어떻게 구현할지에 대한 구체적인 설계입니다.

### 1. 통합 타입 시스템: `MeshestraValue`와 타입 매핑 전략
- **과제**: Core의 `MeshestraValue`를 각 `Adapter`가 사용하는 네이티브 타입(예: `mysql_async::Value`, `postgres::types::ToSql`)으로 어떻게 변환할 것인가?
- **설계**:
    - **`MeshestraValue` Enum 정의**: `Text(String)`, `Integer(i64)`, `Float(f64)`, `Boolean(bool)`, `Bytes(Vec<u8>)`, `Json(serde_json::Value)` 등 DB와 주고받을 공통 타입을 구체적으로 정의합니다.
    - **양방향 매핑 Trait 설계**: 각 어댑터는 두 개의 핵심 트레이트를 구현합니다.
        1. `TryFrom<DriverRow>`: DB 드라이버의 로우 레벨 결과(`mysql_async::Row`)를 `Vec<MeshestraValue>`로 변환합니다.
        2. `TryInto<DriverValue>`: `MeshestraValue`를 드라이버가 이해하는 파라미터 타입(`mysql_async::Value`)으로 변환합니다.
    - 이를 통해 Core와 Adapter 사이의 데이터 '번역' 계층이 완성됩니다.

### 2. 메타데이터 파이프라인: Proc-Macro와 런타임 레지스트리 연결
- **과제**: 컴파일 타임에 실행되는 `#[derive(Entity)]` 매크로가 어떻게 런타임에 존재하는 단일 저장소(Registry)에 엔티티 정보를 "등록"할 수 있는가?
- **설계**:
    - **`inventory` Crate 도입**: 컴파일러의 링커 기능을 활용하여, 코드 여러 곳에 흩어져 있는 정적(static) 데이터를 수집해 하나의 컬렉션으로 만들어주는 `inventory` 라이브러리를 사용합니다.
    - **동작 흐름**:
        1. `#[derive(Entity)]` 매크로는 엔티티 정보를 담은 `static EntityMetadata` 구조체를 생성하는 코드를 만듭니다.
        2. 이 구조체에 `#[inventory::submit]` 어트리뷰트를 붙여 등록합니다.
        3. 런타임 시, `inventory::iter::<EntityMetadata>()`를 호출하면 프로그램 전체에 등록된 모든 엔티티 메타데이터를 순회할 수 있습니다.
        4. PersistenceModule은 이 데이터를 수집하여 `MetadataRegistry`를 초기화합니다.

### 3. 비동기 트랜잭션 컨텍스트 전파
- **과제**: `#[transactional]` 어트리뷰트가 붙은 비동기 함수 내에서, 여러 리포지토리가 동일한 트랜잭션 커넥션을 어떻게 공유할 것인가?
- **설계**:
    - **Tokio `task_local!` 매크로 사용**: 비동기 작업(Task) 로컬 저장소를 생성하여, 현재 실행 스코프의 트랜잭션 커넥션을 저장합니다.
    - **동작 흐름**:
        1. `#[transactional]` 매크로는 메서드 실행 전에 커넥션 풀에서 커넥션을 하나 가져옵니다.
        2. 이 커넥션을 `task_local!` 변수에 할당합니다.
        3. 리포지토리의 모든 메서드는 쿼리 실행 전 `task_local!`을 먼저 확인하여, 진행 중인 트랜잭션 커넥션이 있으면 그것을 사용합니다. 없으면 풀에서 새로 가져옵니다.
        4. `#[transactional]` 매크로는 메서드 종료 시점에 결과에 따라 `COMMIT` 또는 `ROLLBACK`을 실행하고 커넥션을 반납합니다.

### 4. 통합 에러 처리 전략
- **과제**: `PersistenceError`를 어떻게 구체화하여, 특정 드라이버에 종속되지 않으면서도 상세한 오류 정보를 제공할 것인가?
- **설계**:
    - **`PersistenceError` Enum 상세화**:
        ```rust
        pub enum PersistenceError {
            // 드라이버 레벨에서 발생한 모든 알 수 없는 오류
            DriverError(Box<dyn std::error::Error + Send + Sync>),
            // 연결 실패, 풀 타임아웃 등
            ConnectionError(String),
            // Hydration 실패 (DB 데이터 -> 엔티티 변환)
            DeserializationError(String),
            // 고유 제약 조건 위반 등
            UniqueConstraintViolation { constraint_name: String },
            // 엔티티를 찾지 못함
            NotFound,
            // 기타 ORM 레벨 오류...
        }
        ```
    - **`Adapter`의 책임**: 각 `Adapter`는 `mysql_async::Error` 같은 네이티브 오류를 캐치하여 적절한 `PersistenceError` variant로 변환한 뒤 Core에 반환할 책임이 있습니다. 이를 통해 Core는 드라이버의 구체적인 오류 타입을 몰라도 일관된 방식으로 에러를 처리할 수 있습니다.

### 5. 초기화 및 설정 (Initialization and Configuration) API
- **과제**: 사용자가 `meshestra-persistence` 모듈을 자신의 애플리케이션에 통합하고, 런타임 설정을 어떻게 주입할 것인가?
- **설계**: **동적 모듈(Dynamic Module)** 패턴을 사용한 설정 API를 제공합니다. `PersistenceModule`은 `with_adapter`라는 정적 팩토리 메서드를 통해 런타임 설정을 받아 완전한 기능을 갖춘 모듈을 생성하여 반환합니다.
- **동작 흐름**:
    1. **사용자**: `PersistenceModule::with_adapter(config)`를 호출하여 데이터베이스 설정을 주입합니다.
    2. **팩토리 메서드**: 이 메서드는 `DynamicModule::builder()`를 사용하여 `ConnectionPool`과 `TransactionManager` 구현체 등 필요한 서비스(Provider)들을 동적으로 구성합니다.
    3. **DI 컨테이너**: 최종적으로 빌드된 `DynamicModule`을 메인 애플리케이션의 모듈 목록에 등록하면, DI 컨테이너가 의존성 그래프를 구성합니다.
- **예시 코드**:
    ```rust
    // 사용자는 infrastructure/database/mod.rs 와 같은 파일을 생성하여
    // 영속성 모듈을 설정합니다.
    #[module]
    pub struct DatabaseModule;

    // PersistenceModule의 구현 블록을 통해 설정용 팩토리 메서드를 제공합니다.
    impl PersistenceModule {
        pub fn with_adapter(config: DatabaseConfig) -> DynamicModule {
            // 런타임 설정을 바탕으로 필요한 Provider들을 동적으로 구성
            let db_adapter = MySqlAdapter::new(config.url, config.pool_options);
            
            DynamicModule::builder()
                .module::<DatabaseModule>() // 이 동적 모듈이 어떤 정적 모듈과 연관되는지 명시
                .providers(vec![
                    // 1. 설정된 어댑터를 Provider로 등록
                    Provider::new(db_adapter), 
                    // 2. 이 어댑터가 제공할 트랜잭션 관리자 등록
                    Provider::new(MySqlTransactionManager::new()), 
                ])
                .exports(vec![
                    // 다른 모듈에서 `Arc<dyn TransactionManager>`를 주입받을 수 있도록 공개
                    Type::of::<Arc<dyn TransactionManager>>(),
                ])
                .build()
        }
    }

    // 최종적으로 main.rs 또는 app_module.rs에서 다음과 같이 사용됩니다.
    // let db_config = load_config(); // .env 등에서 설정 로드
    // MeshestraApplication::new()
    //      .module(AppModule)
    //      .module(PersistenceModule::with_adapter(db_config)) // 동적 모듈 등록
    //      .run()
    ```

### 6. 아이덴티티 맵 (Identity Map)과 데이터 일관성
- **정의**: "동일한 데이터베이스 레코드에 대해 단 하나의 엔티티 인스턴스만 존재하도록 보장하는 메모리 내 캐시 전략"입니다. 단순한 성능 캐시를 넘어, 데이터 무결성을 위한 핵심 장치입니다.
- **소속**: `IdentityMap`은 `EntityManager` (또는 `Unit of Work` 컨텍스트) 내부에 존재하며, 생명주기를 같이 합니다. 웹 애플리케이션에서는 일반적으로 요청(Request)마다 새로운 `EntityManager`가 생성되므로, 아이덴티티 맵 또한 요청별로 격리됩니다.
- **핵심 역할 및 이점**:
    - **참조 무결성 보장**: 동일 `Unit of Work` 내에서 ID가 같은 엔티티는 항상 동일한 메모리 인스턴스를 가리킵니다. 이를 통해 데이터의 비일관성을 원천 차단합니다.
    - **변경 추적(Dirty Checking)의 근간**: `Unit of Work`가 `flush`될 때, `IdentityMap`에 저장된 초기 "스냅샷"과 현재 엔티티의 상태를 비교하여 변경된 필드(`dirty fields`)만을 감지하고 최소한의 `UPDATE` 쿼리를 생성할 수 있습니다.
    - **성능 최적화**: 동일 요청 범위 내에서 반복되는 `SELECT` 쿼리를 제거하여 DB 부하를 줄입니다.
- **Rust에서의 구현 설계**:
    ```rust
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    
    // PrimaryKey는 i32, Uuid, String 등을 모두 표현할 수 있는 타입이어야 함
    type PrimaryKey = Box<dyn Any + Send + Sync>;

    pub struct IdentityMap {
        // (엔티티 타입 ID, PK)를 키로 사용하여,
        // 여러 곳에서 소유권을 공유하고(Arc),
        // 내부 데이터를 안전하게 수정(RwLock)할 수 있는 엔티티 객체를 저장
        map: HashMap<(TypeId, PrimaryKey), Arc<RwLock<Box<dyn Any + Send + Sync>>>>,
    }
    ```
    - `Arc<RwLock<...>>` 스마트 포인터를 사용하여, Rust의 소유권 규칙 아래에서도 안전한 공유와 내부 가변성을 구현합니다.
    - `Box<dyn Any>`를 통해 다양한 타입의 엔티티를 하나의 맵에 저장하는 타입 소거(Type Erasure) 기법을 사용합니다.

## 엔티티 관계 및 마이그레이션 설계

엔티티 간의 관계를 정의하고, 이를 데이터베이스 스키마에 자동으로 반영하는 워크플로우를 설계합니다.

### 1. 엔티티 관계(Relation) 설계
엔티티 필드에 매크로 어트리뷰트를 추가하여 관계(`1:1`, `1:N`, `N:1`, `N:M`)를 선언적으로 정의합니다.

#### 관계 정의 어트리뷰트
- **`#[ManyToOne]`**: N:1 관계. 외래 키(FK)를 가지는 '소유(owning)' 측입니다.
- **`#[OneToOne]`**: 1:1 관계. `ManyToOne`처럼 FK를 가지는 쪽과 역관계를 가지는 쪽으로 나뉩니다.
- **`#[OneToMany]`**: 1:N 관계. `ManyToOne`의 반대편으로, '주인(inverse)' 측입니다.
- **`#[ManyToMany]`**: N:M 관계. 별도의 "조인 테이블(Join Table)"을 통해 관계를 맺습니다.

#### 관계 메타데이터
`#[derive(Entity)]` 매크로는 위 어트리뷰트를 파싱하여 `MetadataRegistry`에 `RelationMetadata`를 저장하며, 여기에는 관계 종류, 대상 엔티티, 소유/주인 여부, 조인 컬럼/테이블, 로딩 전략, 영속성 전파(Cascade) 옵션 등이 포함됩니다.

#### 로딩 전략: Eager vs. Lazy
- **Eager Loading (즉시 로딩)**: `#[ManyToOne(fetch = "EAGER")]`
  - 메인 엔티티 조회 시 `JOIN`을 사용해 관계된 엔티티까지 한 번의 쿼리로 모두 가져옵니다. 'N+1 문제'를 방지하지만 불필요한 데이터를 로드할 수 있습니다.
- **Lazy Loading (지연 로딩)**: `#[OneToMany(fetch = "LAZY")]` (기본값)
  - 메인 엔티티 조회 시에는 관계 필드를 비워두고, 해당 필드에 실제로 접근하는 시점에 별도의 쿼리로 데이터를 로드합니다. `Lazy<T>` 래퍼 타입을 사용합니다.

#### 영속성 전파 (Cascade Operations)
부모 엔티티의 영속성 상태 변화(`persist`, `remove`)를 자식에게 전파하는 옵션입니다.
- `cascade = ["insert", "update", "remove", "all"]`
- 예: `#[OneToMany(mapped_by = "user", cascade = ["insert"])]` -> 부모 저장 시 새로운 자식도 함께 `INSERT` 됩니다.

---

### 2. Entity-First 전략과 자동 마이그레이션
`meshestra-persistence`는 코드가 데이터베이스 스키마를 주도하는 **엔티티 우선(Entity-First)** 개발 전략을 채택합니다.

#### 개발 워크플로우
1. **코드 수정**: 개발자는 자유롭게 Rust 엔티티 코드를 수정합니다. 코드가 항상 **진실의 원천(Source of Truth)**입니다.
2. **마이그레이션 생성**: CLI에서 `migration:generate`를 실행합니다.
3. **SQL 검토**: 생성된 마이그레이션 파일의 `up()`/`down()` SQL을 검토합니다.
4. **마이그레이션 실행**: `migration:run`을 실행하여 DB 스키마를 업데이트합니다.

#### 자동 마이그레이션 시스템
- **`migration:generate`의 내부 동작**:
    1. **코드 메타데이터 로딩**: `inventory`를 통해 코드에 정의된 이상적인(Desired) 스키마 상태를 로드합니다.
    2. **DB 스키마 분석**: `Schema Inspector`가 현재 DB의 실제(Current) 스키마 상태를 분석합니다.
    3. **스키마 비교 및 'Diff' 생성**: 'Desired' 상태와 'Current' 상태를 비교하여 `CreateTable`, `AddColumn` 등 구조적 차이(`SchemaDiff`)를 생성합니다.
    4. **SQL 스크립트 생성**: `SchemaDiff`를 DB `Dialect`에 맞는 SQL 문자열로 변환합니다.
    5. **마이그레이션 파일 생성**: 타임스탬프 기반의 마이그레이션 파일(`migrations/TIMESTAMP_...`)을 생성하고, `up()`/`down()` 함수에 SQL을 채웁니다.
- **마이그레이션 실행 및 추적**:
    - `__meshestra_migrations` 테이블을 DB에 두어 실행된 마이그레이션 버전을 기록합니다.
    - `migration:run`은 아직 실행되지 않은 마이그레이션을 순차적으로 실행합니다.
    - `migration:revert`는 마지막 마이그레이션을 롤백합니다.

### Adapter 구현 예시

`Ports and Adapters` 설계가 실제 코드로 어떻게 만나는지에 대한 예시입니다.

```rust
// 1. Adapter 정의 (mysql_async를 내부적으로 사용)
pub struct MySqlAdapter {
    pool: mysql_async::Pool,
}

// 2. Port(Connection Trait) 구현
#[async_trait]
impl Connection for MySqlAdapter {
    async fn execute(&self, sql: &str, params: Vec<MeshestraValue>) -> Result<u64, PersistenceError> {
        // 여기서 Meshestra의 추상화된 명령과 데이터를
        // mysql_async의 로우 레벨 명령으로 번역하여 실행
        let mut conn = self.pool.get_conn().await?;
        // Value Mapping 로직 필요: Vec<MeshestraValue> -> mysql_async::Params
        let mapped_params = map_to_mysql_params(params); 
        conn.exec_drop(sql, mapped_params).await?;
        Ok(conn.affected_rows())
    }
    // ... fetch_all, begin 등 나머지 구현
}
```

