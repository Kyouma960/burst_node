use std::ops::RangeBounds;

use burst_nullable_lmdb::{
    ConfiguredDatabase, DatabaseFlags, Error, LmdbDatabase, LmdbEnvironment, Transaction,
    WriteFlags, WriteTransaction,
};
use burst_types::{Account, VerificationInfo};

use crate::{
    VERIFICATION_TEST_DATABASE, LmdbIterator, LmdbRangeIterator, parallel_traversal,
};

pub struct LmdbVerificationStore {
    database: LmdbDatabase,
}

impl LmdbVerificationStore {
    pub fn new(env: &LmdbEnvironment) -> anyhow::Result<Self> {
        let database = env.create_db(Some("verification"), DatabaseFlags::empty())?;

        Ok(Self { database })
    }

    pub fn database(&self) -> LmdbDatabase {
        self.database
    }

    pub fn put(
        &self,
        txn: &mut WriteTransaction,
        account: &Account,
        info: &VerificationInfo,
    ) {
        txn.put(
            self.database,
            account.as_bytes(),
            &info.to_bytes(),
            WriteFlags::empty(),
        )
        .unwrap();
    }

    pub fn get(&self, txn: &dyn Transaction, account: &Account) -> Option<VerificationInfo> {
        match txn.get(self.database, account.as_bytes()) {
            Err(Error::NotFound) => None,
            Ok(mut bytes) => Some(
                VerificationInfo::deserialize(&mut bytes)
                    .expect("Should be valid verification data"),
            ),
            Err(e) => {
                panic!("Could not load verification info: {:?}", e);
            }
        }
    }

    pub fn exists(&self, txn: &dyn Transaction, account: &Account) -> bool {
        txn.exists(self.database, account.as_bytes())
    }

    pub fn del(&self, txn: &mut WriteTransaction, account: &Account) {
        txn.delete(self.database, account.as_bytes(), None).unwrap();
    }

    pub fn count(&self, txn: &dyn Transaction) -> u64 {
        txn.count(self.database)
    }

    pub fn clear(&self, txn: &mut WriteTransaction) {
        txn.clear_db(self.database).unwrap()
    }

    pub fn iter<'tx>(
        &self,
        tx: &'tx dyn Transaction,
    ) -> impl Iterator<Item = (Account, VerificationInfo)> + 'tx + use<'tx> {
        let cursor = tx.open_ro_cursor(self.database).unwrap();
        LmdbIterator::new(cursor, read_verification_record)
    }

    pub fn iter_range<'txn>(
        &self,
        tx: &'txn dyn Transaction,
        range: impl RangeBounds<Account> + 'static,
    ) -> impl Iterator<Item = (Account, VerificationInfo)> + 'txn {
        let cursor = tx.open_ro_cursor(self.database).unwrap();
        LmdbRangeIterator::new(
            cursor,
            range.start_bound().map(|b| b.as_bytes().to_vec()),
            range.end_bound().map(|b| b.as_bytes().to_vec()),
            read_verification_record,
        )
    }

    pub fn for_each_par(
        &self,
        env: &LmdbEnvironment,
        thread_count: usize,
        action: impl Fn(&mut dyn Iterator<Item = (Account, VerificationInfo)>) + Send + Sync,
    ) {
        parallel_traversal(thread_count, &|start, end, is_last| {
            let txn = env.begin_read();
            let start_account = Account::from(start);
            let end_account = Account::from(end);
            if is_last {
                let mut iter = self.iter_range(&txn, start_account..);
                action(&mut iter);
            } else {
                let mut iter = self.iter_range(&txn, start_account..end_account);
                action(&mut iter);
            }
            txn.commit();
        })
    }
}

pub struct ConfiguredVerificationDatabaseBuilder {
    database: ConfiguredDatabase,
}

impl ConfiguredVerificationDatabaseBuilder {
    pub fn new() -> Self {
        Self {
            database: ConfiguredDatabase::new(
                VERIFICATION_TEST_DATABASE,
                "verification",
            ),
        }
    }

    pub fn verification(mut self, account: &Account, info: &VerificationInfo) -> Self {
        self.database.insert(account.as_bytes(), info.to_bytes());
        self
    }

    pub fn build(self) -> ConfiguredDatabase {
        self.database
    }

    pub fn create(verifications: Vec<(Account, VerificationInfo)>) -> ConfiguredDatabase {
        let mut builder = Self::new();
        for (account, info) in verifications {
            builder = builder.verification(&account, &info);
        }
        builder.build()
    }
}

fn read_verification_record(key: &[u8], mut value: &[u8]) -> (Account, VerificationInfo) {
    let account = Account::from_bytes(key.try_into().unwrap());
    let info = VerificationInfo::deserialize(&mut value).unwrap();
    (account, info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use burst_nullable_lmdb::PutEvent;
    use burst_types::UnixTimestamp;
    use std::sync::Arc;

    struct Fixture {
        env: Arc<LmdbEnvironment>,
        store: LmdbVerificationStore,
    }

    impl Fixture {
        fn new() -> Self {
            Self::with_env(LmdbEnvironment::new_null())
        }

        fn with_env(env: LmdbEnvironment) -> Self {
            let env = Arc::new(env);
            Self {
                store: LmdbVerificationStore::new(&env).unwrap(),
                env,
            }
        }
    }

    #[test]
    fn empty_store() {
        let fixture = Fixture::new();
        let store = &fixture.store;
        let txn = fixture.env.begin_read();
        assert!(store.get(&txn, &Account::from(0)).is_none());
        assert_eq!(store.exists(&txn, &Account::from(0)), false);
        assert!(store.iter(&txn).next().is_none());
        assert!(store.iter_range(&txn, Account::from(0)..).next().is_none());
    }

    #[test]
    fn add_account() {
        let fixture = Fixture::new();
        let mut txn = fixture.env.begin_write();
        let put_tracker = txn.track_puts();

        let account = Account::from(1);
        let mut info = VerificationInfo::default();
        info.account_creation_timestamp = UnixTimestamp::from(1000);
        fixture.store.put(&mut txn, &account, &info);

        assert_eq!(
            put_tracker.output(),
            vec![PutEvent {
                database: LmdbDatabase::new_null(42),
                key: account.as_bytes().to_vec(),
                value: info.to_bytes(),
                flags: WriteFlags::empty(),
            }]
        )
    }

    #[test]
    fn load() {
        let account = Account::from(1);
        let mut info = VerificationInfo::default();
        info.account_creation_timestamp = UnixTimestamp::from(1000);

        let env = LmdbEnvironment::null_builder()
            .database("verification", LmdbDatabase::new_null(100))
            .entry(account.as_bytes(), &info.to_bytes())
            .build()
            .build();

        let fixture = Fixture::with_env(env);
        let txn = fixture.env.begin_read();
        let result = fixture.store.get(&txn, &account);

        assert_eq!(result, Some(info))
    }

    #[test]
    fn iterate_one_account() -> anyhow::Result<()> {
        let account = Account::from(1);
        let mut info = VerificationInfo::default();
        info.account_creation_timestamp = UnixTimestamp::from(1000);

        let env = LmdbEnvironment::null_builder()
            .database("verification", LmdbDatabase::new_null(100))
            .entry(account.as_bytes(), &info.to_bytes())
            .build()
            .build();

        let fixture = Fixture::with_env(env);
        let txn = fixture.env.begin_read();
        let mut it = fixture.store.iter(&txn);
        assert_eq!(it.next(), Some((account, info)));
        assert!(it.next().is_none());
        Ok(())
    }

    #[test]
    fn clear() {
        let fixture = Fixture::new();
        let mut txn = fixture.env.begin_write();
        let clear_tracker = txn.track_clears();

        fixture.store.clear(&mut txn);

        assert_eq!(clear_tracker.output(), vec![LmdbDatabase::new_null(42)])
    }
}

