#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;
    use tokio::time::{Duration, timeout};
    use tokio_stream::StreamExt;

    #[derive(Clone, Debug, PartialEq)]
    struct TestItem {
        data: String,
    }

    impl Mergeable for TestItem {
        fn merge(&mut self, other: Self) -> Option<Self> {
            if self.data.len() + other.data.len() <= 10 {
                self.data.push_str(&other.data);
                None
            } else {
                Some(other)
            }
        }

        fn len(&self) -> usize {
            self.data.len()
        }

        fn slice(&self, r: std::ops::Range<usize>) -> Option<Self> {
            if r.start <= self.data.len() && r.end <= self.data.len() {
                Some(TestItem {
                    data: self.data[r].to_string(),
                })
            } else {
                None
            }
        }
    }

    #[tokio::test]
    async fn test_basic_publish_subscribe() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let mut stream = ctx.clone().subscribe(1, None);

        publisher.publish(TestItem {
            data: "hello".to_string(),
        });
        publisher.publish(TestItem {
            data: "world".to_string(),
        });

        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "helloworld");
    }

    #[tokio::test]
    async fn test_merge_limit() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let mut stream = ctx.clone().subscribe(1, None);

        publisher.publish(TestItem {
            data: "12345".to_string(),
        });
        publisher.publish(TestItem {
            data: "67890".to_string(),
        });
        publisher.publish(TestItem {
            data: "extra".to_string(),
        });

        let item1 = stream.next().await.unwrap();
        assert_eq!(item1.data, "1234567890");

        let item2 = stream.next().await.unwrap();
        assert_eq!(item2.data, "extra");
    }

    #[tokio::test]
    async fn test_subscriber_gets_snapshot() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        publisher.publish(TestItem {
            data: "hello".to_string(),
        });

        let mut stream = ctx.clone().subscribe(1, None);
        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "hello");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let mut stream1 = ctx.clone().subscribe(1, None);
        let mut stream2 = ctx.clone().subscribe(1, None);

        publisher.publish(TestItem {
            data: "test".to_string(),
        });

        let item1 = stream1.next().await.unwrap();
        let item2 = stream2.next().await.unwrap();

        assert_eq!(item1, item2);
        assert_eq!(item1.data, "test");
    }

    #[tokio::test]
    async fn test_buffer_cleared_on_publisher_drop() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        publisher.publish(TestItem {
            data: "hello".to_string(),
        });
        publisher.publish(TestItem {
            data: "world".to_string(),
        });

        drop(publisher);

        let mut stream = ctx.clone().subscribe(1, None);

        let result = timeout(Duration::from_millis(100), stream.next()).await;
        assert!(result.is_err(), "should timeout as buffer was cleared");
    }

    #[tokio::test]
    async fn test_reconnection() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let mut stream = ctx.clone().subscribe(1, None);

        publisher.publish(TestItem {
            data: "hello".to_string(),
        });

        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "hello");

        publisher.publish(TestItem {
            data: "world".to_string(),
        });

        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "world");

        drop(publisher);

        let mut publisher2 = ctx.clone().publish(1).expect("should create new publisher");
        publisher2.publish(TestItem {
            data: "hi".to_string(),
        });

        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "hi");
    }

    #[tokio::test]
    async fn test_cursor_resume() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        publisher.publish(TestItem {
            data: "first".to_string(),
        });
        publisher.publish(TestItem {
            data: "second".to_string(),
        });

        let cursor = Cursor {
            index: 0,
            offset: 5,
        };
        let mut stream = ctx.clone().subscribe(1, Some(cursor));

        publisher.publish(TestItem {
            data: "third".to_string(),
        });

        // Cursor at (0, 5) reads from "firstsecond"[5..] = "second"
        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "second");

        // Then reads "third"
        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "third");
    }

    #[tokio::test]
    async fn test_single_publisher_constraint() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let _publisher1 = ctx
            .clone()
            .publish(1)
            .expect("should create first publisher");
        let publisher2 = ctx.clone().publish(1);

        assert!(publisher2.is_none());
    }

    #[tokio::test]
    async fn test_publisher_reuse_after_drop() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let publisher1 = ctx
            .clone()
            .publish(1)
            .expect("should create first publisher");
        drop(publisher1);

        let publisher2 = ctx.clone().publish(1);
        assert!(publisher2.is_some());
    }

    #[tokio::test]
    async fn test_empty_items_skipped() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let mut stream = ctx.clone().subscribe(1, None);

        publisher.publish(TestItem {
            data: "".to_string(),
        });
        publisher.publish(TestItem {
            data: "content".to_string(),
        });

        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "content");
    }

    #[tokio::test]
    async fn test_stop_channel() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let publisher = ctx.clone().publish(1).expect("should create publisher");

        let stop_task = tokio::spawn({
            let ctx = ctx.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                ctx.stop(1).await;
            }
        });

        let halt_future = publisher.wait_halt();
        tokio::time::timeout(Duration::from_secs(1), halt_future)
            .await
            .expect("should receive halt signal");

        drop(publisher);
        stop_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_publishable_check() {
        let ctx = Arc::new(Context::<TestItem>::new());

        assert!(ctx.publishable(1));

        let _publisher = ctx.clone().publish(1).expect("should create publisher");

        assert!(!ctx.publishable(1));
    }

    #[tokio::test]
    async fn test_concurrent_publish_subscribe() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let subscribe_task = tokio::spawn({
            let ctx = ctx.clone();
            async move {
                let mut stream = ctx.subscribe(1, None);
                let mut items = Vec::new();
                for _ in 0..100 {
                    if let Ok(Some(item)) = timeout(Duration::from_millis(50), stream.next()).await
                    {
                        items.push(item);
                    } else {
                        break;
                    }
                }
                items
            }
        });

        for i in 0..100 {
            publisher.publish(TestItem {
                data: format!("{}", i),
            });
            tokio::task::yield_now().await;
        }

        drop(publisher);

        let items = subscribe_task.await.unwrap();
        assert!(items.len() > 0);
    }

    #[tokio::test]
    async fn test_slice_invalid_boundary() {
        #[derive(Clone, Debug)]
        struct InvalidSliceItem {
            data: String,
        }

        impl Mergeable for InvalidSliceItem {
            fn merge(&mut self, other: Self) -> Option<Self> {
                Some(other)
            }

            fn len(&self) -> usize {
                self.data.len()
            }

            fn slice(&self, _r: std::ops::Range<usize>) -> Option<Self> {
                None
            }
        }

        let ctx = Arc::new(Context::<InvalidSliceItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        publisher.publish(InvalidSliceItem {
            data: "test1".to_string(),
        });
        publisher.publish(InvalidSliceItem {
            data: "test2".to_string(),
        });

        let mut stream = ctx.subscribe(
            1,
            Some(Cursor {
                index: 0,
                offset: 2,
            }),
        );

        drop(publisher);

        // When slice always returns None, no data can be read
        // Stream should timeout waiting for data
        let result = timeout(Duration::from_millis(100), stream.next()).await;
        assert!(result.is_err(), "Should timeout when slice always fails");
    }

    #[tokio::test]
    async fn test_take_limits_stream() {
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let stream = ctx.clone().subscribe(1, None);

        publisher.publish(TestItem {
            data: "a".to_string(),
        });
        publisher.publish(TestItem {
            data: "b".to_string(),
        });
        publisher.publish(TestItem {
            data: "c".to_string(),
        });
        publisher.publish(TestItem {
            data: "d".to_string(),
        });
        publisher.publish(TestItem {
            data: "e".to_string(),
        });
        publisher.publish(TestItem {
            data: "f".to_string(),
        });
        publisher.publish(TestItem {
            data: "g".to_string(),
        });
        publisher.publish(TestItem {
            data: "h".to_string(),
        });
        publisher.publish(TestItem {
            data: "i".to_string(),
        });
        publisher.publish(TestItem {
            data: "j".to_string(),
        });
        publisher.publish(TestItem {
            data: "k".to_string(),
        });
        publisher.publish(TestItem {
            data: "extra".to_string(),
        });

        let items: Vec<_> = stream.take(2).collect().await;
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].data, "abcdefghij");
        assert_eq!(items[1].data, "kextra");
    }

    #[tokio::test]
    async fn test_reconnection_preserves_cursor() {
        // Regression test for malformed streaming output when switching tabs.
        // This test verifies that when a publisher drops and reconnects, the subscriber
        // maintains its cursor position and doesn't re-read already consumed data.
        let ctx = Arc::new(Context::<TestItem>::new());
        let mut publisher = ctx.clone().publish(1).expect("should create publisher");

        let mut stream = ctx.clone().subscribe(1, None);

        // Publisher sends some data
        publisher.publish(TestItem {
            data: "first".to_string(),
        });
        publisher.publish(TestItem {
            data: "second".to_string(),
        });

        // Subscriber reads the first batch
        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "firstsecond");

        // Publisher drops (simulating tab switch or connection loss)
        drop(publisher);

        // New publisher starts (simulating reconnection)
        let mut publisher2 = ctx.clone().publish(1).expect("should create new publisher");

        // New publisher sends more data
        publisher2.publish(TestItem {
            data: "third".to_string(),
        });
        publisher2.publish(TestItem {
            data: "fourth".to_string(),
        });

        // Subscriber should only receive new data, not duplicate old data
        let item = stream.next().await.unwrap();
        assert_eq!(item.data, "thirdfourth",
            "Expected 'thirdfourth' but got '{}'. This indicates the cursor position was not preserved on reconnection.",
            item.data);
    }
}
