use abi::Config;
use sqlx_db_tester::TestPg;
use std::{ops::Deref, path::Path};

pub struct TestConfig {
    #[allow(dead_code)]
    db: TestPg,
    pub config: Config,
}

impl Deref for TestConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl TestConfig {
    pub fn new(filename: impl AsRef<Path>) -> Self {
        let mut config = Config::load(filename).unwrap();
        let db = TestPg::new(config.db.server_url(), Path::new("../migrations"));
        config.db.name = db.dbname.clone();
        Self { db, config }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new("fixtures/config.yml")
    }
}
