use std::{sync::Mutex, collections::HashMap};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use std::error::Error;

static CACHE_STRING: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

#[derive(Serialize, Deserialize)]
struct CacheItem<T> {
    #[serde(bound(serialize="T: Serialize", deserialize="T: DeserializeOwned"))]
    pub data: T,
    pub ts: i64,
    pub ttl: i64,
}

pub fn setex<T>(key: &str, value: &T, ttl: i64) -> Result<(), Box<dyn Error>> where T: Serialize + DeserializeOwned + Clone {
    let item = CacheItem {
        data: value.clone(),
        ts: chrono::Utc::now().timestamp_millis(),
        ttl,
    };

    let data = serde_json::to_string(&item)?;
    let mut cc = CACHE_STRING.lock()?;
    cc.insert(key.to_owned(), data);

    Ok(())
}

pub fn set<T>(key: &str, value: &T) -> Result<(), Box<dyn Error>> where T: Serialize + DeserializeOwned + Clone {
    setex(key, value, -1)
}

pub fn get<T>(key: &str) -> Option<T> where T: Serialize + DeserializeOwned + Clone {
    let cache = CACHE_STRING.lock().ok()?;
    let item_s = cache.get(&key.to_owned())?;

    let item: CacheItem<T> = serde_json::from_str(item_s.as_str()).ok()?;

    if item.ttl <= 0 {
        return Some(item.data);
    }

    let now = chrono::Utc::now().timestamp_millis();
    if now - item.ts <= item.ttl {
        return Some(item.data);
    }

    None
}


#[cfg(test)]
mod test {
    use serde::{Serialize, Deserialize};
    use std::time::Duration;

    use crate::local_cache;

    #[test]
    fn test() {
        local_cache::set("aa", &3);

        assert_eq!(local_cache::get::<i32>("aa"), Some(3));
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct UserTest {
        pub id: i64,
        pub username: String,
        pub password: Option<String>
    }

    #[test]
    fn test2() {
        let user = UserTest {
            id: 10,
            username: "111".to_string(),
            password: Some("asdf".to_owned())
        };

        local_cache::set("aa", &user);

        assert_eq!(local_cache::get::<UserTest>("aa"), Some(user));
    }

    #[test]
    fn test3() {
        let user = UserTest {
            id: 10,
            username: "111".to_string(),
            password: Some("asdf".to_owned())
        };

        local_cache::setex("aa", &user, 3000);

        assert_eq!(local_cache::get::<UserTest>("aa"), Some(user));

        std::thread::sleep(Duration::from_secs(4));

        assert_eq!(local_cache::get::<UserTest>("aa"), None);
    }
}

