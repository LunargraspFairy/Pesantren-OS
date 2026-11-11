use std::{path::PathBuf, sync::Arc};
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use sipesantren::{
    build_app_router,
    config::{AppConfig, PONDOK_LOGO_URL},
    db::init_db_pool,
    notification::ConfigurableWaGateway,
    seed::seed_initial_data,
    sse::SseBroadcaster,
    AppState,
};

async fn setup_test_context() -> (axum::Router, AppState, PathBuf, String) {
    let test_id = uuid::Uuid::now_v7().to_string();
    let db_path = format!("target/test_{}.db", test_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let photo_dir = std::env::temp_dir().join(format!("sipesantren_test_photos_{}", test_id));
    tokio::fs::create_dir_all(&photo_dir)
        .await
        .expect("Gagal membuat folder foto pengujian");

    let mut config = AppConfig::from_env();
    config.database_url = db_url.clone();
    config.photo_storage_dir = photo_dir.to_string_lossy().to_string();

    let db = init_db_pool(&db_url).await.expect("Gagal inisialisasi pool SQLite");
    seed_initial_data(&db).await.expect("Gagal seeding data pengujian");

    let sse = Arc::new(SseBroadcaster::new());
    let notifier = Arc::new(ConfigurableWaGateway::new(db.clone(), &config));
    let photo_base_dir = Arc::new(photo_dir.clone());

    let state = AppState {
        db: db.clone(),
        config: Arc::new(config),
        sse,
        notifier,
        photo_base_dir,
    };

    let router = build_app_router(state.clone());
    (router, state, photo_dir, db_path)
}

async fn login_and_get_cookie(app: &axum::Router, phone: &str, pass: &str) -> String {
    let form_body = format!("phone_number={}&password={}", phone, pass);

    let req = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_body))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Login gagal untuk nomor {}",
        phone
    );

    let cookie_header = res
        .headers()
        .get(header::SET_COOKIE)
        .expect("Header Set-Cookie tidak ditemukan pada respon login")
        .to_str()
        .unwrap();

    let raw_cookie = cookie_header.split(';').next().unwrap();
    raw_cookie.to_string()
}

#[tokio::test]
async fn test_complete_12_step_sipesantren_e2e_flow() {
    let (app, state, photo_dir, db_path) = setup_test_context().await;

    // =========================================================================
    // STEP 1: Login 4 Peran (+ WALI 2 untuk audit isolasi data)
    // =========================================================================
    let wali_cookie = login_and_get_cookie(&app, "081234567801", "DemoPass123!").await;
    let wali2_cookie = login_and_get_cookie(&app, "081234567802", "DemoPass123!").await;
    let admin_cookie = login_and_get_cookie(&app, "08110000001", "DemoPass123!").await;
    let musyrif_cookie = login_and_get_cookie(&app, "08110000002", "DemoPass123!").await;
    let asatidz_cookie = login_and_get_cookie(&app, "08110000004", "DemoPass123!").await;

    assert!(wali_cookie.starts_with("sipesantren_session="));
    assert!(admin_cookie.starts_with("sipesantren_session="));
    assert!(musyrif_cookie.starts_with("sipesantren_session="));
    assert!(asatidz_cookie.starts_with("sipesantren_session="));

    // =========================================================================
    // STEP 2: Pilar 1 — Wali Request Sambang + Titipan
    // =========================================================================
    let visit_form = "santri_id=santri-01&requested_date=2026-11-20&requested_session=PAGI&visitor_count=2&has_titipan=true&titipan_description=Kurma+Ajwa+%26+kitab+kuning";

    let req = Request::builder()
        .method("POST")
        .uri("/api/visits")
        .header(header::COOKIE, &wali_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(visit_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let visit_res: Value = serde_json::from_slice(&body).unwrap();
    let visit_id = visit_res["id"].as_str().unwrap().to_string();
    let ticket_code = visit_res["ticket_code"].as_str().unwrap().to_string();

    assert_eq!(visit_res["current_status"], "REQUESTED");
    assert_eq!(visit_res["has_titipan"], 1);
    assert!(!ticket_code.is_empty());

    // =========================================================================
    // STEP 3: Pilar 1 — Admin Jadwalkan Sambang
    // =========================================================================
    let sched_form = "scheduled_start=2026-11-20T09:00:00&scheduled_end=2026-11-20T09:45:00&meeting_spot=Gazebo+Putra+01";

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/visits/{}/schedule", visit_id))
        .header(header::COOKIE, &admin_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(sched_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let sched_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sched_res["current_status"], "SCHEDULED");
    assert_eq!(sched_res["meeting_spot"], "Gazebo Putra 01");

    // =========================================================================
    // STEP 4: Pilar 1 — Wali Konfirmasi Jadwal
    // =========================================================================
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/visits/{}/wali-confirm", visit_id))
        .header(header::COOKIE, &wali_cookie)
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let conf_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(conf_res["current_status"], "CONFIRMED_BY_WALI");

    // =========================================================================
    // STEP 5: Pilar 1 — Admin Panggil Santri ke Asrama
    // =========================================================================
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/visits/{}/call-santri", visit_id))
        .header(header::COOKIE, &admin_cookie)
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let call_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(call_res["current_status"], "SANTRI_CALLED");

    // =========================================================================
    // STEP 6: Pilar 1 — Musyrif Konfirmasi Santri & Titipan
    // =========================================================================
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/visits/{}/musyrif-confirm", visit_id))
        .header(header::COOKIE, &musyrif_cookie)
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let musy_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(musy_res["current_status"], "SANTRI_CONFIRMED");
    assert_eq!(musy_res["santri_notified_by_musyrif"], 1);
    assert_eq!(musy_res["titipan_confirmed_to_wali"], 1);

    // =========================================================================
    // STEP 7: Pilar 1 — Admin Upload Foto Pertemuan Logbook (WebP)
    // =========================================================================
    let boundary = "---------------------------974767299852498929531610575";
    let mut multipart_body = Vec::new();
    multipart_body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    multipart_body.extend_from_slice(b"Content-Disposition: form-data; name=\"meeting_notes\"\r\n\r\n");
    multipart_body.extend_from_slice(b"Pertemuan lancar dan tertib di Gazebo\r\n");
    multipart_body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    multipart_body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"photo\"; filename=\"meeting.webp\"\r\n",
    );
    multipart_body.extend_from_slice(b"Content-Type: image/webp\r\n\r\n");
    // Header biner WebP valid (RIFF + WEBP + payload minimal)
    multipart_body
        .extend_from_slice(b"RIFF\x14\x00\x00\x00WEBPVP8 \x08\x00\x00\x00\x30\x01\x00\x9d\x01\x2a\x01\x00\x01\x00");
    multipart_body.extend_from_slice(b"\r\n");
    multipart_body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/visits/{}/logbook", visit_id))
        .header(header::COOKIE, &admin_cookie)
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(multipart_body))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let comp_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(comp_res["current_status"], "COMPLETED");

    // =========================================================================
    // STEP 8: Pilar 1 — Audit Privasi Foto Pertemuan (5 Peran & Publik)
    // =========================================================================
    let photo_uri = format!("/api/visits/{}/logbook/photo", visit_id);

    // 8a. WALI pemilik -> 200 OK (image/webp)
    let req = Request::builder()
        .method("GET")
        .uri(&photo_uri)
        .header(header::COOKIE, &wali_cookie)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "image/webp"
    );

    // 8b. ADMIN -> 200 OK (image/webp)
    let req = Request::builder()
        .method("GET")
        .uri(&photo_uri)
        .header(header::COOKIE, &admin_cookie)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 8c. MUSYRIF -> 403 Forbidden
    let req = Request::builder()
        .method("GET")
        .uri(&photo_uri)
        .header(header::COOKIE, &musyrif_cookie)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 8d. ASATIDZ -> 403 Forbidden
    let req = Request::builder()
        .method("GET")
        .uri(&photo_uri)
        .header(header::COOKIE, &asatidz_cookie)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 8e. WALI LAIN (santri berbeda) -> 403 Forbidden
    let req = Request::builder()
        .method("GET")
        .uri(&photo_uri)
        .header(header::COOKIE, &wali2_cookie)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 8f. Pihak ketiga tanpa cookie -> 401 Unauthorized
    let req = Request::builder()
        .method("GET")
        .uri(&photo_uri)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // =========================================================================
    // STEP 9: Pilar 2 — Asatidz Input Setoran Tahfidz & Update Juz
    // =========================================================================
    let tahfidz_form = "santri_id=santri-01&session_date=2026-11-20&log_type=ZIYADAH&juz_number=6&surah_or_kitab=QS.+Al-Maidah&ayah_or_bait_start=16&ayah_or_bait_end=35&predicate=MUMTAZ&score_numeric=95&notes=Makhraj+tajwid+sangat+baik&update_total_juz=6";

    let req = Request::builder()
        .method("POST")
        .uri("/api/tahfidz")
        .header(header::COOKIE, &asatidz_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(tahfidz_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let updated_juz: i64 = sqlx::query_scalar("SELECT total_juz_memorized FROM santri WHERE id = 'santri-01'")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(updated_juz, 6, "Total juz hafalan wajib bertambah menjadi 6");

    // =========================================================================
    // STEP 10: Pilar 3 — Asatidz Buat Sesi Ujian & Input Nilai Rapor
    // =========================================================================
    let exam_form = "term_id=term-2026-ganjil&subject_id=subj-fiqih&exam_type=UAS&class_level=1+Aliyah&exam_date=2026-11-25&title=UAS+Fiqih+Semester+Ganjil";

    let req = Request::builder()
        .method("POST")
        .uri("/api/exams")
        .header(header::COOKIE, &asatidz_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(exam_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let exam_res: Value = serde_json::from_slice(&body).unwrap();
    let exam_id = exam_res["id"].as_str().unwrap().to_string();

    let grade_form = format!(
        "exam_id={}&santri_id=santri-01&score=92.0&teacher_note=Mumtaz+pemahaman+matang",
        exam_id
    );

    let req = Request::builder()
        .method("POST")
        .uri("/api/exams/grades")
        .header(header::COOKIE, &asatidz_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(grade_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let grade_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(grade_res["letter_grade"], "A");
    assert_eq!(grade_res["predicate_label"], "Mumtaz (Istimewa)");

    // =========================================================================
    // STEP 11: Pilar 4 — Musyrif Input Absensi & Catatan Kesantrian
    // =========================================================================
    let att_form = "santri_id=santri-01&attendance_date=2026-11-20&activity_type=ASRAMA_MALAM&status=HADIR&note=Tepat+waktu+di+kamar+asrama";

    let req = Request::builder()
        .method("POST")
        .uri("/api/attendance")
        .header(header::COOKIE, &musyrif_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(att_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let note_form = "santri_id=santri-01&note_date=2026-11-20&category=PRESTASI&title=Juara+1+Hifdzil+Quran&description=Juara+1+Lomba+Hifdzil+Quran+Tingkat+Kabupaten";

    let req = Request::builder()
        .method("POST")
        .uri("/api/santri-notes")
        .header(header::COOKIE, &musyrif_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(note_form))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // =========================================================================
    // STEP 12: Audit Branding & Anti-Slop View
    // =========================================================================
    // 12a. Verifikasi Dashboard Wali memuat Logo Pondok dan komponen Anti-Slop
    let req = Request::builder()
        .method("GET")
        .uri("/wali")
        .header(header::COOKIE, &wali_cookie)
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(
        html.contains(PONDOK_LOGO_URL),
        "Halaman /wali wajib memuat URL logo resmi pondok pesantren"
    );
    assert!(
        html.contains("font-mono tabular-nums"),
        "Halaman /wali wajib mematuhi aturan tipografi tabular-nums anti-slop"
    );
    assert!(
        html.contains("Muhammad Fatih Al-Faruq"),
        "Halaman /wali wajib memuat data santri terkait"
    );

    // 12b. Verifikasi Halaman Login Publik memuat Logo Resmi Pondok
    let req = Request::builder()
        .method("GET")
        .uri("/login")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let login_html = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        login_html.contains(PONDOK_LOGO_URL),
        "Halaman /login wajib memuat URL logo resmi pondok pesantren"
    );

    // Pembersihan resource direktori pengujian
    let _ = tokio::fs::remove_dir_all(&photo_dir).await;
    let _ = tokio::fs::remove_file(&db_path).await;
    let _ = tokio::fs::remove_file(format!("{}-wal", db_path)).await;
    let _ = tokio::fs::remove_file(format!("{}-shm", db_path)).await;
}
