import React, { useState } from 'react';
import {
  ShieldCheck,
  Lock,
  CheckCircle2,
  Clock,
  AlertTriangle,
  FileText,
  BookOpen,
  Calendar,
  Camera,
  Bell,
  User,
  Printer,
  Search,
  Building2,
  Volume2,
  Check,
  ChevronRight,
  Sparkles,
  ArrowRight,
  ExternalLink,
  X
} from 'lucide-react';

const PONDOK_LOGO_URL =
  'https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQ9GEdwjS0LGmdqXny3Td834UU4FwxsV4DsvbxtRKx_Rh8mJd2IQv599LF5&s=10';

type Role = 'wali' | 'admin' | 'musyrif' | 'asatidz' | 'public';
type VisitStatus =
  | 'REQUESTED'
  | 'SCHEDULED'
  | 'CONFIRMED_BY_WALI'
  | 'SANTRI_CALLED'
  | 'SANTRI_CONFIRMED'
  | 'COMPLETED';

interface Visit {
  id: string;
  ticket_code: string;
  santri_id: string;
  santri_name: string;
  masked_name: string;
  dormitory_name: string;
  room_name: string;
  wali_name: string;
  wali_phone: string;
  requested_date: string;
  requested_session: string;
  scheduled_start?: string;
  scheduled_end?: string;
  meeting_spot?: string;
  has_titipan: boolean;
  titipan_description?: string;
  visitor_count: number;
  current_status: VisitStatus;
  santri_notified_by_musyrif: boolean;
  titipan_confirmed_to_wali: boolean;
  has_logbook: boolean;
  logbook_photo_url?: string;
  meeting_notes?: string;
  created_at: string;
  updated_at: string;
}

interface TahfidzLog {
  id: string;
  session_date: string;
  log_type: string;
  juz_number: number;
  surah_or_kitab: string;
  ayah_start: number;
  ayah_end: number;
  predicate: string;
  score: number;
  evaluator_name: string;
  notes?: string;
}

interface RaporRow {
  subject_code: string;
  subject_name: string;
  category: string;
  kkm_score: number;
  harian: number;
  uts: number;
  uas: number;
  syafahi: number;
  final_score: number;
  letter_grade: string;
  predicate_label: string;
}

interface AttendanceRecord {
  id: string;
  date: string;
  activity_type: string;
  status: string;
  notes?: string;
}

interface SantriNote {
  id: string;
  date: string;
  category: string;
  title: string;
  description: string;
  recorded_by: string;
}

// Clean status helper mapping internal enums to elegant Indonesian UI labels
function getStatusBadge(status: VisitStatus) {
  switch (status) {
    case 'REQUESTED':
      return {
        label: 'Menunggu Jadwal',
        badgeClass: 'bg-slate-100 text-slate-800 border-slate-300',
        dotClass: 'bg-slate-400',
      };
    case 'SCHEDULED':
      return {
        label: 'Jadwal Ditetapkan',
        badgeClass: 'bg-blue-50 text-blue-900 border-blue-200',
        dotClass: 'bg-blue-600',
      };
    case 'CONFIRMED_BY_WALI':
      return {
        label: 'Wali Tiba di Posko',
        badgeClass: 'bg-indigo-50 text-indigo-900 border-indigo-200',
        dotClass: 'bg-indigo-600',
      };
    case 'SANTRI_CALLED':
      return {
        label: 'Dipanggil ke Posko',
        badgeClass: 'bg-amber-100 text-amber-950 border-amber-300 animate-pulse',
        dotClass: 'bg-amber-500 animate-ping',
      };
    case 'SANTRI_CONFIRMED':
      return {
        label: 'Santri Menuju Posko',
        badgeClass: 'bg-emerald-100 text-emerald-900 border-emerald-300',
        dotClass: 'bg-emerald-600',
      };
    case 'COMPLETED':
      return {
        label: 'Selesai di Posko',
        badgeClass: 'bg-emerald-950 text-emerald-100 border-emerald-900',
        dotClass: 'bg-emerald-400',
      };
  }
}

export default function App() {
  const [currentRole, setCurrentRole] = useState<Role>('wali');
  const [waliTab, setWaliTab] = useState<'sambang' | 'tahfidz' | 'rapor' | 'kesantrian'>('sambang');
  const [asatidzPanel, setAsatidzPanel] = useState<'tahfidz' | 'ujian' | 'kesantrian'>('tahfidz');
  const [publicTicketSearch, setPublicTicketSearch] = useState('');
  const [securityModalOpen, setSecurityModalOpen] = useState(false);
  const [securityAuditRole, setSecurityAuditRole] = useState<Role | 'other_wali' | 'anon'>('wali');
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // In-Memory Seed State
  const [santriTotalJuz, setSantriTotalJuz] = useState<number>(5);
  const [visits, setVisits] = useState<Visit[]>([
    {
      id: 'vis-01',
      ticket_code: 'TKT-20261120-001',
      santri_id: 'santri-01',
      santri_name: 'Muhammad Fatih Al-Faruq',
      masked_name: 'Mu****** **-******',
      dormitory_name: 'Asrama Al-Farabi (Putra)',
      room_name: 'Kamar Abu Bakar 02',
      wali_name: 'Ir. H. Ahmad Dahlan',
      wali_phone: '081234567801',
      requested_date: '2026-11-20',
      requested_session: 'Sesi Pagi',
      scheduled_start: '09:00',
      scheduled_end: '09:45',
      meeting_spot: 'Gazebo Utama Putra 01',
      has_titipan: true,
      titipan_description: 'Kurma Ajwa 1kg & Kitab Fathul Qorib',
      visitor_count: 2,
      current_status: 'SANTRI_CALLED',
      santri_notified_by_musyrif: false,
      titipan_confirmed_to_wali: false,
      has_logbook: false,
      created_at: '2026-11-20 07:30',
      updated_at: '2026-11-20 08:45',
    },
    {
      id: 'vis-02',
      ticket_code: 'TKT-20261113-088',
      santri_id: 'santri-01',
      santri_name: 'Muhammad Fatih Al-Faruq',
      masked_name: 'Mu****** **-******',
      dormitory_name: 'Asrama Al-Farabi (Putra)',
      room_name: 'Kamar Abu Bakar 02',
      wali_name: 'Ir. H. Ahmad Dahlan',
      wali_phone: '081234567801',
      requested_date: '2026-11-13',
      requested_session: 'Sesi Siang',
      scheduled_start: '13:30',
      scheduled_end: '14:15',
      meeting_spot: 'Ruang Tamu Keluarga Posko',
      has_titipan: false,
      visitor_count: 3,
      current_status: 'COMPLETED',
      santri_notified_by_musyrif: true,
      titipan_confirmed_to_wali: false,
      has_logbook: true,
      logbook_photo_url:
        'https://images.unsplash.com/photo-1544717305-2782549b5136?auto=format&fit=crop&w=600&q=80',
      meeting_notes: 'Pertemuan tatap muka berjalan lancar dan tertib, santri sehat serta siap melanjutkan hafalan.',
      created_at: '2026-11-13 08:00',
      updated_at: '2026-11-13 14:20',
    },
  ]);

  const [tahfidzLogs, setTahfidzLogs] = useState<TahfidzLog[]>([
    {
      id: 'tahf-1',
      session_date: '2026-11-19',
      log_type: 'ZIYADAH (Hafalan Baru)',
      juz_number: 5,
      surah_or_kitab: 'QS. An-Nisa',
      ayah_start: 148,
      ayah_end: 176,
      predicate: 'Mumtaz (95)',
      score: 95,
      evaluator_name: 'Ust. Ahmad Fauzi, Lc., M.Ag.',
      notes: 'Makharijul huruf fasih, tajwid dan waqaf sangat tepat.',
    },
    {
      id: 'tahf-2',
      session_date: '2026-11-17',
      log_type: 'MURAJA`AH (Pengulangan)',
      juz_number: 4,
      surah_or_kitab: 'QS. Ali Imran',
      ayah_start: 92,
      ayah_end: 150,
      predicate: 'Jayyid Jiddan (88)',
      score: 88,
      evaluator_name: 'Ust. Ahmad Fauzi, Lc., M.Ag.',
      notes: 'Kelancaran baik, pertahankan tempo tilawah tartil.',
    },
    {
      id: 'tahf-3',
      session_date: '2026-11-14',
      log_type: 'ZIYADAH (Hafalan Baru)',
      juz_number: 5,
      surah_or_kitab: 'QS. An-Nisa',
      ayah_start: 100,
      ayah_end: 147,
      predicate: 'Mumtaz (93)',
      score: 93,
      evaluator_name: 'Ust. Ahmad Fauzi, Lc., M.Ag.',
      notes: 'Hafalan matang tanpa tersendat.',
    },
  ]);

  const [raporRows, setRaporRows] = useState<RaporRow[]>([
    {
      subject_code: 'MP-01',
      subject_name: 'Tahfidz Al-Qur`an 30 Juz',
      category: 'Al-Qur`an & Ulumul Qur`an',
      kkm_score: 75,
      harian: 94,
      uts: 92,
      uas: 95,
      syafahi: 96,
      final_score: 94.2,
      letter_grade: 'A',
      predicate_label: 'Mumtaz (Istimewa)',
    },
    {
      subject_code: 'MP-02',
      subject_name: 'Fiqih Ibadah & Muamalah (Fathul Qorib)',
      category: 'Dirasah Islamiyah',
      kkm_score: 70,
      harian: 88,
      uts: 85,
      uas: 90,
      syafahi: 87,
      final_score: 87.6,
      letter_grade: 'B',
      predicate_label: 'Jayyid Jiddan (Sangat Baik)',
    },
    {
      subject_code: 'MP-03',
      subject_name: 'Bahasa Arab & Nahwu Sharaf (Al-Jurumiyah)',
      category: 'Kaidah Bahasa',
      kkm_score: 70,
      harian: 84,
      uts: 82,
      uas: 86,
      syafahi: 85,
      final_score: 84.4,
      letter_grade: 'B',
      predicate_label: 'Jayyid Jiddan (Sangat Baik)',
    },
    {
      subject_code: 'MP-04',
      subject_name: 'Hadits Nabawi (Arbain An-Nawawiyah)',
      category: 'Dirasah Islamiyah',
      kkm_score: 70,
      harian: 90,
      uts: 88,
      uas: 92,
      syafahi: 91,
      final_score: 90.3,
      letter_grade: 'A',
      predicate_label: 'Mumtaz (Istimewa)',
    },
  ]);

  const [attendanceRecords, setAttendanceRecords] = useState<AttendanceRecord[]>([
    {
      id: 'att-1',
      date: '2026-11-20',
      activity_type: 'Presensi Malam Asrama',
      status: 'Hadir',
      notes: 'Tepat waktu di kamar Abu Bakar 02.',
    },
    {
      id: 'att-2',
      date: '2026-11-20',
      activity_type: 'Sholat Subuh Berjamaah',
      status: 'Hadir',
      notes: 'Shaf pertama Masjid Jami`.',
    },
    {
      id: 'att-3',
      date: '2026-11-19',
      activity_type: 'Presensi Malam Asrama',
      status: 'Hadir',
    },
    {
      id: 'att-4',
      date: '2026-11-19',
      activity_type: 'Kajian Kitab Kuning',
      status: 'Hadir',
    },
  ]);

  const [santriNotes, setSantriNotes] = useState<SantriNote[]>([
    {
      id: 'sn-1',
      date: '2026-11-18',
      category: 'Prestasi',
      title: 'Juara 1 Musabaqah Hifdzil Qur`an',
      description: 'Meraih peringkat pertama MHQ 5 Juz Tingkat Kabupaten mewakili Pondok Pesantren.',
      recorded_by: 'Ust. Zulkifli, S.Pd.I',
    },
    {
      id: 'sn-2',
      date: '2026-11-12',
      category: 'Kedisiplinan & Adab',
      title: 'Teladan Kebersihan Kamar Asrama',
      description: 'Menjadi ketua regu piket asrama terbaik dengan nilai inspeksi kebersihan 98/100.',
      recorded_by: 'Ust. Zulkifli, S.Pd.I',
    },
  ]);

  // Form States for Simulation
  const [newVisitReq, setNewVisitReq] = useState({
    requested_date: '2026-11-25',
    requested_session: 'Sesi Pagi (08:00 - 11:30)',
    has_titipan: true,
    titipan_description: 'Madu Alami & Buku Doa Harian',
    visitor_count: 2,
  });

  const [tahfidzForm, setTahfidzForm] = useState({
    session_date: '2026-11-20',
    log_type: 'ZIYADAH',
    juz_number: 6,
    surah_or_kitab: 'QS. Al-Maidah',
    ayah_start: 1,
    ayah_end: 25,
    predicate: 'Mumtaz (95)',
    score: 95,
    notes: 'Awal juz 6 lancar dan tajwid tepat.',
    update_juz: true,
  });

  const [gradeForm, setGradeForm] = useState({
    subject_code: 'MP-02',
    score: 92,
  });

  const triggerToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => {
      setToastMessage(null);
    }, 3200);
  };

  // Web Audio API Pleasant Dual-Tone Chime (880Hz -> 1320Hz)
  const playAirportChime = () => {
    try {
      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      if (!AudioCtx) return;
      const ctx = new AudioCtx();
      const now = ctx.currentTime;
      const osc1 = ctx.createOscillator();
      const osc2 = ctx.createOscillator();
      const gain = ctx.createGain();

      osc1.type = 'sine';
      osc1.frequency.setValueAtTime(880, now); // A5

      osc2.type = 'sine';
      osc2.frequency.setValueAtTime(1320, now + 0.16); // E6

      gain.gain.setValueAtTime(0.25, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.85);

      osc1.connect(gain);
      osc2.connect(gain);
      gain.connect(ctx.destination);

      osc1.start(now);
      osc1.stop(now + 0.16);
      osc2.start(now + 0.16);
      osc2.stop(now + 0.85);
    } catch {
      // AudioContext fallback handled silently
    }
  };

  // State Transition Handlers
  const handleWaliRequest = (e: React.FormEvent) => {
    e.preventDefault();
    const newVisit: Visit = {
      id: `vis-${Date.now()}`,
      ticket_code: `TKT-20261125-${Math.floor(100 + Math.random() * 900)}`,
      santri_id: 'santri-01',
      santri_name: 'Muhammad Fatih Al-Faruq',
      masked_name: 'Mu****** **-******',
      dormitory_name: 'Asrama Al-Farabi (Putra)',
      room_name: 'Kamar Abu Bakar 02',
      wali_name: 'Ir. H. Ahmad Dahlan',
      wali_phone: '081234567801',
      requested_date: newVisitReq.requested_date,
      requested_session: newVisitReq.requested_session,
      has_titipan: newVisitReq.has_titipan,
      titipan_description: newVisitReq.titipan_description,
      visitor_count: newVisitReq.visitor_count,
      current_status: 'REQUESTED',
      santri_notified_by_musyrif: false,
      titipan_confirmed_to_wali: false,
      has_logbook: false,
      created_at: '2026-11-20 09:00',
      updated_at: '2026-11-20 09:00',
    };
    setVisits([newVisit, ...visits]);
    triggerToast('Permohonan sambang berhasil dikirim ke Posko Pelayanan');
  };

  const handleAdminSchedule = (visitId: string) => {
    setVisits(
      visits.map((v) =>
        v.id === visitId
          ? {
              ...v,
              current_status: 'SCHEDULED',
              scheduled_start: '10:00',
              scheduled_end: '10:45',
              meeting_spot: 'Gazebo Utama Putra 02',
              updated_at: '2026-11-20 09:15',
            }
          : v
      )
    );
    triggerToast('Jadwal kunjungan telah ditetapkan di Gazebo Utama');
  };

  const handleWaliConfirm = (visitId: string) => {
    setVisits(
      visits.map((v) =>
        v.id === visitId
          ? {
              ...v,
              current_status: 'CONFIRMED_BY_WALI',
              updated_at: '2026-11-20 09:30',
            }
          : v
      )
    );
    triggerToast('Konfirmasi kehadiran Wali berhasil diterima di Posko');
  };

  const handleAdminCall = (visitId: string) => {
    playAirportChime();
    setVisits(
      visits.map((v) =>
        v.id === visitId
          ? {
              ...v,
              current_status: 'SANTRI_CALLED',
              updated_at: '2026-11-20 09:40',
            }
          : v
      )
    );
    triggerToast('Panggilan tersiar ke Asrama dengan chime otomatis');
  };

  const handleMusyrifConfirm = (visitId: string) => {
    setVisits(
      visits.map((v) =>
        v.id === visitId
          ? {
              ...v,
              current_status: 'SANTRI_CONFIRMED',
              santri_notified_by_musyrif: true,
              titipan_confirmed_to_wali: true,
              updated_at: '2026-11-20 09:45',
            }
          : v
      )
    );
    triggerToast('Santri & titipan dikonfirmasi siap menuju Posko');
  };

  const handleAdminCompleteLogbook = (visitId: string) => {
    setVisits(
      visits.map((v) =>
        v.id === visitId
          ? {
              ...v,
              current_status: 'COMPLETED',
              has_logbook: true,
              logbook_photo_url:
                'https://images.unsplash.com/photo-1544717305-2782549b5136?auto=format&fit=crop&w=600&q=80',
              meeting_notes:
                'Pertemuan tatap muka selesai dan tertib. Titipan kurma dan kitab kuning telah diserahkan langsung ke santri.',
              updated_at: '2026-11-20 10:40',
            }
          : v
      )
    );
    triggerToast('Logbook foto pertemuan (WebP) berhasil diarsipkan');
  };

  const handleAddTahfidz = (e: React.FormEvent) => {
    e.preventDefault();
    const newLog: TahfidzLog = {
      id: `tahf-${Date.now()}`,
      session_date: tahfidzForm.session_date,
      log_type: tahfidzForm.log_type === 'ZIYADAH' ? 'ZIYADAH (Hafalan Baru)' : 'MURAJA`AH (Pengulangan)',
      juz_number: Number(tahfidzForm.juz_number),
      surah_or_kitab: tahfidzForm.surah_or_kitab,
      ayah_start: Number(tahfidzForm.ayah_start),
      ayah_end: Number(tahfidzForm.ayah_end),
      predicate: tahfidzForm.predicate,
      score: Number(tahfidzForm.score),
      evaluator_name: 'Ust. Ahmad Fauzi, Lc., M.Ag.',
      notes: tahfidzForm.notes,
    };
    setTahfidzLogs([newLog, ...tahfidzLogs]);
    if (tahfidzForm.update_juz && Number(tahfidzForm.juz_number) > santriTotalJuz) {
      setSantriTotalJuz(Number(tahfidzForm.juz_number));
    }
    triggerToast(`Setoran Juz ${tahfidzForm.juz_number} tersimpan dan progres 30 Juz diperbarui`);
  };

  const handleUpdateGrade = (e: React.FormEvent) => {
    e.preventDefault();
    setRaporRows(
      raporRows.map((r) => {
        if (r.subject_code === gradeForm.subject_code) {
          const uas = Number(gradeForm.score);
          const finalScore = Number(((r.harian * 0.2) + (r.uts * 0.3) + (uas * 0.3) + (r.syafahi * 0.2)).toFixed(1));
          let letter = 'D';
          let pred = 'Rasib (Kurang)';
          if (finalScore >= 90) {
            letter = 'A';
            pred = 'Mumtaz (Istimewa)';
          } else if (finalScore >= 80) {
            letter = 'B';
            pred = 'Jayyid Jiddan (Sangat Baik)';
          } else if (finalScore >= 70) {
            letter = 'C';
            pred = 'Maqbul (Cukup)';
          }
          return {
            ...r,
            uas,
            final_score: finalScore,
            letter_grade: letter,
            predicate_label: pred,
          };
        }
        return r;
      })
    );
    triggerToast('Nilai ujian tersimpan dan Rapor Digital langsung diperbarui');
  };

  const activeVisit = visits.find((v) => v.current_status !== 'COMPLETED');
  const completedVisits = visits.filter((v) => v.current_status === 'COMPLETED');

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900 font-sans antialiased flex flex-col selection:bg-emerald-900 selection:text-white">
      {/* INSTITUTIONAL TOPBAR (Clean Product Hierarchy, No Spec Leaks) */}
      <header className="bg-emerald-950 text-white border-b border-emerald-900 sticky top-0 z-40 shadow-sm print:hidden">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <img
              src={PONDOK_LOGO_URL}
              alt="Logo Resmi Pondok Pesantren"
              className="h-10 w-10 rounded-md object-contain bg-white p-0.5 border border-emerald-800/40 shadow-xs"
            />
            <div>
              <div className="font-bold text-base sm:text-lg tracking-tight text-white leading-none">
                SiPesantren
              </div>
              <div className="text-[11px] text-emerald-300 font-medium tracking-normal mt-0.5">
                Sistem Terpadu Kepengasuhan, Tahfidz & Akademik
              </div>
            </div>
          </div>

          <div className="flex items-center space-x-2.5">
            {/* Quick Chime Audio Tester Button */}
            <button
              onClick={playAirportChime}
              title="Bunyikan Chime Panggilan Web Audio API"
              className="btn-interactive flex items-center space-x-1.5 px-3 py-1.5 rounded-md bg-emerald-900/80 hover:bg-emerald-900 text-emerald-200 hover:text-white text-xs border border-emerald-800"
            >
              <Volume2 className="w-3.5 h-3.5 text-emerald-300" />
              <span className="hidden sm:inline">Uji Chime Asrama</span>
            </button>

            {/* Active User Indicator */}
            <div className="hidden sm:flex items-center space-x-2 pl-3 border-l border-emerald-800/60 text-xs">
              <div className="w-2 h-2 rounded-full bg-emerald-400"></div>
              <span className="text-emerald-200 font-medium">Sesi Terverifikasi</span>
            </div>
          </div>
        </div>

        {/* ROLE BAR: SINGLE-LINE HORIZONTAL SEGMENTED PILL BAR (ibelick / motion-primitives) */}
        <div className="bg-emerald-900/70 border-t border-emerald-900/90 px-4 sm:px-6 lg:px-8">
          <div className="max-w-7xl mx-auto flex items-center space-x-1.5 py-2 overflow-x-auto no-scrollbar">
            {[
              { id: 'wali', label: 'Portal Wali Santri', desc: 'Ir. H. Ahmad Dahlan' },
              { id: 'admin', label: 'Posko Pelayanan Sambang', desc: 'Penjadwalan & Logbook' },
              { id: 'musyrif', label: 'Monitor Asrama', desc: 'Zona Asrama Putra' },
              { id: 'asatidz', label: 'Portal Dewan Asatidz', desc: 'Tahfidz & Evaluasi' },
              { id: 'public', label: 'Pelacakan Publik', desc: 'Status Transparansi' },
            ].map((r) => {
              const active = currentRole === r.id;
              return (
                <button
                  key={r.id}
                  onClick={() => setCurrentRole(r.id as Role)}
                  className={`btn-interactive flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium whitespace-nowrap shrink-0 transition-colors ${
                    active
                      ? 'bg-emerald-800 text-white font-bold ring-1 ring-emerald-400 shadow-xs'
                      : 'text-emerald-200 hover:text-white hover:bg-emerald-900/80'
                  }`}
                >
                  <span>{r.label}</span>
                  <span className="text-[10px] opacity-60 font-normal hidden md:inline">({r.desc})</span>
                </button>
              );
            })}
          </div>
        </div>
      </header>

      {/* TOAST NOTIFICATION CONTAINER (Sonner/Vaul Pattern with @starting-style easing) */}
      {toastMessage && (
        <div className="fixed bottom-5 right-5 z-50 animate-view-enter">
          <div className="bg-emerald-950 text-white text-xs font-medium px-4 py-3 rounded-lg shadow-xl border border-emerald-800 flex items-center space-x-2.5 max-w-sm">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span>{toastMessage}</span>
          </div>
        </div>
      )}

      {/* MAIN CONTENT AREA */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 flex-1 w-full animate-view-enter" key={currentRole}>
        {/* =========================================================================
            ROLE 1: PORTAL WALI SANTRI (4 TABS WITH BLUR-MASKED CROSSFADE)
            ========================================================================= */}
        {currentRole === 'wali' && (
          <div className="space-y-6">
            {/* Student Profile Bar */}
            <div className="bg-white border border-slate-200 rounded-xl p-4 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div className="flex items-center space-x-3.5">
                <div className="h-11 w-11 rounded-lg bg-emerald-100 border border-emerald-300 flex items-center justify-center text-emerald-900 font-bold text-sm shrink-0">
                  FA
                </div>
                <div>
                  <div className="text-[11px] text-slate-500 font-medium tracking-normal">Santri Binaan</div>
                  <h1 className="text-base font-bold text-slate-900 tracking-tight">Muhammad Fatih Al-Faruq</h1>
                  <div className="text-xs text-slate-600 font-mono">
                    NIS: <span className="font-semibold text-slate-800">2026-001</span> • Asrama Al-Farabi (Kamar Abu Bakar 02) • Kelas 1 Aliyah
                  </div>
                </div>
              </div>

              {/* 4-Tab Single-Line Horizontal Segmented Pill Bar (No wrapping on mobile) */}
              <div className="bg-slate-100 p-1 rounded-lg flex items-center space-x-1 overflow-x-auto no-scrollbar shrink-0 border border-slate-200">
                {[
                  { id: 'sambang', label: 'Sambang & Titipan' },
                  { id: 'tahfidz', label: `Tahfidz (${santriTotalJuz} Juz)` },
                  { id: 'rapor', label: 'Rapor Akademik' },
                  { id: 'kesantrian', label: 'Presensi & Adab' },
                ].map((t) => (
                  <button
                    key={t.id}
                    onClick={() => setWaliTab(t.id as any)}
                    className={`btn-interactive px-3 py-1.5 rounded-md text-xs whitespace-nowrap transition-colors ${
                      waliTab === t.id
                        ? 'bg-emerald-950 text-white font-bold shadow-xs'
                        : 'text-slate-700 hover:text-slate-900 hover:bg-slate-200 font-medium'
                    }`}
                  >
                    {t.label}
                  </button>
                ))}
              </div>
            </div>

            {/* TAB CONTENT: BLUR-MASKED CROSSFADE (Emil Kowalski Design Eng) */}
            <div className="animate-view-enter" key={waliTab}>
              {/* TAB 1: SAMBANG & TITIPAN */}
              {waliTab === 'sambang' && (
                <div className="space-y-6">
                  {/* Active Visit Tracking Card (Cal.com Booking Slot & Stepper style) */}
                  {activeVisit && (
                    <div className="bg-white border-2 border-emerald-800 rounded-xl p-5 shadow-xs space-y-4">
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-slate-200 gap-2">
                        <div>
                          <span className="text-[11px] font-semibold text-emerald-800 uppercase tracking-wider">
                            Status Pelaksanaan Sambang
                          </span>
                          <div className="text-base font-bold text-slate-900 font-mono tabular-nums">
                            {activeVisit.ticket_code}
                          </div>
                        </div>

                        <div className="flex items-center space-x-2">
                          {activeVisit.has_titipan && (
                            <span className="px-2.5 py-1 rounded-full text-xs font-bold bg-amber-50 text-amber-950 border border-amber-300 flex items-center space-x-1.5">
                              <span className="w-2 h-2 rounded-full bg-amber-500"></span>
                              <span>ADA TITIPAN FISIK</span>
                            </span>
                          )}
                          <span className={`px-2.5 py-1 rounded-md text-xs font-bold border flex items-center space-x-1.5 ${getStatusBadge(activeVisit.current_status).badgeClass}`}>
                            <span className={`w-1.5 h-1.5 rounded-full ${getStatusBadge(activeVisit.current_status).dotClass}`}></span>
                            <span>{getStatusBadge(activeVisit.current_status).label}</span>
                          </span>
                        </div>
                      </div>

                      {/* 6-Stage Stepper */}
                      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2 text-center text-xs">
                        {[
                          { num: '1', title: 'Diajukan', status: 'REQUESTED' },
                          { num: '2', title: 'Terjadwal', status: 'SCHEDULED' },
                          { num: '3', title: 'Wali Tiba', status: 'CONFIRMED_BY_WALI' },
                          { num: '4', title: 'Dipanggil', status: 'SANTRI_CALLED' },
                          { num: '5', title: 'Menuju Posko', status: 'SANTRI_CONFIRMED' },
                          { num: '6', title: 'Selesai', status: 'COMPLETED' },
                        ].map((step, idx) => {
                          const isCurrent = activeVisit.current_status === step.status;
                          const isPast =
                            ['REQUESTED', 'SCHEDULED', 'CONFIRMED_BY_WALI', 'SANTRI_CALLED', 'SANTRI_CONFIRMED', 'COMPLETED'].indexOf(
                              activeVisit.current_status
                            ) >= idx;
                          return (
                            <div
                              key={step.num}
                              className={`p-2.5 rounded-lg border transition-all ${
                                isCurrent
                                  ? 'bg-emerald-950 text-white font-bold border-emerald-950 shadow-xs'
                                  : isPast
                                  ? 'bg-emerald-50 text-emerald-950 border-emerald-200'
                                  : 'bg-slate-50 text-slate-400 border-slate-200'
                              }`}
                            >
                              <div className="text-[10px] opacity-75 font-mono">Langkah {step.num}</div>
                              <div className="font-semibold text-xs mt-0.5">{step.title}</div>
                            </div>
                          );
                        })}
                      </div>

                      {/* Slot Details Bar */}
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs bg-slate-50 p-3.5 rounded-lg border border-slate-200">
                        <div>
                          <span className="text-slate-500">Jadwal Pertemuan:</span>{' '}
                          <strong className="font-mono tabular-nums text-slate-900">
                            {activeVisit.scheduled_start
                              ? `${activeVisit.scheduled_start} - ${activeVisit.scheduled_end} WIB`
                              : 'Menunggu penetapan Posko'}
                          </strong>
                        </div>
                        <div>
                          <span className="text-slate-500">Titik Temu:</span>{' '}
                          <strong className="text-slate-900">{activeVisit.meeting_spot || '-'}</strong>
                        </div>
                        <div>
                          <span className="text-slate-500">Rincian Titipan:</span>{' '}
                          <strong className="text-amber-900">{activeVisit.titipan_description || 'Tanpa Titipan'}</strong>
                        </div>
                      </div>

                      {/* Wali Action */}
                      {activeVisit.current_status === 'SCHEDULED' && (
                        <div className="p-3.5 bg-emerald-50 border border-emerald-200 rounded-lg flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                          <div className="text-xs text-emerald-950 font-medium">
                            Jadwal telah ditetapkan oleh petugas Posko. Konfirmasi kehadiran Anda saat tiba di lokasi pondok pesantren.
                          </div>
                          <button
                            onClick={() => handleWaliConfirm(activeVisit.id)}
                            className="btn-interactive px-3.5 py-2 bg-emerald-800 hover:bg-emerald-900 text-white text-xs font-bold rounded-md shadow-xs shrink-0"
                          >
                            Konfirmasi Tiba di Lokasi Pondok
                          </button>
                        </div>
                      )}
                    </div>
                  )}

                  {/* Form Pengajuan & Galeri Logbook */}
                  <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* Form Pengajuan Baru */}
                    <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
                      <h2 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                        Ajukan Jadwal Sambang
                      </h2>
                      <form onSubmit={handleWaliRequest} className="space-y-3.5 text-xs">
                        <div>
                          <label className="block text-slate-700 font-semibold mb-1">Rencana Tanggal</label>
                          <input
                            type="date"
                            value={newVisitReq.requested_date}
                            onChange={(e) => setNewVisitReq({ ...newVisitReq, requested_date: e.target.value })}
                            className="w-full px-3 py-2 border border-slate-300 rounded-md font-mono text-slate-900 focus:outline-none focus:ring-1 focus:ring-emerald-700"
                          />
                        </div>
                        <div>
                          <label className="block text-slate-700 font-semibold mb-1">Pilihan Sesi</label>
                          <select
                            value={newVisitReq.requested_session}
                            onChange={(e) => setNewVisitReq({ ...newVisitReq, requested_session: e.target.value })}
                            className="w-full px-3 py-2 border border-slate-300 rounded-md text-slate-900 focus:outline-none focus:ring-1 focus:ring-emerald-700"
                          >
                            <option value="Sesi Pagi (08:00 - 11:30)">Sesi Pagi (08:00 - 11:30 WIB)</option>
                            <option value="Sesi Siang (13:30 - 16:30)">Sesi Siang (13:30 - 16:30 WIB)</option>
                          </select>
                        </div>
                        <div className="pt-1">
                          <label className="flex items-center space-x-2 text-slate-900 font-semibold cursor-pointer">
                            <input
                              type="checkbox"
                              checked={newVisitReq.has_titipan}
                              onChange={(e) => setNewVisitReq({ ...newVisitReq, has_titipan: e.target.checked })}
                              className="rounded text-emerald-800 focus:ring-emerald-700 h-4 w-4"
                            />
                            <span>Membawa Titipan Barang / Makanan</span>
                          </label>
                          {newVisitReq.has_titipan && (
                            <input
                              type="text"
                              placeholder="Misal: Kurma Ajwa, vitamin, kitab..."
                              value={newVisitReq.titipan_description}
                              onChange={(e) => setNewVisitReq({ ...newVisitReq, titipan_description: e.target.value })}
                              className="w-full mt-2 px-3 py-2 border border-amber-300 rounded-md bg-amber-50 text-slate-900 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-emerald-700"
                            />
                          )}
                        </div>
                        <button
                          type="submit"
                          className="btn-interactive w-full py-2.5 bg-emerald-800 hover:bg-emerald-900 text-white font-bold rounded-md shadow-xs transition-colors"
                        >
                          Kirim Pengajuan Sambang
                        </button>
                      </form>
                    </div>

                    {/* Galeri Logbook Foto Pertemuan Privat */}
                    <div className="lg:col-span-2 bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
                      <div className="flex items-center justify-between border-b border-slate-200 pb-2.5">
                        <div className="flex items-center space-x-2">
                          <Camera className="w-4 h-4 text-emerald-800" />
                          <h2 className="text-sm font-bold text-slate-900">
                            Dokumentasi Pertemuan Logbook Privat
                          </h2>
                        </div>
                        <span className="text-[11px] font-semibold text-emerald-800 bg-emerald-50 px-2 py-0.5 rounded border border-emerald-200 flex items-center space-x-1">
                          <Lock className="w-3 h-3" />
                          <span>Hanya Terlihat Oleh Wali & Posko</span>
                        </span>
                      </div>

                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        {completedVisits.map((v, idx) => (
                          <div
                            key={v.id}
                            style={{ '--index': idx } as React.CSSProperties}
                            className="animate-stagger-item border border-slate-200 rounded-lg overflow-hidden bg-slate-50 shadow-2xs"
                          >
                            <img
                              src={v.logbook_photo_url}
                              alt="Dokumentasi Pertemuan"
                              className="w-full h-44 object-cover"
                            />
                            <div className="p-3.5 text-xs space-y-1.5">
                              <div className="flex justify-between items-center">
                                <span className="font-mono font-bold text-slate-800">{v.ticket_code}</span>
                                <span className="font-mono text-slate-500 tabular-nums">{v.requested_date}</span>
                              </div>
                              <p className="text-slate-600 italic">"{v.meeting_notes}"</p>
                              <div className="text-[11px] text-emerald-900 font-semibold pt-1 flex items-center space-x-1">
                                <Check className="w-3.5 h-3.5 text-emerald-700" />
                                <span>Terverifikasi Selesai di Posko</span>
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* TAB 2: PROGRES TAHFIDZ 30 JUZ (Tremor Raw Style Strip & Grid) */}
              {waliTab === 'tahfidz' && (
                <div className="space-y-6">
                  {/* Tremor Raw Style KPI Progress Card */}
                  <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-slate-200 gap-2">
                      <div>
                        <span className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">
                          Target Pencapaian Hafalan Al-Qur'an
                        </span>
                        <h2 className="text-2xl font-bold font-mono tracking-tight text-slate-900 tabular-nums mt-0.5">
                          {santriTotalJuz} <span className="text-sm font-sans font-normal text-slate-500">dari 30 Juz</span>
                        </h2>
                      </div>
                      <div className="text-xs text-slate-600 font-mono">
                        Pencapaian: <strong className="text-emerald-900 font-bold">{((santriTotalJuz / 30) * 100).toFixed(1)}%</strong>
                      </div>
                    </div>

                    {/* Continuous Progress Bar */}
                    <div className="w-full bg-slate-100 rounded-full h-2.5 overflow-hidden border border-slate-200">
                      <div
                        className="bg-emerald-800 h-2.5 rounded-full transition-all duration-500"
                        style={{ width: `${(santriTotalJuz / 30) * 100}%` }}
                      ></div>
                    </div>

                    {/* 30 Juz Micro-Blocks Grid */}
                    <div className="grid grid-cols-6 sm:grid-cols-10 lg:grid-cols-15 gap-1.5 text-center pt-1">
                      {Array.from({ length: 30 }, (_, i) => i + 1).map((juz) => {
                        const completed = juz <= santriTotalJuz;
                        return (
                          <div
                            key={juz}
                            className={`p-2 rounded-md text-xs font-mono font-bold border transition-colors ${
                              completed
                                ? 'bg-emerald-800 text-white border-emerald-900 shadow-2xs'
                                : 'bg-slate-100 text-slate-400 border-slate-200'
                            }`}
                          >
                            <div className="text-[9px] opacity-75 font-sans font-normal">Juz</div>
                            {juz}
                          </div>
                        );
                      })}
                    </div>
                  </div>

                  {/* Riwayat Setoran Tahfidz */}
                  <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-3">
                    <h3 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                      Riwayat Setoran Hafalan & Mutun
                    </h3>
                    <div className="overflow-x-auto">
                      <table className="w-full text-xs text-left">
                        <thead className="bg-slate-100 text-slate-700 font-semibold border-b border-slate-300">
                          <tr>
                            <th className="py-2.5 px-3">Tanggal</th>
                            <th className="py-2.5 px-3">Jenis</th>
                            <th className="py-2.5 px-3">Juz</th>
                            <th className="py-2.5 px-3">Surah / Kitab</th>
                            <th className="py-2.5 px-3">Ayat / Bait</th>
                            <th className="py-2.5 px-3">Predikat</th>
                            <th className="py-2.5 px-3">Nilai</th>
                            <th className="py-2.5 px-3">Penguji</th>
                            <th className="py-2.5 px-3">Catatan Asatidz</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-200">
                          {tahfidzLogs.map((log, idx) => (
                            <tr
                              key={log.id}
                              style={{ '--index': idx } as React.CSSProperties}
                              className="animate-stagger-item hover:bg-slate-50 transition-colors"
                            >
                              <td className="py-2.5 px-3 font-mono tabular-nums">{log.session_date}</td>
                              <td className="py-2.5 px-3">
                                <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-100 text-emerald-900">
                                  {log.log_type}
                                </span>
                              </td>
                              <td className="py-2.5 px-3 font-mono tabular-nums font-bold">{log.juz_number}</td>
                              <td className="py-2.5 px-3 font-semibold text-slate-900">{log.surah_or_kitab}</td>
                              <td className="py-2.5 px-3 font-mono tabular-nums">
                                {log.ayah_start} - {log.ayah_end}
                              </td>
                              <td className="py-2.5 px-3 font-semibold text-emerald-900">{log.predicate}</td>
                              <td className="py-2.5 px-3 font-mono font-bold tabular-nums text-emerald-950">
                                {log.score}
                              </td>
                              <td className="py-2.5 px-3 text-slate-800">{log.evaluator_name}</td>
                              <td className="py-2.5 px-3 text-slate-600 italic">{log.notes || '-'}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
              )}

              {/* TAB 3: RAPOR DIGITAL SIAP CETAK */}
              {waliTab === 'rapor' && (
                <div className="bg-white border border-slate-300 rounded-xl p-6 shadow-sm space-y-6">
                  <div className="flex items-center justify-between pb-4 border-b border-slate-200">
                    <div className="flex items-center space-x-3">
                      <img
                        src={PONDOK_LOGO_URL}
                        alt="Logo Pondok Pesantren"
                        className="h-12 w-12 object-contain bg-white p-0.5 border border-slate-200 rounded"
                      />
                      <div>
                        <h2 className="text-base font-bold text-slate-900 uppercase tracking-tight">
                          Laporan Hasil Belajar Santri (Rapor Akademik)
                        </h2>
                        <div className="text-xs text-slate-600">
                          Semester Ganjil • Tahun Ajaran <span className="font-mono">2026/2027</span>
                        </div>
                      </div>
                    </div>

                    <button
                      onClick={() => window.print()}
                      className="btn-interactive flex items-center space-x-1.5 px-3.5 py-2 rounded-md bg-emerald-800 hover:bg-emerald-900 text-white font-medium text-xs shadow-xs"
                    >
                      <Printer className="w-3.5 h-3.5" />
                      <span>Cetak Rapor Digital</span>
                    </button>
                  </div>

                  {/* Profile Strip */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs bg-slate-50 p-3.5 rounded-lg border border-slate-200 font-medium">
                    <div>
                      <span className="text-slate-500">Nama Santri:</span>
                      <div className="font-bold text-slate-900">Muhammad Fatih Al-Faruq</div>
                    </div>
                    <div>
                      <span className="text-slate-500">Nomor Induk Santri:</span>
                      <div className="font-mono font-bold text-slate-900">2026-001</div>
                    </div>
                    <div>
                      <span className="text-slate-500">Kelas / Tingkat:</span>
                      <div className="text-slate-900">1 Aliyah (Aliyah Awwal)</div>
                    </div>
                    <div>
                      <span className="text-slate-500">Asrama Tinggal:</span>
                      <div className="text-slate-900">Asrama Al-Farabi</div>
                    </div>
                  </div>

                  {/* Rapor Subject Table */}
                  <div className="overflow-x-auto">
                    <table className="w-full text-xs text-left border border-slate-300">
                      <thead className="bg-emerald-950 text-white font-semibold">
                        <tr>
                          <th className="py-2.5 px-3 border border-emerald-900">Kode</th>
                          <th className="py-2.5 px-3 border border-emerald-900">Mata Pelajaran</th>
                          <th className="py-2.5 px-3 border border-emerald-900">KKM</th>
                          <th className="py-2.5 px-3 border border-emerald-900 text-center">Harian</th>
                          <th className="py-2.5 px-3 border border-emerald-900 text-center">UTS</th>
                          <th className="py-2.5 px-3 border border-emerald-900 text-center">UAS</th>
                          <th className="py-2.5 px-3 border border-emerald-900 text-center">Syafahi</th>
                          <th className="py-2.5 px-3 border border-emerald-900 text-center bg-emerald-900">Nilai Akhir</th>
                          <th className="py-2.5 px-3 border border-emerald-900 text-center">Huruf</th>
                          <th className="py-2.5 px-3 border border-emerald-900">Predikat</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-slate-200">
                        {raporRows.map((r) => (
                          <tr key={r.subject_code} className="hover:bg-slate-50">
                            <td className="py-2 px-3 font-mono font-bold border border-slate-300 text-slate-700">
                              {r.subject_code}
                            </td>
                            <td className="py-2 px-3 font-bold border border-slate-300 text-slate-900">
                              {r.subject_name}
                            </td>
                            <td className="py-2 px-3 font-mono tabular-nums border border-slate-300 text-slate-700">
                              {r.kkm_score}
                            </td>
                            <td className="py-2 px-3 font-mono tabular-nums text-center border border-slate-300">
                              {r.harian}
                            </td>
                            <td className="py-2 px-3 font-mono tabular-nums text-center border border-slate-300">
                              {r.uts}
                            </td>
                            <td className="py-2 px-3 font-mono tabular-nums text-center border border-slate-300 font-bold text-emerald-900">
                              {r.uas}
                            </td>
                            <td className="py-2 px-3 font-mono tabular-nums text-center border border-slate-300">
                              {r.syafahi}
                            </td>
                            <td className="py-2 px-3 font-mono tabular-nums font-extrabold text-center border border-slate-300 bg-emerald-50 text-emerald-950">
                              {r.final_score}
                            </td>
                            <td className="py-2 px-3 font-mono font-bold text-center border border-slate-300">
                              {r.letter_grade}
                            </td>
                            <td className="py-2 px-3 font-semibold border border-slate-300 text-slate-900">
                              {r.predicate_label}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>

                  <div className="text-[11px] text-slate-500 italic">
                    * Catatan: Predikat kelulusan dihitung berdasarkan perpaduan bobot evaluasi Harian (20%), UTS (30%), UAS (30%), dan Syafahi/Lisan (20%).
                  </div>
                </div>
              )}

              {/* TAB 4: PRESENSI & CATATAN ADAB */}
              {waliTab === 'kesantrian' && (
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                  {/* Presensi Strips */}
                  <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
                    <h3 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                      Rekapitulasi Kehadiran Santri
                    </h3>
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5 text-center">
                      <div className="bg-emerald-50 border border-emerald-200 p-3 rounded-lg">
                        <div className="text-[11px] font-bold text-emerald-800">HADIR</div>
                        <div className="text-xl font-mono font-bold text-emerald-950 mt-1">98%</div>
                      </div>
                      <div className="bg-blue-50 border border-blue-200 p-3 rounded-lg">
                        <div className="text-[11px] font-bold text-blue-800">SAKIT</div>
                        <div className="text-xl font-mono font-bold text-blue-950 mt-1">1 hari</div>
                      </div>
                      <div className="bg-amber-50 border border-amber-200 p-3 rounded-lg">
                        <div className="text-[11px] font-bold text-amber-800">IZIN RESMI</div>
                        <div className="text-xl font-mono font-bold text-amber-950 mt-1">1 hari</div>
                      </div>
                      <div className="bg-slate-100 border border-slate-200 p-3 rounded-lg">
                        <div className="text-[11px] font-bold text-slate-600">ALPHA</div>
                        <div className="text-xl font-mono font-bold text-slate-900 mt-1">0</div>
                      </div>
                    </div>

                    <div className="space-y-2 pt-2">
                      <h4 className="text-xs font-semibold text-slate-700">Presensi Harian Terakhir:</h4>
                      {attendanceRecords.map((att, idx) => (
                        <div
                          key={att.id}
                          style={{ '--index': idx } as React.CSSProperties}
                          className="animate-stagger-item p-2.5 rounded-lg bg-slate-50 border border-slate-200 flex justify-between items-center text-xs"
                        >
                          <div>
                            <span className="font-semibold text-slate-900">{att.activity_type}</span>
                            <span className="text-[10px] text-slate-500 font-mono ml-2">{att.date}</span>
                            {att.notes && <div className="text-[11px] text-slate-600 mt-0.5">{att.notes}</div>}
                          </div>
                          <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-100 text-emerald-900">
                            {att.status}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Catatan Adab & Prestasi */}
                  <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
                    <h3 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                      Catatan Adab, Prestasi & Kesehatan
                    </h3>
                    <div className="space-y-3">
                      {santriNotes.map((note, idx) => (
                        <div
                          key={note.id}
                          style={{ '--index': idx } as React.CSSProperties}
                          className="animate-stagger-item p-3.5 rounded-lg border border-slate-200 bg-slate-50 space-y-1.5 text-xs"
                        >
                          <div className="flex justify-between items-center">
                            <span
                              className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                                note.category === 'Prestasi'
                                  ? 'bg-amber-100 text-amber-900'
                                  : 'bg-emerald-100 text-emerald-900'
                              }`}
                            >
                              {note.category}
                            </span>
                            <span className="font-mono text-slate-400 text-[10px]">{note.date}</span>
                          </div>
                          <div className="font-bold text-slate-900 text-xs">{note.title}</div>
                          <p className="text-slate-600 text-xs leading-relaxed">{note.description}</p>
                          <div className="text-[10px] text-slate-400 pt-1">Pencatat: {note.recorded_by}</div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* =========================================================================
            ROLE 2: POSKO PELAYANAN SAMBANG (ADMIN)
            ========================================================================= */}
        {currentRole === 'admin' && (
          <div className="space-y-6">
            {/* Tremor Raw Style KPI Strip */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="bg-white border border-slate-200 p-4 rounded-xl shadow-xs">
                <div className="text-[11px] text-slate-500 font-medium uppercase tracking-wider">Total Santri Aktif</div>
                <div className="text-2xl font-bold font-mono text-slate-900 mt-1">2</div>
              </div>
              <div className="bg-white border border-slate-200 p-4 rounded-xl shadow-xs">
                <div className="text-[11px] text-slate-500 font-medium uppercase tracking-wider">Kunjungan Hari Ini</div>
                <div className="text-2xl font-bold font-mono text-emerald-900 mt-1">{visits.length}</div>
              </div>
              <div className="bg-white border border-slate-200 p-4 rounded-xl shadow-xs">
                <div className="text-[11px] text-slate-500 font-medium uppercase tracking-wider">Menunggu Panggilan</div>
                <div className="text-2xl font-bold font-mono text-amber-900 mt-1">
                  {visits.filter((v) => v.current_status === 'CONFIRMED_BY_WALI').length}
                </div>
              </div>
              <div className="bg-white border border-slate-200 p-4 rounded-xl shadow-xs">
                <div className="text-[11px] text-slate-500 font-medium uppercase tracking-wider">Logbook Tersimpan</div>
                <div className="text-2xl font-bold font-mono text-emerald-950 mt-1">{completedVisits.length}</div>
              </div>
            </div>

            {/* Structured Content: Mobile Action Cards (shadcn-admin) + Desktop Table */}
            <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-slate-200 gap-2">
                <div>
                  <h2 className="text-sm font-bold text-slate-900">Antrean Pelayanan Sambang & Logbook</h2>
                  <p className="text-xs text-slate-600">Tetapkan jadwal pertemuan, panggil santri ke asrama, dan selesaikan dokumentasi foto.</p>
                </div>
              </div>

              {/* MOBILE VIEW (< 768px): Structured Action Cards (cosscom/ui / shadcn-admin) */}
              <div className="block md:hidden space-y-3">
                {visits.map((v, idx) => (
                  <div
                    key={v.id}
                    style={{ '--index': idx } as React.CSSProperties}
                    className="animate-stagger-item border border-slate-200 rounded-lg p-3.5 bg-slate-50 space-y-3"
                  >
                    <div className="flex justify-between items-start gap-2">
                      <div>
                        <div className="font-mono font-bold text-sm text-emerald-950">{v.ticket_code}</div>
                        <div className="font-bold text-slate-900 text-sm mt-0.5">{v.santri_name}</div>
                        <div className="text-[11px] text-slate-600">{v.dormitory_name} • {v.room_name}</div>
                      </div>
                      <span className={`px-2 py-0.5 rounded text-[10px] font-bold border shrink-0 ${getStatusBadge(v.current_status).badgeClass}`}>
                        {getStatusBadge(v.current_status).label}
                      </span>
                    </div>

                    <div className="text-xs text-slate-700 bg-white p-2.5 rounded border border-slate-200 space-y-1">
                      <div><span className="text-slate-500">Wali:</span> <span className="font-medium text-slate-900">{v.wali_name}</span> (<span className="font-mono">{v.wali_phone}</span>)</div>
                      <div><span className="text-slate-500">Lokasi:</span> <span className="font-medium">{v.meeting_spot || 'Menunggu penetapan'}</span></div>
                      {v.has_titipan && (
                        <div className="text-amber-900 font-semibold pt-1">Titipan: {v.titipan_description}</div>
                      )}
                    </div>

                    {/* Touch-Friendly Action Buttons */}
                    <div className="space-y-2 pt-1">
                      {v.current_status === 'REQUESTED' && (
                        <button
                          onClick={() => handleAdminSchedule(v.id)}
                          className="btn-interactive w-full py-2 bg-emerald-800 hover:bg-emerald-900 text-white rounded-md font-bold text-xs shadow-xs"
                        >
                          Tetapkan Jadwal Pertemuan
                        </button>
                      )}

                      {v.current_status === 'CONFIRMED_BY_WALI' && (
                        <button
                          onClick={() => handleAdminCall(v.id)}
                          className="btn-interactive w-full py-2 bg-amber-500 hover:bg-amber-600 text-emerald-950 font-bold rounded-md text-xs shadow-xs flex items-center justify-center space-x-1.5"
                        >
                          <Bell className="w-4 h-4" />
                          <span>Panggil Santri (Bunyikan Chime Asrama)</span>
                        </button>
                      )}

                      {v.current_status === 'SANTRI_CONFIRMED' && (
                        <button
                          onClick={() => handleAdminCompleteLogbook(v.id)}
                          className="btn-interactive w-full py-2 bg-emerald-700 hover:bg-emerald-800 text-white font-bold rounded-md text-xs shadow-xs flex items-center justify-center space-x-1.5"
                        >
                          <Camera className="w-4 h-4" />
                          <span>Unggah Foto Logbook & Selesaikan</span>
                        </button>
                      )}

                      {v.current_status === 'COMPLETED' && (
                        <button
                          onClick={() => {
                            setSecurityAuditRole('admin');
                            setSecurityModalOpen(true);
                          }}
                          className="btn-interactive w-full py-2 bg-white border border-slate-300 text-slate-800 font-semibold rounded-md text-xs"
                        >
                          Buka Foto Pertemuan Terverifikasi
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              {/* DESKTOP VIEW (>= 768px): Structured Table */}
              <div className="hidden md:block overflow-x-auto">
                <table className="w-full text-xs text-left">
                  <thead className="bg-slate-100 text-slate-700 font-semibold border-b border-slate-300">
                    <tr>
                      <th className="py-2.5 px-3">Tiket & Wali</th>
                      <th className="py-2.5 px-3">Santri & Asrama</th>
                      <th className="py-2.5 px-3">Status Saat Ini</th>
                      <th className="py-2.5 px-3">Titipan Fisik</th>
                      <th className="py-2.5 px-3">Aksi Petugas Posko</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-200">
                    {visits.map((v) => (
                      <tr key={v.id} className="hover:bg-slate-50 transition-colors">
                        <td className="py-3 px-3">
                          <div className="font-mono font-bold text-emerald-950">{v.ticket_code}</div>
                          <div className="font-medium text-slate-900">{v.wali_name}</div>
                          <div className="font-mono text-slate-500 text-[11px]">{v.wali_phone}</div>
                        </td>
                        <td className="py-3 px-3">
                          <div className="font-bold text-slate-900">{v.santri_name}</div>
                          <div className="text-slate-600 text-[11px]">
                            {v.dormitory_name} • {v.room_name}
                          </div>
                        </td>
                        <td className="py-3 px-3">
                          <span
                            className={`px-2 py-0.5 rounded text-[10px] font-bold border ${getStatusBadge(v.current_status).badgeClass}`}
                          >
                            {getStatusBadge(v.current_status).label}
                          </span>
                        </td>
                        <td className="py-3 px-3">
                          {v.has_titipan ? (
                            <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-50 text-amber-950 border border-amber-300">
                              ADA TITIPAN
                            </span>
                          ) : (
                            <span className="text-slate-400 text-[10px]">-</span>
                          )}
                        </td>
                        <td className="py-3 px-3 space-x-2">
                          {v.current_status === 'REQUESTED' && (
                            <button
                              onClick={() => handleAdminSchedule(v.id)}
                              className="btn-interactive px-3 py-1.5 bg-emerald-800 hover:bg-emerald-900 text-white rounded-md font-medium text-xs shadow-xs"
                            >
                              Tetapkan Jadwal
                            </button>
                          )}

                          {v.current_status === 'CONFIRMED_BY_WALI' && (
                            <button
                              onClick={() => handleAdminCall(v.id)}
                              className="btn-interactive px-3 py-1.5 bg-amber-500 hover:bg-amber-600 text-emerald-950 font-bold rounded-md text-xs shadow-xs inline-flex items-center space-x-1"
                            >
                              <Bell className="w-3.5 h-3.5" />
                              <span>Panggil Santri (Chime)</span>
                            </button>
                          )}

                          {v.current_status === 'SANTRI_CONFIRMED' && (
                            <button
                              onClick={() => handleAdminCompleteLogbook(v.id)}
                              className="btn-interactive px-3 py-1.5 bg-emerald-700 hover:bg-emerald-800 text-white font-bold rounded-md text-xs shadow-xs inline-flex items-center space-x-1"
                            >
                              <Camera className="w-3.5 h-3.5" />
                              <span>Selesaikan & Foto WebP</span>
                            </button>
                          )}

                          {v.current_status === 'COMPLETED' && (
                            <button
                              onClick={() => {
                                setSecurityAuditRole('admin');
                                setSecurityModalOpen(true);
                              }}
                              className="btn-interactive px-3 py-1.5 bg-white hover:bg-slate-100 text-slate-800 border border-slate-300 rounded-md text-xs"
                            >
                              Buka Foto
                            </button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}

        {/* =========================================================================
            ROLE 3: MONITOR ASRAMA (TV SCREEN WITH AMPLE TOP PADDING)
            ========================================================================= */}
        {currentRole === 'musyrif' && (
          <div className="bg-emerald-950 text-white rounded-xl pt-6 sm:pt-8 pb-8 px-5 sm:px-8 shadow-xl border border-emerald-900 space-y-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-emerald-900 gap-4">
              <div className="flex items-center space-x-3.5">
                <img
                  src={PONDOK_LOGO_URL}
                  alt="Logo Pondok Pesantren"
                  className="h-12 w-12 object-contain bg-white p-0.5 rounded-md border border-emerald-800"
                />
                <div>
                  <h1 className="text-xl sm:text-2xl font-bold tracking-tight text-white">
                    Monitor Panggilan: Asrama Al-Farabi
                  </h1>
                  <div className="text-xs text-emerald-300 mt-0.5">
                    Zona Asrama Putra • Musyrif: <span className="font-semibold text-white">Ust. Zulkifli, S.Pd.I</span>
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-3">
                {/* Secondary Security Guard Test Trigger */}
                <button
                  onClick={() => {
                    setSecurityAuditRole('musyrif');
                    setSecurityModalOpen(true);
                  }}
                  className="btn-interactive text-xs border border-emerald-700 hover:bg-emerald-900/80 text-emerald-200 px-3 py-1.5 rounded-md flex items-center space-x-1.5"
                >
                  <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
                  <span>Uji Akses Privasi Foto</span>
                </button>

                <div className="flex items-center space-x-2 text-xs bg-emerald-900/80 px-3 py-1.5 rounded-full border border-emerald-800">
                  <span className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-ping"></span>
                  <span className="font-mono text-emerald-200">LIVE SSE AKTIF</span>
                </div>
              </div>
            </div>

            {/* Panggilan Santri */}
            <div className="space-y-4">
              <div className="text-sm font-bold uppercase tracking-wider text-emerald-300">
                Panggilan Menunggu Tindakan Musyrif
              </div>

              {visits.filter((v) => v.current_status === 'SANTRI_CALLED').length === 0 ? (
                <div className="p-8 text-center bg-emerald-900/40 rounded-xl border border-emerald-900 text-emerald-300 text-xs">
                  Tidak ada panggilan santri yang sedang menunggu saat ini. Layar akan otomatis bersuara chime saat ada panggilan masuk dari Posko Pelayanan Sambang.
                </div>
              ) : (
                visits
                  .filter((v) => v.current_status === 'SANTRI_CALLED')
                  .map((v) => (
                    <div
                      key={v.id}
                      className="p-5 rounded-xl bg-emerald-900/90 border-2 border-amber-400 shadow-lg flex flex-col md:flex-row md:items-center justify-between gap-4 animate-stagger-item"
                    >
                      <div className="space-y-1">
                        <div className="flex items-center space-x-2">
                          <span className="px-2.5 py-0.5 rounded text-[11px] font-bold bg-amber-400 text-emerald-950 font-mono">
                            DIPANGGIL KE POSKO
                          </span>
                          <span className="font-mono text-xs text-emerald-200">{v.ticket_code}</span>
                        </div>
                        <h2 className="text-xl sm:text-2xl font-bold text-white tracking-tight">{v.santri_name}</h2>
                        <div className="text-xs text-emerald-200">
                          {v.room_name} • Wali: <strong>{v.wali_name}</strong> • Titik Temu:{' '}
                          <span className="underline font-semibold">{v.meeting_spot}</span>
                        </div>

                        {v.has_titipan && (
                          <div className="p-2.5 bg-amber-500/20 border border-amber-400/50 rounded-md text-xs text-amber-200 mt-2 font-medium">
                            <strong>TITIPAN DARI WALI:</strong> {v.titipan_description}
                          </div>
                        )}
                      </div>

                      <button
                        onClick={() => handleMusyrifConfirm(v.id)}
                        className="btn-interactive px-5 py-3 bg-amber-400 hover:bg-amber-300 text-emerald-950 font-extrabold text-sm rounded-lg shadow-md flex items-center justify-center space-x-2 shrink-0"
                      >
                        <Check className="w-4 h-4 stroke-[3]" />
                        <span>Konfirmasi Santri & Titipan</span>
                      </button>
                    </div>
                  ))
              )}
            </div>
          </div>
        )}

        {/* =========================================================================
            ROLE 4: PORTAL DEWAN ASATIDZ (HIGH-SPEED FORM ANTI-TRUNCATE)
            ========================================================================= */}
        {currentRole === 'asatidz' && (
          <div className="space-y-6">
            <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <span className="text-xs font-semibold uppercase text-emerald-800">
                  Workspace Pengajar & Penguji
                </span>
                <h1 className="text-base font-bold text-slate-900 tracking-tight">Ust. Ahmad Fauzi, Lc., M.Ag.</h1>
                <div className="text-xs text-slate-600">
                  Semester Ganjil 2026/2027 • Evaluasi Berjalan
                </div>
              </div>

              {/* Segmented Control Bar */}
              <div className="bg-slate-100 p-1 rounded-lg flex items-center space-x-1 shrink-0 border border-slate-200">
                <button
                  onClick={() => setAsatidzPanel('tahfidz')}
                  className={`btn-interactive px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
                    asatidzPanel === 'tahfidz'
                      ? 'bg-emerald-950 text-white font-bold shadow-xs'
                      : 'text-slate-700 hover:text-slate-900'
                  }`}
                >
                  Setoran Tahfidz
                </button>
                <button
                  onClick={() => setAsatidzPanel('ujian')}
                  className={`btn-interactive px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
                    asatidzPanel === 'ujian'
                      ? 'bg-emerald-950 text-white font-bold shadow-xs'
                      : 'text-slate-700 hover:text-slate-900'
                  }`}
                >
                  Nilai Ujian & Rapor
                </button>
              </div>
            </div>

            {/* PANEL TAHFIDZ ENTRY (Responsive Grid Anti-Truncate: grid-cols-1 sm:grid-cols-3) */}
            {asatidzPanel === 'tahfidz' && (
              <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-view-enter">
                <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
                  <h2 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                    Input Setoran Hafalan Baru
                  </h2>
                  <form onSubmit={handleAddTahfidz} className="space-y-3.5 text-xs">
                    <div>
                      <label className="block text-slate-700 font-semibold mb-1">Nama Santri</label>
                      <input
                        type="text"
                        disabled
                        value="Muhammad Fatih Al-Faruq (NIS: 2026-001)"
                        className="w-full px-3 py-2 border border-slate-200 rounded-md bg-slate-50 text-slate-700 font-medium"
                      />
                    </div>

                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <div>
                        <label className="block text-slate-700 font-semibold mb-1">Target Juz</label>
                        <input
                          type="number"
                          value={tahfidzForm.juz_number}
                          onChange={(e) => setTahfidzForm({ ...tahfidzForm, juz_number: Number(e.target.value) })}
                          className="w-full px-3 py-2 border border-slate-300 rounded-md font-mono text-slate-900"
                        />
                      </div>
                      <div>
                        <label className="block text-slate-700 font-semibold mb-1">Jenis Setoran</label>
                        <select
                          value={tahfidzForm.log_type}
                          onChange={(e) => setTahfidzForm({ ...tahfidzForm, log_type: e.target.value })}
                          className="w-full px-3 py-2 border border-slate-300 rounded-md text-slate-900 font-medium"
                        >
                          <option value="ZIYADAH">ZIYADAH (Hafalan Baru)</option>
                          <option value="MURAJA_AH">MURAJA'AH (Ulang)</option>
                        </select>
                      </div>
                    </div>

                    <div>
                      <label className="block text-slate-700 font-semibold mb-1">Surah / Kitab</label>
                      <input
                        type="text"
                        value={tahfidzForm.surah_or_kitab}
                        onChange={(e) => setTahfidzForm({ ...tahfidzForm, surah_or_kitab: e.target.value })}
                        className="w-full px-3 py-2 border border-slate-300 rounded-md text-slate-900 font-medium"
                      />
                    </div>

                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="block text-slate-700 font-semibold mb-1">Ayat Mulai</label>
                        <input
                          type="number"
                          value={tahfidzForm.ayah_start}
                          onChange={(e) => setTahfidzForm({ ...tahfidzForm, ayah_start: Number(e.target.value) })}
                          className="w-full px-3 py-2 border border-slate-300 rounded-md font-mono text-slate-900"
                        />
                      </div>
                      <div>
                        <label className="block text-slate-700 font-semibold mb-1">Ayat Akhir</label>
                        <input
                          type="number"
                          value={tahfidzForm.ayah_end}
                          onChange={(e) => setTahfidzForm({ ...tahfidzForm, ayah_end: Number(e.target.value) })}
                          className="w-full px-3 py-2 border border-slate-300 rounded-md font-mono text-slate-900"
                        />
                      </div>
                    </div>

                    {/* Anti-Truncate Dropdowns */}
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <div>
                        <label className="block text-slate-700 font-semibold mb-1">Predikat</label>
                        <select
                          value={tahfidzForm.predicate}
                          onChange={(e) => setTahfidzForm({ ...tahfidzForm, predicate: e.target.value })}
                          className="w-full px-3 py-2 border border-slate-300 rounded-md text-slate-900 font-medium"
                        >
                          <option value="Mumtaz (95)">Mumtaz (90 - 100)</option>
                          <option value="Jayyid Jiddan (85)">Jayyid Jiddan (80 - 89)</option>
                          <option value="Jayyid (75)">Jayyid (70 - 79)</option>
                          <option value="Maqbul (65)">Maqbul (60 - 69)</option>
                        </select>
                      </div>
                      <div>
                        <label className="block text-slate-700 font-semibold mb-1">Skor Angka</label>
                        <input
                          type="number"
                          value={tahfidzForm.score}
                          onChange={(e) => setTahfidzForm({ ...tahfidzForm, score: Number(e.target.value) })}
                          className="w-full px-3 py-2 border border-slate-300 rounded-md font-mono text-slate-900 font-bold"
                        />
                      </div>
                    </div>

                    <label className="flex items-center space-x-2 text-slate-800 font-medium cursor-pointer pt-1">
                      <input
                        type="checkbox"
                        checked={tahfidzForm.update_juz}
                        onChange={(e) => setTahfidzForm({ ...tahfidzForm, update_juz: e.target.checked })}
                        className="rounded text-emerald-800 focus:ring-emerald-700 h-4 w-4"
                      />
                      <span>Otomatis Naikkan Total Capaian Juz Santri</span>
                    </label>

                    <button
                      type="submit"
                      className="btn-interactive w-full py-2.5 bg-emerald-800 hover:bg-emerald-900 text-white font-bold rounded-md shadow-xs"
                    >
                      Simpan Catatan Setoran
                    </button>
                  </form>
                </div>

                {/* Staggered Recent List */}
                <div className="lg:col-span-2 bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-3">
                  <h3 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                    Setoran Santri Masuk Terkini
                  </h3>
                  <div className="space-y-2.5">
                    {tahfidzLogs.map((l, idx) => (
                      <div
                        key={l.id}
                        style={{ '--index': idx } as React.CSSProperties}
                        className="animate-stagger-item p-3.5 bg-slate-50 border border-slate-200 rounded-lg flex justify-between items-center text-xs"
                      >
                        <div>
                          <div className="font-bold text-slate-900">
                            Juz {l.juz_number} • {l.surah_or_kitab} (Ayat {l.ayah_start}-{l.ayah_end})
                          </div>
                          <div className="text-[11px] text-slate-500 font-mono mt-0.5">
                            {l.session_date} • {l.notes || 'Lancar'}
                          </div>
                        </div>
                        <div className="text-right shrink-0">
                          <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-100 text-emerald-900">
                            {l.predicate}
                          </span>
                          <div className="font-mono font-bold text-emerald-950 mt-1">{l.score}</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* PANEL UJIAN & NILAI RAPOR */}
            {asatidzPanel === 'ujian' && (
              <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4 max-w-xl animate-view-enter">
                <h2 className="text-sm font-bold text-slate-900 border-b border-slate-200 pb-2">
                  Input Nilai Ujian Semester (Memperbarui Rapor Digital Secara Riil)
                </h2>
                <form onSubmit={handleUpdateGrade} className="space-y-3.5 text-xs">
                  <div>
                    <label className="block text-slate-700 font-semibold mb-1">Mata Pelajaran</label>
                    <select
                      value={gradeForm.subject_code}
                      onChange={(e) => setGradeForm({ ...gradeForm, subject_code: e.target.value })}
                      className="w-full px-3 py-2 border border-slate-300 rounded-md text-slate-900 font-medium"
                    >
                      <option value="MP-01">MP-01: Tahfidz Al-Qur`an 30 Juz</option>
                      <option value="MP-02">MP-02: Fiqih Ibadah & Muamalah (Fathul Qorib)</option>
                      <option value="MP-03">MP-03: Bahasa Arab & Nahwu Sharaf (Al-Jurumiyah)</option>
                      <option value="MP-04">MP-04: Hadits Nabawi (Arbain An-Nawawiyah)</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-slate-700 font-semibold mb-1">Komponen Penilaian</label>
                    <input
                      type="text"
                      disabled
                      value="UAS (Ujian Akhir Semester Ganjil)"
                      className="w-full px-3 py-2 border border-slate-200 rounded-md bg-slate-50 text-slate-700 font-medium"
                    />
                  </div>
                  <div>
                    <label className="block text-slate-700 font-semibold mb-1">Skor Nilai (0 - 100)</label>
                    <input
                      type="number"
                      step="0.5"
                      value={gradeForm.score}
                      onChange={(e) => setGradeForm({ ...gradeForm, score: Number(e.target.value) })}
                      className="w-full px-3 py-2 border border-slate-300 rounded-md font-mono text-slate-900 font-bold"
                    />
                  </div>
                  <button
                    type="submit"
                    className="btn-interactive w-full py-2.5 bg-emerald-800 hover:bg-emerald-900 text-white font-bold rounded-md shadow-xs"
                  >
                    Simpan Nilai & Sinkronkan Rapor Wali
                  </button>
                </form>
              </div>
            )}
          </div>
        )}

        {/* =========================================================================
            ROLE 5: PELACAKAN PUBLIK (PAPAN ALUR PIHAK KETIGA)
            ========================================================================= */}
        {currentRole === 'public' && (
          <div className="space-y-6">
            <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <h1 className="text-base font-bold text-slate-900 tracking-tight">
                  Papan Pantau Transparansi Alur Sambang
                </h1>
                <p className="text-xs text-slate-600">
                  Pelacakan alur bagi tamu dan keluarga santri. Data identitas disamarkan dan dokumen foto bersifat privat.
                </p>
              </div>

              <div className="flex items-center space-x-2">
                {/* Secondary Security Trigger */}
                <button
                  onClick={() => {
                    setSecurityAuditRole('anon');
                    setSecurityModalOpen(true);
                  }}
                  className="btn-interactive text-xs border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-md flex items-center space-x-1.5 shrink-0"
                >
                  <ShieldCheck className="w-3.5 h-3.5 text-slate-600" />
                  <span>Audit Privasi Foto</span>
                </button>

                <input
                  type="text"
                  placeholder="Cari Kode Tiket (TKT-...)"
                  value={publicTicketSearch}
                  onChange={(e) => setPublicTicketSearch(e.target.value)}
                  className="px-3 py-1.5 border border-slate-300 rounded-md text-xs font-mono text-slate-900 focus:outline-none focus:ring-1 focus:ring-emerald-700"
                />
              </div>
            </div>

            {/* Stepper Table */}
            <div className="bg-white border border-slate-200 rounded-xl p-5 shadow-xs space-y-4">
              <div className="overflow-x-auto">
                <table className="w-full text-xs text-left">
                  <thead className="bg-slate-100 text-slate-700 font-semibold border-b border-slate-300">
                    <tr>
                      <th className="py-2.5 px-3">Kode Tiket</th>
                      <th className="py-2.5 px-3">Nama Santri (Disamarkan)</th>
                      <th className="py-2.5 px-3">Asrama</th>
                      <th className="py-2.5 px-3">Tahap Pelaksanaan</th>
                      <th className="py-2.5 px-3">Titik Temu</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-200">
                    {visits
                      .filter(
                        (v) =>
                          !publicTicketSearch ||
                          v.ticket_code.toLowerCase().includes(publicTicketSearch.toLowerCase())
                      )
                      .map((v) => (
                        <tr key={v.id} className="hover:bg-slate-50 transition-colors">
                          <td className="py-3 px-3 font-mono font-bold text-emerald-950">{v.ticket_code}</td>
                          <td className="py-3 px-3 font-mono text-slate-800">{v.masked_name}</td>
                          <td className="py-3 px-3 text-slate-600">{v.dormitory_name}</td>
                          <td className="py-3 px-3">
                            <span className={`px-2 py-0.5 rounded text-[10px] font-bold border ${getStatusBadge(v.current_status).badgeClass}`}>
                              {getStatusBadge(v.current_status).label}
                            </span>
                          </td>
                          <td className="py-3 px-3 font-medium text-slate-900">{v.meeting_spot || 'Menunggu jadwal'}</td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}
      </main>

      {/* =========================================================================
          INTERACTIVE PRIVACY AUDIT MODAL (RBAC 403 Forbidden vs 200 OK)
          ========================================================================= */}
      {securityModalOpen && (
        <div className="fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center p-4 animate-view-enter">
          <div className="bg-white rounded-xl border border-slate-300 max-w-lg w-full p-6 shadow-2xl space-y-5">
            <div className="flex items-center justify-between border-b border-slate-200 pb-3">
              <div className="flex items-center space-x-2">
                <ShieldCheck className="w-5 h-5 text-emerald-900" />
                <h3 className="font-bold text-sm text-slate-900">
                  Uji Otorisasi Privasi Foto Logbook Pertemuan
                </h3>
              </div>
              <button
                onClick={() => setSecurityModalOpen(false)}
                className="btn-interactive text-slate-400 hover:text-slate-600 text-sm font-mono p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="text-xs text-slate-600 space-y-1.5 leading-relaxed">
              <p>
                Sesuai prinsip isolasi data kepengasuhan santri, berkas foto pertemuan tatap muka di posko dilindungi dan <strong>hanya dapat diakses oleh Wali Santri yang bersangkutan serta Admin Posko</strong>.
              </p>
              <p className="font-semibold text-slate-800">
                Pilih peran pengujian untuk membuka endpoint <code className="font-mono bg-slate-100 px-1 py-0.5 rounded border border-slate-200 text-slate-800">/api/visits/:id/logbook/photo</code>:
              </p>
            </div>

            {/* Role Test Selector */}
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 text-xs">
              {[
                { id: 'wali', label: 'Wali Pemilik' },
                { id: 'admin', label: 'Admin Posko' },
                { id: 'musyrif', label: 'Musyrif Asrama' },
                { id: 'asatidz', label: 'Dewan Asatidz' },
                { id: 'other_wali', label: 'Wali Santri Lain' },
                { id: 'anon', label: 'Tamu Publik' },
              ].map((item) => (
                <button
                  key={item.id}
                  onClick={() => setSecurityAuditRole(item.id as any)}
                  className={`btn-interactive px-3 py-2 rounded-lg border text-left font-medium transition-colors ${
                    securityAuditRole === item.id
                      ? 'bg-emerald-950 text-white border-emerald-950 font-bold shadow-2xs'
                      : 'bg-slate-50 text-slate-700 border-slate-300 hover:bg-slate-100'
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </div>

            {/* Audit Result Display */}
            <div className="p-4 rounded-lg border text-xs space-y-2">
              {securityAuditRole === 'wali' || securityAuditRole === 'admin' ? (
                <div className="bg-emerald-50 border border-emerald-200 text-emerald-950 p-3 rounded-lg space-y-2">
                  <div className="flex items-center space-x-2 font-bold text-emerald-900">
                    <CheckCircle2 className="w-4 h-4 text-emerald-700" />
                    <span>HTTP 200 OK — AKSES DIIZINKAN</span>
                  </div>
                  <p className="text-[11px] text-emerald-800">
                    Identitas Anda terverifikasi sebagai {securityAuditRole === 'wali' ? 'Wali Santri Sah Pemilik Tiket' : 'Admin Posko Pelayanan'}. Dokumen foto pertemuan berhasil didekripsi dan dimuat.
                  </p>
                  <img
                    src="https://images.unsplash.com/photo-1544717305-2782549b5136?auto=format&fit=crop&w=600&q=80"
                    alt="Foto Validated"
                    className="w-full h-36 object-cover rounded-md border border-emerald-300 mt-2"
                  />
                </div>
              ) : securityAuditRole === 'anon' ? (
                <div className="bg-red-50 border border-red-300 text-red-950 p-3 rounded-lg space-y-1">
                  <div className="flex items-center space-x-2 font-bold text-red-800">
                    <Lock className="w-4 h-4 text-red-700" />
                    <span>HTTP 401 UNAUTHORIZED — SESI LOGIN DIPERLUKAN</span>
                  </div>
                  <p className="text-[11px] text-red-800">
                    Akses ditolak: Pihak ketiga tanpa otentikasi sesi aktif dilarang mengakses arsip dokumentasi privat keluarga santri.
                  </p>
                </div>
              ) : (
                <div className="bg-red-50 border border-red-300 text-red-950 p-3 rounded-lg space-y-1">
                  <div className="flex items-center space-x-2 font-bold text-red-800">
                    <AlertTriangle className="w-4 h-4 text-red-700" />
                    <span>HTTP 403 FORBIDDEN — HAK AKSES DITOLAK</span>
                  </div>
                  <p className="text-[11px] text-red-800">
                    Peran Anda sebagai <strong>{securityAuditRole.toUpperCase()}</strong> tidak berwenang melihat foto ini. Dokumentasi logbook pertemuan keluarga santri hanya disajikan kepada Wali yang bersangkutan dan Admin Posko.
                  </p>
                </div>
              )}
            </div>

            <div className="flex justify-end">
              <button
                onClick={() => setSecurityModalOpen(false)}
                className="btn-interactive px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-800 font-semibold text-xs rounded-md"
              >
                Tutup Pengujian
              </button>
            </div>
          </div>
        </div>
      )}

      {/* FOOTER */}
      <footer className="bg-slate-100 border-t border-slate-200 py-3 text-center text-xs text-slate-500 font-mono print:hidden">
        SiPesantren &copy; 2026 • Sistem Terpadu Kepengasuhan, Tahfidz & Akademik Pondok Pesantren
      </footer>
    </div>
  );
}
