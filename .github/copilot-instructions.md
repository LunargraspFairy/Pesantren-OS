# SiPesantren — GitHub Copilot & Coding Agent Architecture Guide

This document establishes the non-negotiable architectural invariants, database schema contracts, security policies, and Anti-Slop UI constitution for **SiPesantren** (Sistem Informasi Lembaga Kepengasuhan, Tahfidz & Akademik Pondok Pesantren).

---

## 1. Scope & Core Boundaries

1. **100% Non-Financial Domain**:
   - Strictly NO tuition fees (SPP), NO payment gateways (Midtrans, Xendit, etc.), NO billing invoices, and NO monetary ledgers.
   - Any pull request or prompt introducing financial transactions, wallets, or payments violates system boundaries and must be rejected immediately.

2. **The 4 Non-Financial Pillars**:
   - **Pilar 1 (Sambang & Titipan)**: 6-stage verified visit workflow (`REQUESTED` → `SCHEDULED` → `CONFIRMED_BY_WALI` → `SANTRI_CALLED` → `SANTRI_CONFIRMED` → `COMPLETED`) with physical item verification badge (`ADA TITIPAN`) and encrypted/private logbook photos.
   - **Pilar 2 (Tahfidz & Mutun)**: Continuous tracking of Quranic memorization (Ziyadah & Muraja'ah across 30 Juz) with surah, verses, predicate, score, and automatic progress calculation.
   - **Pilar 3 (Ujian & Rapor Digital)**: Academic terms, subject assessments (Harian, UTS, UAS, Syafahi), weighted grading, letter grade calculation (A/B/C/D), and print-ready digital report cards (`@media print`).
   - **Pilar 4 (Kehadiran & Kesantrian)**: Daily activities (Asrama Malam, Sholat Subuh, Kajian Kitab), disciplinary tracking, health status, and achievements visible to guardians.

---

## 2. Database Schema (SQLite WAL — 15 Tables)

Engine: SQLite 3 with `PRAGMA journal_mode = WAL`, `PRAGMA synchronous = NORMAL`, `PRAGMA foreign_keys = ON`, `PRAGMA busy_timeout = 10000`, and `PRAGMA cache_size = -64000`.

### Schema Summary:
1. `dormitories`: Asrama data (`id`, `name`, `gender_zone`, `capacity`, `building_location`).
2. `users`: Credentials and roles (`id`, `full_name`, `phone_number`, `password_hash`, `role` [ADMIN|WALI|MUSYRIF|ASATIDZ], `dormitory_id`, `is_active`).
3. `sessions`: Ephemeral token storage (`id`, `user_id`, `token_hash`, `expires_at`, `created_at`).
4. `santri`: Student records (`id`, `nis`, `full_name`, `gender`, `dormitory_id`, `room_name`, `class_level`, `total_juz_memorized`, `is_active`).
5. `santri_guardians`: Relational mapping (`id`, `santri_id`, `guardian_user_id`, `relationship_type`).
6. `academic_terms`: Semester periods (`id`, `term_name`, `academic_year`, `semester_label`, `is_active`). Enforces partial unique index `idx_single_active_term` on `is_active = 1`.
7. `subjects`: Curricular courses (`id`, `code`, `name`, `category`, `kkm_score`).
8. `visits`: Core visit lifecycle records (`id`, `ticket_code`, `santri_id`, `wali_user_id`, `requested_date`, `requested_session`, `scheduled_start`, `scheduled_end`, `meeting_spot`, `has_titipan`, `titipan_description`, `current_status`, `visitor_count`, `santri_notified_by_musyrif`, `titipan_confirmed_to_wali`, `has_logbook`).
9. `visit_status_logs`: Append-only audit trail (`id`, `visit_id`, `from_status`, `to_status`, `actor_id`, `public_note`, `created_at`).
10. `visit_logbooks`: Private meeting documentation (`id`, `visit_id`, `photo_filename`, `mime_type`, `file_size_bytes`, `meeting_notes`, `recorded_by_admin_id`, `created_at`).
11. `tahfidz_logs`: Memorization records (`id`, `santri_id`, `evaluator_id`, `session_date`, `log_type`, `juz_number`, `surah_or_kitab`, `ayah_or_bait_start`, `ayah_or_bait_end`, `predicate`, `score_numeric`, `notes`).
12. `exams`: Exam sessions (`id`, `term_id`, `subject_id`, `exam_type`, `class_level`, `exam_date`, `examiner_id`, `title`, `max_score`, `weight_percentage`).
13. `exam_grades`: Student grades (`id`, `exam_id`, `santri_id`, `score`, `letter_grade`, `predicate_label`, `notes`).
14. `attendance_logs`: Daily roll calls (`id`, `santri_id`, `recorded_by_id`, `attendance_date`, `activity_type`, `status`, `notes`).
15. `santri_notes`: Character and conduct records (`id`, `santri_id`, `recorded_by_id`, `note_date`, `category`, `title`, `description`, `is_public_to_wali`).

### Transaction Rule:
- **Write-First SQLite Rule**: Any transaction started with `let mut tx = state.db.begin().await?;` must execute `INSERT` or `UPDATE` as its first statement to instantly acquire the SQLite `IMMEDIATE` write lock, preventing concurrent read-to-write deadlocks.

---

## 3. Security & RBAC Isolation

1. **Role Access Control**:
   - `ADMIN`: Full operational authority over scheduling, calling santri, and uploading logbooks.
   - `WALI`: Restricted strictly to their own linked santri.
   - `MUSYRIF`: Restricted to santri residing in their assigned dormitory (`dormitory_id`).
   - `ASATIDZ`: High-speed entry of tahfidz logs, exam sessions, and grading.

2. **Logbook Meeting Photo Privacy Invariant**:
   - Endpoint: `GET /api/visits/:id/logbook/photo`.
   - Access is permitted **ONLY** to `ADMIN` and the specific `WALI` who owns the visit record (`wali_user_id == user.id`).
   - `MUSYRIF`, `ASATIDZ`, and any other `WALI` MUST be rejected with **`403 FORBIDDEN`**.
   - Unauthenticated requests MUST be rejected with **`401 UNAUTHORIZED`**.
   - Meeting photos MUST NEVER be placed in public `/static` directories.

3. **Public Flow Data Protection**:
   - Endpoint: `/alur` and `/api/sse/public-flow`.
   - Santri names MUST be masked (e.g., `Mu****** **-******`).
   - Guardian phone numbers, private notes, and meeting photos are strictly stripped from public responses.

---

## 4. Askama SSR Type-Safety Rules

1. **No Raw Option Rendering**:
   - In Askama 0.12.1, `Option<T>` does NOT implement `Display`.
   - NEVER write `{{ item.meeting_spot }}` or `{{ visit.meeting_notes }}`.
   - ALWAYS use explicit display helper methods:
     - `.schedule_window_display()`
     - `.spot_display()`
     - `.titipan_display()`
     - `.meeting_notes_display()`
     - `.teacher_note_display()`
     - `.notes_display()`
     - `.note_display()`
   - For formatted grades in Rapor, use pre-formatted string fields: `harian_display`, `uts_display`, `uas_display`, `syafahi_display`, `final_display`.
   - For optional blocks, use `{% if let Some(x) = opt %} ... {{ x }} ... {% endif %}`.

---

## 5. Anti-Slop UI & Branding Constitution

1. **Branding & Logo**:
   - Official URL: `https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQ9GEdwjS0LGmdqXny3Td834UU4FwxsV4DsvbxtRKx_Rh8mJd2IQv599LF5&s=10`.
   - Application Name: **SiPesantren**.

2. **Color Palette (Institutional Emerald)**:
   - Primary: `bg-emerald-950` (#022c22) and `bg-emerald-800` (#065f46).
   - Accents: `bg-amber-500` (#f59e0b) for active alerts and titipan badges.
   - Canvas: `bg-slate-50` (#f8fafc) and cards `bg-white` with clean hairline borders `border-slate-200`.

3. **Prohibited Patterns (Anti-AI Slop)**:
   - ZERO decorative emojis in labels, buttons, cards, or tables.
   - ZERO neon/purple gradients (`from-purple-600 to-indigo-600` is strictly forbidden).
   - ZERO `window.alert()` or `window.confirm()` in client JavaScript (use inline modals and badges).
   - MANDATORY `font-mono tabular-nums` for all numbers, dates, times, NIS, Juz, and ticket codes.
