//! Tahap 1 — Validasi dan Normalisasi Data.
//!
//! Modul ini menyediakan fungsi-fungsi murni (pure functions) untuk:
//! - Normalisasi representasi string dan atribut spesies.
//! - Validasi dataset spesies sebelum pemrosesan.
//! - Validasi parameter kueri eksplorasi relasi.
//! - Validasi bobot scoring relasi.
//! - Pencarian referensi spesies pusat dari snapshot dataset.

#![allow(dead_code)]

/// Membersihkan spasi berlebih pada teks (trim dan merapatkan spasi ganda)
/// tanpa mengubah kapitalisasi huruf.
pub(crate) fn sanitize_name(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Menormalisasi label: merapikan spasi dan mengubah ke huruf kecil.
pub(crate) fn normalize_label(s: &str) -> String {
    sanitize_name(s).to_lowercase()
}
