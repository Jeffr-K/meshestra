#[module]
pub struct DatabaseModule;

// TODO: 이걸 앱모듈에 연결하면 인식 되어야함.
// 이 설계를 실제로 돌리려면 #[module] 매크로가 ports 필드를 파싱할 수 있어야 하고, ContainerBuilder가 이 동적 모듈들을 처리할 수 있어야 합니다.
impl PersistenceModule {
    pub fn with_adapter(config: DatabaseConfig) -> DynamicModule {
        // 여기서 런타임 config를 바탕으로 Provider를 생성함
        DynamicModule::builder()
            .module::<Self>() // 상단에 정의한 매크로 모듈과 연결
            .providers([
                Provider::new(config), // 런타임 설정값 주입
                SeaOrmTransactionManager::new(),
            ])
            .exports([dyn TransactionManager])
            .build()
    }
}
