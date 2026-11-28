//! ADR-032 phase 32.1: knowledge is tenant-isolated and soft-deleted entries stay hidden.

use stheno_ai::{CreateKnowledgeEntry, KnowledgeService, UpdateKnowledgeEntry};
use stheno_db::diesel_async::{AsyncConnection, AsyncPgConnection, SimpleAsyncConnection};
use stheno_db::testing::TestDb;
use stheno_types::Error;

#[tokio::test]
async fn entries_are_isolated_deduplicated_and_soft_deleted() {
    let db = TestDb::migrated().await;
    let service = KnowledgeService;
    let mut conn = db.conn().await;
    conn.batch_execute("SET ROLE stheno_app; SET app.tenant_id = 'ten_a';")
        .await
        .expect("switch to the app role and tenant A");
    let mut conn_ref = &mut conn;

    let entry = service
        .create(
            &mut conn_ref,
            CreateKnowledgeEntry {
                title: "  Returns  ".to_string(),
                category: Some("  Policy  ".to_string()),
                content: "Unused items can be returned within 14 days. বাংলা সহায়তা".to_string(),
            },
        )
        .await
        .expect("create tenant A entry");
    assert_eq!(entry.title, "Returns");
    assert_eq!(entry.category.as_deref(), Some("Policy"));
    assert_eq!(
        entry.content,
        "Unused items can be returned within 14 days. বাংলা সহায়তা"
    );

    let english_hits = service
        .search(&mut conn_ref, "14 days", 50)
        .await
        .expect("search English policy text");
    assert_eq!(english_hits.len(), 1);
    assert_eq!(english_hits[0].entry.id, entry.id);
    let bangla_hits = service
        .search(&mut conn_ref, "বাংলা", 8)
        .await
        .expect("search Bangla policy text");
    assert_eq!(bangla_hits.len(), 1);
    assert!(service
        .search(&mut conn_ref, "   ", 8)
        .await
        .expect("empty search returns no matches")
        .is_empty());
    assert!(matches!(
        service.search(&mut conn_ref, &"x".repeat(513), 8).await,
        Err(Error::Invalid {
            code: "knowledge_query_too_long",
            ..
        })
    ));
    assert!(matches!(
        service.search(&mut conn_ref, "bad\0query", 8).await,
        Err(Error::Invalid {
            code: "knowledge_query_invalid",
            ..
        })
    ));

    for index in 0..10 {
        service
            .create(
                &mut conn_ref,
                CreateKnowledgeEntry {
                    title: format!("Shipping FAQ {index}"),
                    category: Some("Delivery".to_string()),
                    content: format!("Shipping information entry {index}."),
                },
            )
            .await
            .expect("create search corpus entry");
    }
    assert_eq!(
        service
            .search(&mut conn_ref, "shipping", 50)
            .await
            .expect("search results are capped")
            .len(),
        8
    );

    let duplicate = service
        .create(
            &mut conn_ref,
            CreateKnowledgeEntry {
                title: "returns".to_string(),
                category: Some("policy".to_string()),
                content: "Another answer".to_string(),
            },
        )
        .await;
    assert!(matches!(
        duplicate,
        Err(Error::Conflict {
            code: "duplicate_knowledge_entry",
            ..
        })
    ));

    conn_ref
        .batch_execute("SET app.tenant_id = 'ten_b';")
        .await
        .expect("switch to tenant B");
    assert!(service
        .list(&mut conn_ref, 1000, 0)
        .await
        .expect("list tenant B knowledge")
        .is_empty());
    assert!(matches!(
        service.delete(&mut conn_ref, &entry.id).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        service
            .update(
                &mut conn_ref,
                &entry.id,
                UpdateKnowledgeEntry {
                    title: "Returns".to_string(),
                    category: Some("Policy".to_string()),
                    content: "Foreign tenant update must not work".to_string(),
                },
            )
            .await,
        Err(Error::NotFound { .. })
    ));

    let mut unscoped_conn = AsyncPgConnection::establish(&db.url)
        .await
        .expect("connect without tenant scope");
    unscoped_conn
        .batch_execute("SET ROLE stheno_app;")
        .await
        .expect("switch to the app role without setting a tenant");
    let mut unscoped_ref = &mut unscoped_conn;
    assert!(service
        .list(&mut unscoped_ref, 10, 0)
        .await
        .expect("unscoped listing fails closed")
        .is_empty());
    assert!(service
        .create(
            &mut unscoped_ref,
            CreateKnowledgeEntry {
                title: "No tenant".to_string(),
                category: None,
                content: "Must not be stored without scope".to_string(),
            },
        )
        .await
        .is_err());
    unscoped_ref
        .batch_execute("RESET ROLE;")
        .await
        .expect("reset unscoped test role");

    conn_ref
        .batch_execute("SET app.tenant_id = 'ten_a';")
        .await
        .expect("switch back to tenant A");
    service
        .delete(&mut conn_ref, &entry.id)
        .await
        .expect("soft-delete tenant A entry");
    assert!(service
        .search(&mut conn_ref, "returns", 50)
        .await
        .expect("search active entries")
        .is_empty());
    assert!(matches!(
        service.delete(&mut conn_ref, &entry.id).await,
        Err(Error::NotFound { .. })
    ));
    let recreated = service
        .create(
            &mut conn_ref,
            CreateKnowledgeEntry {
                title: "Returns".to_string(),
                category: Some("Policy".to_string()),
                content: "A corrected active entry".to_string(),
            },
        )
        .await
        .expect("soft deletion releases the active unique key");
    service
        .delete(&mut conn_ref, &recreated.id)
        .await
        .expect("remove recreated fixture entry");

    conn_ref
        .batch_execute("RESET ROLE;")
        .await
        .expect("reset test role");
    db.drop().await;
}
