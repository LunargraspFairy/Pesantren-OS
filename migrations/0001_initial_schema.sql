-- ============================================================================
-- BAGIAN A: ENTITAS INTI, PENGGUNA, SESI, ASRAMA & SANTRI (4 TABEL)
-- ============================================================================

CREATE TABLE IF NOT EXISTS dormitories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    gender_zone TEXT NOT NULL CHECK (gender_zone IN ('PUTRA', 'PUTRI')),
    building_location TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    full_name TEXT NOT NULL,
    phone_number TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('ADMIN', 'WALI', 'MUSYRIF', 'ASATIDZ')),
    dormitory_id TEXT REFERENCES dormitories(id) ON DELETE SET NULL,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_dormitory ON users(dormitory_id);

CREATE TABLE IF NOT EXISTS sessions (
    token_hash TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);

CREATE TABLE IF NOT EXISTS santri (
    id TEXT PRIMARY KEY NOT NULL,
    nis TEXT NOT NULL UNIQUE,
    full_name TEXT NOT NULL,
    class_level TEXT NOT NULL DEFAULT 'Kelas 1 Ula',
    dormitory_id TEXT NOT NULL REFERENCES dormitories(id) ON DELETE RESTRICT,
    room_name TEXT NOT NULL,
    primary_wali_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    total_juz_memorized INTEGER NOT NULL DEFAULT 0 CHECK (total_juz_memorized >= 0 AND total_juz_memorized <= 30),
    current_surah_focus TEXT NOT NULL DEFAULT 'Juz 30 (An-Naba)',
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_santri_wali ON santri(primary_wali_id);
CREATE INDEX IF NOT EXISTS idx_santri_dormitory ON santri(dormitory_id);
CREATE INDEX IF NOT EXISTS idx_santri_class ON santri(class_level);

-- ============================================================================
-- BAGIAN B: SAMBANG, TITIPAN, LOGBOOK FOTO PRIVAT & OUTBOX WA (4 TABEL)
-- ============================================================================

CREATE TABLE IF NOT EXISTS visit_requests (
    id TEXT PRIMARY KEY NOT NULL,
    ticket_code TEXT NOT NULL UNIQUE,
    wali_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    santri_id TEXT NOT NULL REFERENCES santri(id) ON DELETE RESTRICT,
    requested_date TEXT NOT NULL,
    requested_session TEXT NOT NULL CHECK (requested_session IN ('PAGI', 'SIANG', 'SORE')),
    scheduled_start TEXT,
    scheduled_end TEXT,
    meeting_spot TEXT,
    visitor_count INTEGER NOT NULL DEFAULT 1 CHECK (visitor_count >= 1 AND visitor_count <= 10),
    has_titipan INTEGER NOT NULL DEFAULT 0 CHECK (has_titipan IN (0, 1)),
    titipan_description TEXT,
    current_status TEXT NOT NULL DEFAULT 'REQUESTED' CHECK (
        current_status IN (
            'REQUESTED',
            'SCHEDULED',
            'CONFIRMED_BY_WALI',
            'SANTRI_CALLED',
            'SANTRI_CONFIRMED',
            'COMPLETED',
            'CANCELLED'
        )
    ),
    santri_notified_by_musyrif INTEGER NOT NULL DEFAULT 0 CHECK (santri_notified_by_musyrif IN (0, 1)),
    titipan_confirmed_to_wali INTEGER NOT NULL DEFAULT 0 CHECK (titipan_confirmed_to_wali IN (0, 1)),
    cancel_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_visits_wali ON visit_requests(wali_id);
CREATE INDEX IF NOT EXISTS idx_visits_santri ON visit_requests(santri_id);
CREATE INDEX IF NOT EXISTS idx_visits_status_date ON visit_requests(current_status, requested_date);
CREATE INDEX IF NOT EXISTS idx_visits_date ON visit_requests(requested_date);

CREATE TABLE IF NOT EXISTS visit_status_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    visit_id TEXT NOT NULL REFERENCES visit_requests(id) ON DELETE CASCADE,
    from_status TEXT,
    to_status TEXT NOT NULL,
    actor_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    public_note TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_status_logs_visit ON visit_status_logs(visit_id, id ASC);

CREATE TABLE IF NOT EXISTS visit_logbooks (
    id TEXT PRIMARY KEY NOT NULL,
    visit_id TEXT NOT NULL UNIQUE REFERENCES visit_requests(id) ON DELETE CASCADE,
    captured_by_admin_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    photo_rel_path TEXT NOT NULL,
    photo_size_bytes INTEGER NOT NULL CHECK (photo_size_bytes > 0),
    photo_sha256 TEXT NOT NULL,
    meeting_notes TEXT,
    meeting_timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS notification_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    visit_id TEXT REFERENCES visit_requests(id) ON DELETE SET NULL,
    recipient_phone TEXT NOT NULL,
    event_type TEXT NOT NULL,
    message_body TEXT NOT NULL,
    delivery_status TEXT NOT NULL DEFAULT 'SKIPPED_DISABLED' CHECK (
        delivery_status IN ('SKIPPED_DISABLED', 'PENDING', 'SENT', 'FAILED')
    ),
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    sent_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_outbox_status ON notification_outbox(delivery_status);

-- ============================================================================
-- BAGIAN C: TRACKING HAFALAN AL-QUR'AN & KITAB/MUTUN (1 TABEL)
-- ============================================================================

CREATE TABLE IF NOT EXISTS tahfidz_logs (
    id TEXT PRIMARY KEY NOT NULL,
    santri_id TEXT NOT NULL REFERENCES santri(id) ON DELETE CASCADE,
    evaluator_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    session_date TEXT NOT NULL,
    log_type TEXT NOT NULL CHECK (log_type IN ('ZIYADAH', 'MUROJAAH', 'TASMI_JUZ', 'MUTUN_KITAB')),
    juz_number INTEGER NOT NULL DEFAULT 30 CHECK (juz_number >= 1 AND juz_number <= 30),
    surah_or_kitab TEXT NOT NULL,
    ayah_or_bait_start INTEGER NOT NULL DEFAULT 1 CHECK (ayah_or_bait_start >= 1),
    ayah_or_bait_end INTEGER NOT NULL DEFAULT 1 CHECK (ayah_or_bait_end >= ayah_or_bait_start),
    predicate TEXT NOT NULL CHECK (
        predicate IN ('MUMTAZ', 'JAYYID_JIDDAN', 'JAYYID', 'MAQBUL', 'MENGULANG')
    ),
    score_numeric INTEGER NOT NULL DEFAULT 85 CHECK (score_numeric >= 0 AND score_numeric <= 100),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_tahfidz_santri_date ON tahfidz_logs(santri_id, session_date DESC);

-- ============================================================================
-- BAGIAN D: UJIAN, PENILAIAN AKADEMIK & RAPOR DIGITAL (4 TABEL)
-- ============================================================================

CREATE TABLE IF NOT EXISTS academic_terms (
    id TEXT PRIMARY KEY NOT NULL,
    term_name TEXT NOT NULL UNIQUE,
    academic_year TEXT NOT NULL,
    semester_label TEXT NOT NULL CHECK (semester_label IN ('GANJIL', 'GENAP')),
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Menjamin hanya ada maksimal 1 semester aktif pada satu waktu
CREATE UNIQUE INDEX IF NOT EXISTS idx_single_active_term ON academic_terms(is_active) WHERE is_active = 1;

CREATE TABLE IF NOT EXISTS subjects (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('DINIYAH', 'TAHFIDZ_QURAN', 'UMUM')),
    kkm_score INTEGER NOT NULL DEFAULT 70 CHECK (kkm_score >= 0 AND kkm_score <= 100),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS exams (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL REFERENCES academic_terms(id) ON DELETE RESTRICT,
    subject_id TEXT NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    exam_type TEXT NOT NULL CHECK (exam_type IN ('HARIAN', 'UTS', 'UAS', 'SYAFAHI_LISAN')),
    class_level TEXT NOT NULL,
    exam_date TEXT NOT NULL,
    examiner_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_exams_term_subject ON exams(term_id, subject_id);
CREATE INDEX IF NOT EXISTS idx_exams_date ON exams(exam_date DESC);

CREATE TABLE IF NOT EXISTS exam_grades (
    id TEXT PRIMARY KEY NOT NULL,
    exam_id TEXT NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    santri_id TEXT NOT NULL REFERENCES santri(id) ON DELETE CASCADE,
    score REAL NOT NULL CHECK (score >= 0.0 AND score <= 100.0),
    letter_grade TEXT NOT NULL CHECK (letter_grade IN ('A', 'B', 'C', 'D', 'E')),
    predicate_label TEXT NOT NULL,
    teacher_note TEXT,
    recorded_by_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(exam_id, santri_id)
);

CREATE INDEX IF NOT EXISTS idx_grades_santri ON exam_grades(santri_id);
CREATE INDEX IF NOT EXISTS idx_grades_exam ON exam_grades(exam_id);

-- ============================================================================
-- BAGIAN E: ABSENSI HARIAN & CATATAN KESANTRIAN NON-KEUANGAN (2 TABEL)
-- ============================================================================

CREATE TABLE IF NOT EXISTS attendance_logs (
    id TEXT PRIMARY KEY NOT NULL,
    santri_id TEXT NOT NULL REFERENCES santri(id) ON DELETE CASCADE,
    attendance_date TEXT NOT NULL,
    activity_type TEXT NOT NULL CHECK (
        activity_type IN ('SHOLAT_JAMAAH', 'KBM_DINIYAH', 'HALAQAH_TAHFIDZ', 'ASRAMA_MALAM')
    ),
    status TEXT NOT NULL CHECK (status IN ('HADIR', 'SAKIT', 'IZIN', 'ALFA')),
    note TEXT,
    recorded_by_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(santri_id, attendance_date, activity_type)
);

CREATE INDEX IF NOT EXISTS idx_attendance_santri_date ON attendance_logs(santri_id, attendance_date DESC);

CREATE TABLE IF NOT EXISTS santri_notes (
    id TEXT PRIMARY KEY NOT NULL,
    santri_id TEXT NOT NULL REFERENCES santri(id) ON DELETE CASCADE,
    note_date TEXT NOT NULL,
    category TEXT NOT NULL CHECK (
        category IN ('PRESTASI', 'PEMBINAAN_ADAB', 'IZIN_KESEHATAN_PULANG')
    ),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    recorded_by_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_santri_notes_santri ON santri_notes(santri_id, note_date DESC);
