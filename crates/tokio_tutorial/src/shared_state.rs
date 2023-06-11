use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::thread::yield_now;

type Db = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let db1 = db.clone();
    let db2 = db.clone();
    tokio::spawn(async {
        reader(db1).await;
    });

    tokio::spawn(async {
        writer(db2).await;
    });

    // how to wait?
}

async fn reader(db: Db) {
    loop {
        let db = db.lock().unwrap();
        let _v = db.get("hello");
        yield_now();
    }
}

async fn writer(db: Db) {
    loop {
        let mut db = db.lock().unwrap();
        db.insert("k".into(), "v".into());
        yield_now();
    }
}
