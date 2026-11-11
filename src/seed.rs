use chrono::Utc;
use sqlx::SqlitePool;

use crate::auth::hash_password;

pub async fn seed_initial_data(db: &SqlitePool) -> anyhow::Result<()> {
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await?;

    if user_count > 0 {
        return Ok(());
    }

    tracing::info!("Seeding initial deterministic data for SiPesantren...");
    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let admin_pass_hash = hash_password("admin123")?;
    let musyrif_pass_hash = hash_password("musyrif123")?;
    let ustadz_pass_hash = hash_password("ustadz123")?;
    let wali_pass_hash = hash_password("wali123")?;

    // Mulai transaksi setelah pengecekan (operasi pertama di tx adalah operasi tulis INSERT)
    let mut tx = db.begin().await?;

    // 1. Asrama (2 Dormitories)
    sqlx::query(
        "INSERT INTO dormitories (id, name, gender_zone, building_location, created_at)
         VALUES
            ('dorm-putra-01', 'Asrama Putra Al-Farabi', 'PUTRA', 'Gedung Timur Lt. 1', ?1),
            ('dorm-putri-01', 'Asrama Putri Khadijah', 'PUTRI', 'Gedung Barat Lt. 2', ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 2. Pengguna (6 Akun Pengguna untuk 4 Peran)
    sqlx::query(
        "INSERT INTO users (id, full_name, phone_number, password_hash, role, dormitory_id, is_active, created_at)
         VALUES
            ('user-admin-01', 'Ustadz Hendra (Admin Pondok)', '08110000001', ?1, 'ADMIN', NULL, 1, ?5),
            ('user-musyrif-01', 'Ustadz Zaki (Musyrif Putra)', '08110000002', ?2, 'MUSYRIF', 'dorm-putra-01', 1, ?5),
            ('user-musyrif-02', 'Ustadzah Aisyah (Musyrif Putri)', '08110000003', ?2, 'MUSYRIF', 'dorm-putri-01', 1, ?5),
            ('user-asatidz-01', 'KH. Manshur Al-Hafidz (Asatidz)', '08110000004', ?3, 'ASATIDZ', NULL, 1, ?5),
            ('user-wali-01', 'Bapak Fauzi', '081234567801', ?4, 'WALI', NULL, 1, ?5),
            ('user-wali-02', 'Ibu Rahma', '081234567802', ?4, 'WALI', NULL, 1, ?5)"
    )
    .bind(&admin_pass_hash)
    .bind(&musyrif_pass_hash)
    .bind(&ustadz_pass_hash)
    .bind(&wali_pass_hash)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 3. Santri (2 Santri)
    sqlx::query(
        "INSERT INTO santri (
            id, nis, full_name, class_level, dormitory_id, room_name,
            primary_wali_id, total_juz_memorized, current_surah_focus, is_active, created_at
         ) VALUES
            ('santri-01', '2026001', 'Ahmad Zaidan', 'Kelas 2 Wustha', 'dorm-putra-01', 'Kamar 04', 'user-wali-01', 5, 'Juz 6 (QS. Al-Maidah)', 1, ?1),
            ('santri-02', '2026002', 'Nabila Azzahra', 'Kelas 1 Wustha', 'dorm-putri-01', 'Kamar 02', 'user-wali-02', 8, 'Juz 9 (QS. Al-A''raf)', 1, ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 4. Semester Aktif (academic_terms)
    sqlx::query(
        "INSERT INTO academic_terms (id, term_name, academic_year, semester_label, is_active, created_at)
         VALUES ('term-2026-ganjil', '2026/2027 Ganjil', '2026/2027', 'GANJIL', 1, ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 5. Mata Pelajaran (4 Subjects)
    sqlx::query(
        "INSERT INTO subjects (id, code, name, category, kkm_score, is_active, created_at)
         VALUES
            ('subj-fiqih', 'FIQ-01', 'Fiqih Ibadah (Fathul Qarib)', 'DINIYAH', 70, 1, ?1),
            ('subj-nahwu', 'NHW-01', 'Nahwu & Shorof (Jurumiyah)', 'DINIYAH', 70, 1, ?1),
            ('subj-tahfidz', 'THF-01', 'Tahfidz & Tajwid Al-Qur''an', 'TAHFIDZ_QURAN', 75, 1, ?1),
            ('subj-arab', 'ARB-01', 'Bahasa Arab & Muhadatsah', 'UMUM', 70, 1, ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 6. Sampel Log Tahfidz & Mutun (2 baris untuk santri-01)
    sqlx::query(
        "INSERT INTO tahfidz_logs (
            id, santri_id, evaluator_id, session_date, log_type, juz_number,
            surah_or_kitab, ayah_or_bait_start, ayah_or_bait_end, predicate, score_numeric, notes, created_at
         ) VALUES
            ('tahfidz-sample-01', 'santri-01', 'user-asatidz-01', '2026-11-08', 'ZIYADAH', 6, 'QS. Al-Maidah', 1, 15, 'MUMTAZ', 95, 'Tajwid dan makharijul huruf fasih', ?1),
            ('tahfidz-sample-02', 'santri-01', 'user-asatidz-01', '2026-11-09', 'MUTUN_KITAB', 30, 'Nadzom Imrithi', 1, 25, 'JAYYID_JIDDAN', 88, 'Hafalan bait lancar', ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 7. Sampel Ujian (2 sesi ujian pada term-2026-ganjil)
    sqlx::query(
        "INSERT INTO exams (id, term_id, subject_id, exam_type, class_level, exam_date, examiner_id, title, created_at)
         VALUES
            ('exam-fiqih-uts', 'term-2026-ganjil', 'subj-fiqih', 'UTS', 'Kelas 2 Wustha', '2026-10-15', 'user-asatidz-01', 'UTS Fiqih Ibadah Gasal', ?1),
            ('exam-tahfidz-syafahi', 'term-2026-ganjil', 'subj-tahfidz', 'SYAFAHI_LISAN', 'Kelas 2 Wustha', '2026-10-20', 'user-asatidz-01', 'Ujian Lisan Syafahi Tahfidz', ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 8. Sampel Nilai Ujian (2 nilai untuk santri-01)
    sqlx::query(
        "INSERT INTO exam_grades (
            id, exam_id, santri_id, score, letter_grade, predicate_label, teacher_note, recorded_by_id, updated_at
         ) VALUES
            ('grade-sample-01', 'exam-fiqih-uts', 'santri-01', 85.0, 'B', 'Jayyid Jiddan (Sangat Baik)', 'Pahami bab thaharah lebih mendalam', 'user-asatidz-01', ?1),
            ('grade-sample-02', 'exam-tahfidz-syafahi', 'santri-01', 92.0, 'A', 'Mumtaz (Istimewa)', 'Makhraj dan tajwid sangat baik', 'user-asatidz-01', ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 9. Sampel Absensi Harian (1 baris untuk santri-01)
    sqlx::query(
        "INSERT INTO attendance_logs (id, santri_id, attendance_date, activity_type, status, note, recorded_by_id, created_at)
         VALUES ('att-sample-01', 'santri-01', '2026-11-09', 'HALAQAH_TAHFIDZ', 'HADIR', 'Hadir tepat waktu dan tertib', 'user-asatidz-01', ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 10. Sampel Catatan Prestasi Santri (1 baris untuk santri-01)
    sqlx::query(
        "INSERT INTO santri_notes (id, santri_id, note_date, category, title, description, recorded_by_id, created_at)
         VALUES ('note-sample-01', 'santri-01', '2026-11-08', 'PRESTASI', 'Juara 1 Musabaqah Hifdzil Qur''an Internal', 'Menuntaskan tasmi 5 Juz sekali duduk dengan predikat Mumtaz.', 'user-asatidz-01', ?1)"
    )
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!("Deterministic seed data created successfully.");

    Ok(())
}
