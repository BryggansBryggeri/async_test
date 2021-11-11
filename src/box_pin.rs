use std::{future::Future, pin::Pin, thread::sleep, time::Duration};
use futures::future::join_all;

async fn add_one(i: usize) -> usize {
    sleep(Duration::from_secs(i as u64));
    i + 1
}
fn add_one_async(i: usize) -> Pin<Box<dyn Future<Output = usize>>> {
    Box::pin(add_one(i))
}

async fn to_none(_i: usize) -> usize {
    1
}
fn to_none_async(i: usize) -> Pin<Box<dyn Future<Output = usize>>> {
    Box::pin(to_none(i))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut futures: Vec<fn(usize) -> Pin<Box<dyn Future<Output = usize>>>> = vec![];

    futures.push(add_one_async);
    futures.push(to_none_async);

    println!("Before loop");

    for fut in &futures {
        let num = fut(1).await;
        println!("Future returned {:?}.", num);
    }

    println!("After loop")
}

