/// Integration tests for ScheduledJobDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::CreateJobRequest;
use vl_dao::postgres::scheduled_job::ScheduledJobDao;

fn make_job_req(name: &str) -> CreateJobRequest {
    CreateJobRequest {
        name:            name.into(),
        job_type:        "CLEANUP".into(),
        schedule_type:   "CRON".into(),
        interval_ms:     None,
        cron_expression: Some("0 0 * * *".into()),
        configuration:   serde_json::json!({"retention_days": 30}),
        enabled:         true,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let job = dao.save(tenant_id, &make_job_req("Test Job")).await.unwrap();
    assert_eq!(job.name, "Test Job");
    assert_eq!(job.tenant_id, tenant_id);

    let found = dao.find_by_id(job.id).await.unwrap().unwrap();
    assert_eq!(found.id, job.id);
    assert_eq!(found.job_type, "CLEANUP");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..5u32 {
        dao.save(tenant_id, &make_job_req(&format!("Job-{i}"))).await.unwrap();
    }

    let (jobs, total) = dao.find_by_tenant(tenant_id, 0, 3).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(jobs.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_job(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let job = dao.save(tenant_id, &make_job_req("Original")).await.unwrap();

    let updated_req = CreateJobRequest {
        name: "Updated".into(),
        ..make_job_req("Updated")
    };
    let updated = dao.update(job.id, &updated_req).await.unwrap();
    assert_eq!(updated.name, "Updated");
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_job(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let job = dao.save(tenant_id, &make_job_req("To Delete")).await.unwrap();
    dao.delete(job.id).await.unwrap();
    assert!(dao.find_by_id(job.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_job(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let job = dao.save(tenant_id, &make_job_req("To Cancel")).await.unwrap();
    dao.cancel(job.id).await.unwrap();

    let found = dao.find_by_id(job.id).await.unwrap().unwrap();
    assert!(!found.enabled);
}

#[sqlx::test(migrations = "../../migrations")]
async fn record_and_list_executions(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let job = dao.save(tenant_id, &make_job_req("Exec Job")).await.unwrap();

    dao.record_execution(job.id, "SUCCESS", None, Some(serde_json::json!({"cleaned": 5})))
        .await.unwrap();
    dao.record_execution(job.id, "FAILURE", Some("timeout"), None)
        .await.unwrap();

    let execs = dao.list_executions(job.id, 10).await.unwrap();
    assert_eq!(execs.len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_and_type(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    dao.save(tenant_id, &make_job_req("Cleanup")).await.unwrap();

    let found = dao.find_by_tenant_and_type(tenant_id, "CLEANUP").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().job_type, "CLEANUP");

    let none = dao.find_by_tenant_and_type(tenant_id, "NONEXISTENT").await.unwrap();
    assert!(none.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_next_run(pool: PgPool) {
    let dao = ScheduledJobDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let job = dao.save(tenant_id, &make_job_req("Run Job")).await.unwrap();

    let last = helpers::now_ms();
    let next = last + 86400_000;
    dao.update_next_run(job.id, last, next).await.unwrap();

    let found = dao.find_by_id(job.id).await.unwrap().unwrap();
    assert_eq!(found.last_run_at, Some(last));
    assert_eq!(found.next_run_at, next);
}
