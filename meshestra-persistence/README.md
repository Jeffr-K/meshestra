1. Core Abstraction Layer (Zero-Dependency)
가장 밑바닥에서 드라이버와 코어를 연결하는 인터페이스입니다.

MeshestraValue (Type System): 특정 DB 타입이 아닌 공용 타입 정의 (예: Text, Integer, Float, Blob, DateTime).

Driver Interface (Trait):

Connection: SQL 실행 및 로우 데이터 반환 인터페이스.

Pool: 커넥션 관리 및 획득 인터페이스.

Transaction: 트랜잭션 시작/커밋/롤백 제어 인터페이스.

Row & ResultSet Parser: 드라이버가 넘겨준 로우 데이터를 MeshestraValue로 매핑하는 인터페이스.

Dialect Abstraction: DB별 문법 차이(PostgreSQL의 $1 vs MySQL의 ?)를 처리하는 전략 패턴 클래스.

2. Metadata Registry (The Brain)
컴파일 타임에 정의된 엔티티 정보를 런타임에 관리하는 저장소입니다.

Entity Metadata Storage: 테이블 명, 컬럼 속성(PK, Unique, Nullable), 필드 타입 정보를 저장하는 싱글톤 레지스트리.

Relation Mapper: 1:N, N:M 관계 정보를 추적하고 관리하는 로직.

Naming Strategy: 필드명(userName)을 컬럼명(user_name)으로 변환하는 규칙 엔진.

3. Query Engine (The Heart)
직접 SQL을 쓰지 않아도 쿼리를 생성하고 실행하는 핵심 엔진입니다.

AST Builder (Query Builder): 특정 DB에 의존하지 않는 쿼리 트리(Abstract Syntax Tree) 생성기.

Query Translator: AST를 특정 Dialect에 맞는 SQL 문자열로 렌더링하는 엔진.

Hydrator (Object Mapper): DB에서 조회된 MeshestraValue 목록을 실제 Rust 구조체(Entity) 인스턴스로 조립하는 기능.

Change Set Tracker: 객체의 어떤 필드가 변경되었는지 감지하여 필요한 필드만 UPDATE 하는 유닛 오브 워크 로직.

4. Procedural Macros (The DX)
사용자가 코드를 선언적으로 짤 수 있게 해주는 마법 도구들입니다.

#[derive(Entity)]: 구조체를 분석해 Metadata Registry에 등록하는 매크로.

#[repository]: 인터페이스(Trait)만 정의하면 쿼리 엔진을 호출하는 구현체를 자동 생성하는 매크로.

#[transactional]: 메서드 실행 전/후로 트랜잭션 로직을 주입하는 인터셉터 매크로.

5. Persistence Context & Lifecycle
엔티티의 생태계를 관리하는 런타임 환경입니다.

Identity Map: 동일한 PK를 가진 엔티티는 하나의 컨텍스트 내에서 하나만 존재하도록 보장(Cache 역할).

EntityManager: 사용자가 직접 접근하여 persist(), flush(), remove() 등을 호출하는 통합 창구.

Hooks (Lifecycle Events): BeforeInsert, AfterUpdate 등 이벤트 발생 시 실행될 콜백 시스템.

6. Adapter Implementations (The Bridges)
실제 외부 라이브러리를 Meshestra 인터페이스에 맞게 래핑한 것들입니다.

sqlx-adapter: SQLx의 Pool을 MeshestraConnection으로 구현.

native-libpq-adapter: PostgreSQL Native 라이브러리를 직접 연결.

mock-adapter: DB 없이 메모리에서 동작하는 테스트용 어댑터.

7. Migration System (The Tooling)
데이터베이스 버전을 관리하는 도구입니다.

Schema Generator: 엔티티 메타데이터를 기반으로 CREATE TABLE 구문을 자동 생성하는 로직.

Migration Runner: 변경 이력을 추적하고 상/하향 마이그레이션을 실행하는 CLI 툴.

💡 정리: 어디서부터 시작해야 할까?
모든 걸 한 번에 만들 수는 없습니다. Jeffrey님에게 추천하는 개발 우선순위는 다음과 같습니다:

Step 1 (Core): MeshestraValue와 Connection 트레이트 정의.

Step 2 (Driver): sqlx를 기반으로 한 첫 번째 어댑터 구현.

Step 3 (Macro): #[derive(Entity)]를 통해 테이블 이름과 컬럼 정보만이라도 추출하기.

Step 4 (Engine): 가장 간단한 SELECT * FROM table을 생성하고 객체로 바꾸는 Hydrator 구현.

이 중에서 **가장 먼저 손대고 싶은 "첫 벽"**은 어디인가요? MeshestraValue 같은 데이터 타입 설계부터 시작해 볼까요?
