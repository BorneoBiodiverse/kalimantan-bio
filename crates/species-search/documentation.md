# Catatan Perkembangan & Panduan Modul 1 (Intelligent Species Search)

Dokumen ini mencatat seluruh arsitektur yang telah dirancang untuk Modul 1, peran masing-masing anggota tim, penjelasan kode, serta panduan pengujian (*testing guide*).

---

## 1. Ringkasan & Status Proyek Saat Ini

- **Crate**: `crates/species-search`
- **Cabang Git Aktif**: `module1/feat/intelligent-species-search`
- **Status Saat Ini**: **Interface Scaffold Selesai (100% Kompilasi Lulus)**
- Seluruh file modul telah dibuat dan didaftarkan di root [`src/lib.rs`](src/lib.rs).
- Setiap fungsi publik dan tipe data telah dilengkapi dokumentasi Rustdoc lengkap dengan tag `rust,ignore` sehingga `cargo test` dan `cargo doc` tetap hijau tanpa error.

---

## 2. Struktur File & Pembagian Tugas (5 Orang)

Pekerjaan Modul 1 dipecah menjadi 5 bagian terpisah untuk mencegah *merge conflict* di Git saat pengerjaan paralel:

| File | Tahap | Anggota Bertanggung Jawab | Tugas Utama |
|---|---|---|---|
| [`src/types.rs`](src/types.rs) | Fondasi | **Lead** | Mendefinisikan kontrak data bersama: struct `QueryFilters` dan konfigurasi bobot `ScoringWeights`. |
| [`src/parser.rs`](src/parser.rs) | Tahap 1 | **Anggota 1** | Mengubah token string menjadi struct `QueryFilters` (`extract_filters`). |
| [`src/filter.rs`](src/filter.rs) | Tahap 2 | **Anggota 2** | Menyaring kumpulan spesies dengan predikat boolean (`matches_*` dan `filter_species`). |
| [`src/scoring.rs`](src/scoring.rs) | Tahap 3 | **Anggota 3** | Menghitung skor kecocokan teks, taksonomi, dan konservasi (`score_*` dan `score_species`). |
| [`src/recommendation.rs`](src/recommendation.rs) | Tahap 4 | **Anggota 4** | Menghasilkan saran pencarian lanjutan terkait (`extract_common_attributes`, `rank_related_queries`). |
| [`src/lib.rs`](src/lib.rs) | Tahap 5 | **Lead** | Merangkai seluruh alur menjadi fungsi publik `search` dan `recommend_related_queries`, serta pengujian integrasi. |

---

## 3. Penjelasan Teknis Tiap Komponen

### A. Kontrak Data Bersama: `src/types.rs`
- **`QueryFilters`**: Wadah untuk menyimpan kriteria filter. Menggunakan `Option<T>` (`Some` / `None`) untuk field opsional (seperti status IUCN, taksonomi, perlindungan P.106) dan `Vec<String>` untuk kata kunci teks bebas.
- **`ScoringWeights`**: Menentukan persentase pengaruh tiap kriteria (50% kecocokan teks, 30% taksonomi, 20% status konservasi).

### B. Tahap 1: `src/parser.rs` (Anggota 1)
- Fungsi utama: `pub(crate) fn extract_filters(tokens: &[String]) -> QueryFilters`
- Mengenali token khusus:
  - Kode IUCN: `"cr"`, `"en"`, `"vu"`, `"nt"`, `"lc"`, `"dd"`, atau awalan `iucn:...`.
  - Tipe spesies: `"endemik"` / `"endemic"`, `"asli"` / `"native"`.
  - Status perlindungan: `"dilindungi"`.
  - Kata lainnya otomatis dimasukkan ke `free_text`.

### C. Tahap 2: `src/filter.rs` (Anggota 2)
- Fungsi utama: `pub(crate) fn filter_species<'a>(species_list: &'a [Species], filters: &QueryFilters) -> Vec<&'a Species>`
- Menggunakan prinsip *Pure Predicates* dan *Higher-Order Functions* (`.iter().filter(...)`).
- Mengembalikan vektor pointer referensi `&'a Species` sehingga hemat memori (tanpa *cloning* data).

### D. Tahap 3: `src/scoring.rs` (Anggota 3)
- Fungsi utama: `pub(crate) fn score_species(species: &Species, filters: &QueryFilters, weights: &ScoringWeights) -> f64`
- Memberi nilai kecocokan desimal (0.0 sampai 1.0) untuk tiap spesies yang lolos filter, memprioritaskan kecocokan pada nama ilmiah dan nama lokal dibandingkan teks deskripsi panjang.

### E. Tahap 4: `src/recommendation.rs` (Anggota 4)
- Mengambil pola umum/dominan dari hasil pencarian (misal jika banyak hasil berstatus IUCN `"CR"`), lalu meracik saran query baru untuk pengguna.

---

## 4. Panduan Pengujian (Testing Guide)

Pengujian di Modul 1 dibagi menjadi dua level:

### Level 1: Pengujian Unit Lokal (Untuk Anggota 1, 2, 3, dan 4)
Setiap anggota membuat blok pengujian di bagian paling bawah filenya masing-masing menggunakan anotasi `#[cfg(test)]`.

**Contoh Template Unit Test untuk Anggota 1 (`parser.rs`):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_iucn_token() {
        let tokens = vec!["bekantan".to_string(), "cr".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.iucn, Some("CR".to_string()));
        assert_eq!(filters.free_text, vec!["bekantan".to_string()]);
    }
}
```

**Contoh Template Unit Test untuk Anggota 2 (`filter.rs`):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::{Species, Taxonomy};

    // Buat data dummy spesies sederhana untuk uji coba
    fn dummy_species(name: &str, iucn: &str) -> Species {
        // ... inisialisasi species ...
    }

    #[test]
    fn test_filter_by_iucn() {
        let species_a = dummy_species("Orangutan", "CR");
        let species_b = dummy_species("Bekantan", "EN");
        let list = vec![species_a, species_b];

        let mut filters = QueryFilters::default();
        filters.iucn = Some("CR".to_string());

        let result = filter_species(&list, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scientific_name, "Orangutan");
    }
}
```

---

### Level 2: Pengujian Integrasi Alur Utama (Tugas Lead / Tahap 5)
Ketika seluruh anggota telah menyelesaikan kodenya, Lead akan menguji fungsi `search` menggunakan dataset riil di:
[`crates/shared/fixtures/species.json`](../shared/fixtures/species.json).

**Skenario Pengujian yang Wajib Lulus:**
1. **Pencarian Nama Umum**: Query `"orangutan"` harus mengembalikan spesies Orangutan Kalimantan di peringkat nomor 1.
2. **Pencarian Status Konservasi**: Query `"iucn:CR"` hanya mengembalikan spesies dengan status *Critically Endangered*.
3. **Pencarian Kombinasi Multi-Atribut**: Query `"mamalia dilindungi"` menyaring hanya kelas Mamalia yang berstatus dilindungi.
4. **Pencarian Kosong / Tidak Cocok**: Query kata acak (misal `"xyz123"`) harus mengembalikan vektor kosong secara aman tanpa terjadi *panic* atau *crash*.

---

## 5. Perintah Verifikasi Repositori

Pastikan terminal berada di root repositori dan path Cargo sudah aktif, lalu jalankan:

```powershell
# 1. Cek kompilasi dan validitas tipe data
cargo check -p species-search

# 2. Jalankan semua unit test
cargo test -p species-search

# 3. Validasi kelengkapan dokumentasi Rustdoc (tidak boleh ada warning)
cargo doc -p species-search --no-deps
```
Semua perintah di atas harus menghasilkan status **lulus (exit code 0)** sebelum pekerjaan di-commit dan dibuatkan Pull Request.
