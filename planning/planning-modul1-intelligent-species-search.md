# Planning Function — Modul 1: Intelligent Species Search

**Mata Kuliah:** Pemrograman Fungsional
**Bahasa:** Rust
**Cakupan:** Natural-language species search, multi-attribute filtering, relevance ranking, related-query recommendation

**Repository**: Part of KalimantanBio monorepo workspace (`crates/species-search`)  
**API Integration**: Production KalimantanBio API (PostgreSQL via shared library)

> **PENTING**: Modul ini menggunakan **Shared Library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk arsitektur lengkap.

---

## 1. Tujuan Modul

Modul ini membangun sistem pencarian spesies yang memungkinkan pengguna mencari data biodiversitas menggunakan query bahasa natural maupun filter atribut spesifik, lalu menampilkan hasil yang diurutkan berdasarkan relevansi.

Modul ini bertanggung jawab untuk:

* Mem-parsing query bahasa natural menjadi filter terstruktur.
* Melakukan filtering spesies berdasarkan atribut yang tersedia (taksonomi, status konservasi, tipe spesies, status verifikasi).
* Menghitung skor relevansi dan mengurutkan (ranking) hasil pencarian.
* Menghasilkan rekomendasi query lanjutan berdasarkan hasil pencarian saat ini.

### Batasan Modul

Modul ini **tidak bertanggung jawab** terhadap:

* Menyimpan atau mengubah data spesies (tanggung jawab shared library dan database).
* Visualisasi jaringan relasi antar-spesies (itu Modul 2).
* Eksplorasi hierarki taksonomi mendalam (itu Modul 3).
* Perbandingan antar-spesies secara detail (itu Modul 4).
* Pencarian berbasis publikasi ilmiah (itu Modul 5).

---

## 2. Shared Library Integration

Modul ini menggunakan **shared library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum yang digunakan bersama dengan modul lain.

### 2.1 Shared Types Used

```rust
use kalimantanbio_shared::core::{Species, Taxonomy};
```

**Species** dan **Taxonomy** adalah tipe standar yang digunakan oleh semua modul. Definisi lengkap ada di shared library.

- `Species` - Struktur data lengkap spesies dari production database
- `Taxonomy` - Struktur hierarki taksonomi (kingdom → genus)

### 2.2 Shared Functions Used

#### From `text` module:
```rust
use kalimantanbio_shared::text::{normalize_text, tokenize};
```

- `normalize_text(input: &str) -> String` - Normalisasi teks (lowercase, trim, remove extra spaces)
- `tokenize(text: &str) -> Vec<String>` - Tokenisasi query menjadi kata-kata

**Digunakan di**: Tahap 1 (Parsing & Query Understanding)

#### From `taxonomy` module:
```rust
use kalimantanbio_shared::taxonomy::calculate_taxonomy_similarity;
```

- `calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64` - Menghitung skor kesamaan taksonomi (0.0-1.0)

**Digunakan di**: Tahap 3 (Relevance Scoring)

#### From `scoring` module:
```rust
use kalimantanbio_shared::scoring::{combine_weighted_scores, rank_by_score};
```

- `combine_weighted_scores(scores: &[(f64, f64)]) -> f64` - Menggabungkan skor dengan bobot
- `rank_by_score<T>(items: Vec<(T, f64)>) -> Vec<(T, f64)>` - Mengurutkan berdasarkan skor

**Digunakan di**: Tahap 3 (Relevance Scoring & Ranking)

#### From `db` module:
```rust
use kalimantanbio_shared::db::{create_pool, fetch_all_species};
```

- `create_pool(database_url: &str) -> Result<Pool<Postgres>>` - Membuat connection pool
- `fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>>` - Fetch semua spesies dari database

**Digunakan di**: Tahap 5 (Integrasi & Testing)

### 2.3 Module-Specific Types

Modul ini mendefinisikan tipe tambahan yang spesifik untuk pencarian:

```rust
#[derive(Debug, Clone, Default)]
struct QueryFilters {
    free_text: Vec<String>,           // kata kunci umum, dicocokkan ke nama/deskripsi
    kingdom_id: Option<u64>,
    class_id: Option<u64>,
    iucn: Option<String>,
    cites: Option<String>,
    species_type: Option<String>,
    is_verified: Option<bool>,
    sort: Option<String>,
}
```

### 2.4 Cargo.toml Dependency

```toml
[dependencies]
kalimantanbio-shared = { path = "../shared" }
serde = { workspace = true }
tokio = { workspace = true }
```

### 2.5 Integration Example

```rust
use kalimantanbio_shared::{
    core::{Species, Taxonomy},
    text::{normalize_text, tokenize},
    scoring::{combine_weighted_scores, rank_by_score},
    db::fetch_all_species,
};

pub fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)> {
    // Gunakan fungsi shared library
    let normalized = normalize_text(raw_query);
    let tokens = tokenize(&normalized);
    
    // Module-specific logic
    let filters = extract_filters(&tokens);
    let filtered = filter_species(all_species, &filters);
    
    // Gunakan shared scoring
    rank_by_score(
        filtered.into_iter()
            .map(|s| (s, score_species(s, &filters)))
            .collect()
    )
}
```

---

## 3. Struktur Data (Domain Model)

### 3.1 Tipe dari Shared Library

Modul ini menggunakan `Species` dan `Taxonomy` dari shared library. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk definisi lengkap.

> **Catatan:** Struktur data sudah distandardisasi di shared library dan digunakan oleh semua modul. Tidak boleh ada modifikasi pada tipe ini tanpa koordinasi dengan project lead.

### 3.2 Data yang Digunakan

| Data | Tipe | Deskripsi |
|---|---|---|
| `scientific_name`, `common_name`, `description` | `String` | Sumber utama untuk pencocokan teks bebas (free-text search). |
| `species_type` | `String` | Filter kategori spesies (mis. endemik/asli/introduksi). |
| `iucn`, `cites`, `p106` | `String` | Atribut status konservasi. |
| `taxonomy` | `Taxonomy` | Digunakan untuk filter & scoring berbasis kingdom/class/order/family/genus. |
| `is_verified` | `bool` | Menentukan apakah data sudah diverifikasi; bisa dipakai sebagai filter kualitas hasil. |
| `observation_count`, `recorded_individuals_total`, `latest_observation_year` | `u32` / `Option<u32>` | Bisa dipakai sebagai sinyal tambahan untuk relevansi (mis. spesies yang lebih banyak diobservasi mendapat skor lebih tinggi), opsional. |

---

## 4. Tahap 1 — Parsing & Query Understanding

Mengubah query bahasa natural (String) menjadi `QueryFilters` yang terstruktur.

> **Catatan**: Fungsi `normalize_text` dan `tokenize` menggunakan **shared library**.

| Fungsi | Signature | Deskripsi |
|---|---|---|
| `normalize_text` | **FROM SHARED LIBRARY** `kalimantanbio_shared::text::normalize_text` | Lowercase, hapus tanda baca dan spasi berlebih. |
| `tokenize` | **FROM SHARED LIBRARY** `kalimantanbio_shared::text::tokenize` | Memecah teks menjadi daftar token. |
| `extract_filters` | `fn extract_filters(tokens: &[String]) -> QueryFilters` | **MODULE-SPECIFIC**: Mencocokkan token terhadap kosakata dikenal (nama status IUCN, nama taksonomi umum, kata seperti "endemik"/"dilindungi") dan sisanya masuk `free_text`. |

**Person in Charge:** **(isi nama anggota)**

---

## 5. Tahap 2 — Multi-Attribute Filtering

Menyaring kandidat spesies berdasarkan `QueryFilters`, menggunakan atribut yang tersedia dari production database.

| Fungsi | Signature | Deskripsi |
|---|---|---|
| `matches_free_text` | `fn matches_free_text(species: &Species, filters: &QueryFilters) -> bool` | Cek apakah `scientific_name`, `common_name`, atau `description` memuat kata kunci bebas. |
| `matches_taxonomy` | `fn matches_taxonomy(species: &Species, filters: &QueryFilters) -> bool` | Cek kecocokan `kingdom_id`/`class_id`. |
| `matches_conservation` | `fn matches_conservation(species: &Species, filters: &QueryFilters) -> bool` | Cek kecocokan `iucn`/`cites`. |
| `matches_species_type` | `fn matches_species_type(species: &Species, filters: &QueryFilters) -> bool` | Cek kecocokan `species_type`. |
| `filter_species` | `fn filter_species<'a>(species_list: &'a [Species], filters: &QueryFilters) -> Vec<&'a Species>` | Menggabungkan seluruh predicate di atas via `.iter().filter()`. |

**Person in Charge:** **(isi nama anggota)**

---

## 6. Tahap 3 — Relevance Scoring & Ranking

> **Catatan**: Fungsi `combine_weighted_scores` dan `rank_by_score` menggunakan **shared library**.

| Fungsi | Signature | Deskripsi |
|---|---|---|
| `score_text_match` | `fn score_text_match(species: &Species, filters: &QueryFilters) -> f64` | Skor kecocokan teks bebas terhadap nama/deskripsi. |
| `score_taxonomy_match` | `fn score_taxonomy_match(species: &Species, filters: &QueryFilters) -> f64` | Skor kecocokan taksonomi. Bisa menggunakan `calculate_taxonomy_similarity` dari shared library. |
| `score_conservation_match` | `fn score_conservation_match(species: &Species, filters: &QueryFilters) -> f64` | Skor kecocokan status konservasi. |
| `combine_scores` | **FROM SHARED LIBRARY** `kalimantanbio_shared::scoring::combine_weighted_scores` | Menggabungkan skor per atribut (weighted sum). |
| `score_species` | `fn score_species(species: &Species, filters: &QueryFilters) -> f64` | **MODULE-SPECIFIC**: Memanggil seluruh fungsi skor dan mengembalikan skor total menggunakan `combine_weighted_scores`. |
| `rank_by_score` | **FROM SHARED LIBRARY** `kalimantanbio_shared::scoring::rank_by_score` | Mengurutkan spesies berdasarkan skor (descending). |

**Person in Charge:** **(isi nama anggota)**

---

## 7. Tahap 4 — Related-Query Recommendation

| Fungsi | Signature | Deskripsi |
|---|---|---|
| `extract_common_attributes` | `fn extract_common_attributes(results: &[&Species]) -> Vec<String>` | Mengambil pola atribut yang sering muncul (mis. `iucn`/`family` yang dominan) dari hasil saat ini. |
| `generate_candidate_queries` | `fn generate_candidate_queries(filters: &QueryFilters, attrs: &[String]) -> Vec<String>` | Membuat kandidat query lanjutan berdasarkan filter & atribut umum. |
| `score_query_candidate` | `fn score_query_candidate(query: &str, original_filters: &QueryFilters) -> f64` | Menilai relevansi kandidat query terhadap query asal. |
| `rank_related_queries` | `fn rank_related_queries(candidates: Vec<String>, filters: &QueryFilters) -> Vec<(String, f64)>` | Mengurutkan kandidat query berdasarkan skor. |

**Person in Charge:** **(isi nama anggota)**

---

## 8. Tahap 5 — Integrasi & Testing

> **Catatan**: Fungsi `fetch_all_species` menggunakan **shared library** untuk koneksi database.

| Fungsi / Komponen | Signature / Bentuk | Deskripsi |
|---|---|---|
| `fetch_all_species` | **FROM SHARED LIBRARY** `kalimantanbio_shared::db::fetch_all_species` | Mengambil data dari production database via SQLx. |
| `search` | `fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)>` | **MODULE ENTRY POINT**: Pipeline utama: parse → filter → score → rank. |
| Test fixtures | Use `shared/fixtures/species.json` | Data uji coba dari shared library, untuk unit test yang tidak butuh koneksi database. |

**Person in Charge:** **(isi nama anggota)**

> **Catatan:** Setiap PIC tetap bertanggung jawab terhadap pengujian fungsi yang mereka implementasikan. PIC tahap ini berfokus pada pengujian antar-komponen, koneksi ke database production, dan pengujian end-to-end.

---

## 9. Komposisi Pipeline Utama

```rust
use kalimantanbio_shared::{
    core::Species,
    text::{normalize_text, tokenize},
    scoring::rank_by_score,
};

fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)> {
    // Gunakan shared library untuk normalisasi dan tokenisasi
    let normalized = normalize_text(raw_query);
    let tokens = tokenize(&normalized);
    
    // Module-specific: extract filters
    let filters = extract_filters(&tokens);
    
    // Module-specific: filter species
    let filtered = filter_species(all_species, &filters);
    
    // Module-specific: score each species
    let scored: Vec<(&Species, f64)> = filtered.into_iter()
        .map(|s| (s, score_species(s, &filters)))
        .collect();
    
    // Gunakan shared library untuk ranking
    rank_by_score(scored)
}
```

Pipeline konseptual:

```text
Query Mentah (String)
  ↓
Normalisasi & Tokenisasi (SHARED LIBRARY)
  ↓
Ekstraksi Filter (MODULE-SPECIFIC)
  ↓
Filtering Spesies (MODULE-SPECIFIC)
  ↓
Scoring (MODULE-SPECIFIC + SHARED LIBRARY)
  ↓
Ranking (SHARED LIBRARY)
  ↓
Hasil Terurut (Vec<(&Species, f64)>)
```

Fungsi `search` adalah **entry point** modul dan menjadi contoh penerapan **function composition** — menggabungkan beberapa fungsi kecil (dari shared library dan module-specific) menjadi satu proses besar.

---

## 10. Prinsip Functional Programming yang Perlu Dipegang Tim

### Pure Functions
Fungsi tidak mengubah state eksternal dan hanya bergantung pada input yang diberikan. Untuk input yang sama, fungsi menghasilkan output yang sama. 

**Shared library sudah menyediakan pure functions** untuk text normalization, scoring, dan ranking. Fungsi database (`fetch_all_species`) adalah pengecualian wajar karena melakukan I/O — dipisahkan tegas dari fungsi-fungsi murni lainnya di pipeline.

### Immutability
Gunakan `&T` ketika data hanya perlu dibaca; hasil transformasi dikembalikan sebagai data baru, bukan memodifikasi input. Shared library mengikuti prinsip ini.

### Higher-Order Functions
Manfaatkan `.map()`, `.filter()`, `.fold()`, `.sum()`, `.sort_by()` untuk transformasi, filtering, dan agregasi — hindari `for` loop manual di logika inti. Shared library `rank_by_score` sudah menggunakan higher-order functions.

### Function Composition
Setiap fungsi punya satu tanggung jawab jelas dan dapat diuji independen: `parse → filter → score → rank`. Shared library mendukung komposisi ini.

---

## 11. Pembagian Kerja — 5 Anggota

| # | Tahap | Fungsi / Tanggung Jawab Utama | PIC | Status |
|---|---|---|---|---|
| 1 | Parsing & Query Understanding | `extract_filters` (uses shared `normalize_text`, `tokenize`) | | Belum dimulai |
| 2 | Multi-Attribute Filtering | `matches_free_text`, `matches_taxonomy`, `matches_conservation`, `matches_species_type`, `filter_species` | | Belum dimulai |
| 3 | Relevance Scoring & Ranking | `score_text_match`, `score_taxonomy_match`, `score_conservation_match`, `score_species` (uses shared `combine_weighted_scores`, `rank_by_score`) | | Belum dimulai |
| 4 | Related-Query Recommendation | `extract_common_attributes`, `generate_candidate_queries`, `score_query_candidate`, `rank_related_queries` | | Belum dimulai |
| 5 | Integrasi & Testing | `search` pipeline, database integration (uses shared `fetch_all_species`), test fixtures, e2e test | | Belum dimulai |

### Pembagian Tanggung Jawab

Setiap PIC bertanggung jawab terhadap:

* Implementasi fungsi yang ditugaskan (module-specific logic).
* Menggunakan shared library functions dengan benar.
* Unit testing fungsi tersebut.
* Dokumentasi fungsi.
* Menjaga signature/interface yang telah disepakati.
* Melaporkan perubahan yang dapat memengaruhi komponen lain.

**PENTING**: Pahami fungsi mana yang dari shared library vs module-specific. Jangan duplikasi fungsi yang sudah ada di shared library.

---

## 12. Kesepakatan Antaranggota

Sebelum implementasi dimulai, seluruh anggota perlu menyepakati:

* **Struktur `Species`, `Taxonomy`**: Sudah distandardisasi di shared library. Lihat [MASTERPLAN.md](../MASTERPLAN.md).
* Struktur `QueryFilters` module-specific (termasuk field apa saja yang dipakai).
* Arti setiap field, terutama field yang berasal dari singkatan (`p106`, `cites`).
* Input dan output setiap fungsi module-specific.
* Function signature final untuk fungsi module-specific.
* Format data antar-tahap (`Vec<&Species>` vs `Vec<Species>`, dsb).
* Aturan validasi data (mis. apa yang terjadi jika `image_url` null atau `latest_observation_year` kosong).
* Bobot/formula scoring pada `score_species` (yang akan menggunakan `combine_weighted_scores` dari shared library).
* Format output akhir pipeline `search()`.
* Strategi testing (unit test per fungsi vs integration test dengan database).

**PENTING**: Jangan modifikasi `Species` atau `Taxonomy`. Jika ada kebutuhan perubahan, koordinasi dengan project lead (pemilik shared library).

---

## 13. Independensi Modul

Modul ini dikembangkan sebagai komponen independen dalam KalimantanBio workspace.

Prinsip yang digunakan:

* Modul dapat dikembangkan secara mandiri, terlepas dari status Modul 2–5.
* Modul dapat diuji secara mandiri menggunakan shared fixtures (`shared/fixtures/species.json`), tanpa harus selalu terkoneksi ke production database.
* Modul tidak boleh bergantung pada implementasi internal modul lain.
* Modul hanya mengekspos **interface publik (`pub fn`)** yang dibutuhkan — lihat Bagian 17.
* Integrasi dengan modul lain bersifat opsional dan melalui unified Axum API server (aplikasi). Receiver memakai mock selama pengembangan paralel.
* **Semua modul bergantung HANYA pada shared library**, bukan pada modul lain.

```text
                    KalimantanBio
                 Shared Library
                         │
        ┌────────────────┼────────────────┐
        │                │                │
       M1               M2               M3
        │                │                │
       M4               M5
        │                │
        └────────────────┴────────────────┘
                         │
              Unified Axum API Server
```

**Modul 1 hanya bergantung pada shared library, tidak pada modul lain.**

---

## 14. Kriteria Selesai Modul

* [ ] Seluruh fungsi utama telah diimplementasikan.
* [ ] Setiap fungsi memiliki unit test yang relevan.
* [ ] Pipeline `search()` dapat berjalan end-to-end, baik dengan shared fixtures maupun data dari production database.
* [ ] Input query bahasa natural dapat diproses sesuai spesifikasi.
* [ ] Output menghasilkan format yang telah disepakati (`Vec<(&Species, f64)>` terurut).
* [ ] Tidak ada dependency yang tidak diperlukan (hanya bergantung ke shared library).
* [ ] Tidak ada state global yang tidak diperlukan.
* [ ] Dokumentasi fungsi dan struktur data tersedia.
* [ ] Terdapat demonstrasi penggunaan modul (mis. contoh query & hasilnya).
* [ ] Modul dapat dijalankan secara independen menggunakan shared fixtures.
* [ ] Integrasi dengan shared library berfungsi dengan baik.
* [ ] Interface publik (`pub fn search`) didefinisikan jelas dan teruji — lihat Bagian 17.
* [ ] Seluruh `pub fn` memiliki dokumentasi rustdoc lengkap.

---

## 15. Contoh Skenario Pengujian

### Skenario 1 — Pencarian berbasis kata kunci umum

**Input:**
```text
"orangutan kalimantan"
```

**Expected Output:**
```text
[
  { scientific_name: "Pongo pygmaeus", common_name: "Orangutan Borneo", score: <tinggi> },
  ...
]
```

**Fungsi yang diuji:**
* `normalize_text` (shared library), `tokenize` (shared library), `extract_filters` (module-specific)
* `matches_free_text`, `score_text_match`

---

### Skenario 2 — Pencarian berbasis status konservasi

**Input:**
```text
"spesies berstatus kritis (CR)"
```

**Expected Output:**
```text
Daftar spesies dengan iucn == "CR", diurutkan berdasarkan skor relevansi.
```

**Fungsi yang diuji:**
* `extract_filters` (mendeteksi kata "kritis"/"CR" sebagai filter iucn)
* `matches_conservation`, `score_conservation_match`

---

### Edge Cases

| Case | Input | Expected Result |
|---|---|---|
| Empty input | `""` | Kembalikan list kosong atau seluruh spesies tanpa filter (disepakati tim). |
| Query tidak dikenali | `"asdkjaskjd"` | Tidak ada filter terbentuk, `free_text` diisi token tersebut, kemungkinan hasil kosong. |
| Tidak ada hasil cocok | `"spesies punah dari mars"` | List kosong, tidak error. |
| Banyak hasil cocok | `"mamalia"` | List terurut berdasarkan skor, tidak crash meski hasil banyak. |
| Field opsional null | `image_url: null`, `latest_observation_year: null` | Pipeline tetap berjalan tanpa panic (gunakan `Option<T>`). |

---

## 16. Langkah Selanjutnya

1. **Week 1**: Tunggu shared library v0.1.0 dari project lead (Anda harus menyelesaikan Week 1).
2. Finalisasi struktur `QueryFilters` module-specific bersama seluruh anggota.
3. Sepakati input, output, dan signature setiap fungsi module-specific.
4. Konfirmasi pembagian fungsi berdasarkan PIC (isi tabel Bagian 11).
5. Setiap PIC membuat signature dan dokumentasi singkat fungsi masing-masing.
6. Review interface bersama sebelum implementasi logic dimulai.
7. Gunakan shared fixtures (`shared/fixtures/species.json`) untuk unit test.
8. Implementasikan fungsi secara paralel sesuai pembagian kerja.
9. Setiap PIC membuat unit test untuk fungsi masing-masing.
10. Gabungkan seluruh tahap ke dalam pipeline `search()`.
11. Lakukan pengujian end-to-end dengan production database.
12. Dokumentasikan hasil dan contoh penggunaan modul.
13. Review akhir sebelum modul dianggap selesai.
14. Integrasi ke unified Axum API server (koordinasi dengan API server team).

**PENTING**: Modul ini tidak bisa dimulai sebelum shared library selesai di Week 1. Gunakan waktu Week 1 untuk:
- Mempelajari shared library API
- Merencanakan module-specific logic
- Menyiapkan test scenarios
- Membagi tugas antar anggota

---

## 17. Interface Publik & Komunikasi Antar Modul

Bagian ini menjelaskan batas `mod` dan `pub fn` Modul 1 sebagai bukti pemenuhan aspek **Komunikasi Antar Module** pada rubrik penilaian (40%).

### 17.1 Batas Modul (Owns / Does Not Own / Internal / Public)

| Aspek | Isi |
| --- | --- |
| **Owns** | Parsing query bahasa natural, filtering multi-atribut, relevance scoring, dan rekomendasi query lanjutan untuk pencarian spesies. |
| **Does Not Own** | Data spesies/taksonomi (shared library + database), visualisasi jaringan (M2), hierarki taksonomi mendalam (M3), perbandingan spesies (M4), dan eksplorasi publikasi (M5). |
| **Internal** | `normalize_text`/`tokenize` (panggilan shared), `extract_filters`, `matches_*`, `filter_species`, `score_*`, dan seluruh fungsi rekomendasi. |
| **Publicly Exposes** | `search` (main) dan `recommend_related_queries` (opsional) — dua kapabilitas yang dapat digunakan Axum handler maupun modul lain. |

### 17.2 Fungsi Publik (`pub fn`) — Modul 1

| `pub fn` | Provider | Consumer Potensial | Purpose | Input | Output | Why Needed |
| --- | --- | --- | --- | --- | --- | --- |
| `search` | Modul 1 | Axum handler `/api/v1/search`; opsional M2 (memilih spesies pusat), M4 (kandidat species_ids), M5 (cakupan publikasi) | Menemukan & mengurutkan spesies relevan | `raw_query: &str`, `all_species: &[Species]` | `Vec<(&Species, f64)>` | Kapabilitas utama modul; dipakai API dan modul lain sebagai pintu masuk DISCOVER. |
| `recommend_related_queries` *(komposisi dari fungsi rekomendasi existing)* | Modul 1 | Axum handler `/api/v1/search/recommendations` (opsional) | Menghasilkan rekomendasi query lanjutan | `raw_query: &str`, `all_species: &[Species]` | `Vec<(String, f64)>` | Mengekspos kapabilitas rekomendasi modul sebagai satu fungsi yang dapat dipanggil. |

```rust
/// Mencari spesies menggunakan query bahasa natural atau filter terstruktur.
///
/// # Arguments
///
/// * `raw_query` - Query pengguna (mis. "orangutan kalimantan" atau "iucn:CR").
/// * `all_species` - Seluruh data spesies dari shared library.
///
/// # Returns
///
/// Daftar spesies terurut berdasarkan skor relevansi (tertinggi dulu).
///
/// # Example
///
/// ```
/// let results = search("orangutan kalimantan", &all_species);
/// ```
pub fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)>
```

### 17.3 Visibilitas Fungsi

| Fungsi | Visibilitas | Lapisan | Konsumen |
| --- | --- | --- | --- |
| `normalize_text`, `tokenize`, `calculate_taxonomy_similarity`, `combine_weighted_scores`, `rank_by_score`, `fetch_all_species`, `create_pool` | `pub` (shared library) | Shared | Dipanggil internal oleh pipeline modul |
| `extract_filters` | private | Internal helper (module-specific) | Pipeline `search` |
| `matches_free_text`, `matches_taxonomy`, `matches_conservation`, `matches_species_type`, `filter_species` | private | Internal helper (module-specific) | Pipeline `search` |
| `score_text_match`, `score_taxonomy_match`, `score_conservation_match`, `score_species` | private | Internal helper (module-specific) | Pipeline `search` |
| `extract_common_attributes`, `generate_candidate_queries`, `score_query_candidate`, `rank_related_queries` | private | Internal helper (module-specific) | Wrapper `recommend_related_queries` |
| `search` | `pub fn` | Module API | Axum handler; opsional M2/M4/M5 |
| `recommend_related_queries` | `pub fn` | Module API (opsional) | Axum handler; opsional modul lain |

> Aturan: fungsi pembantu (helper) tetap **private**. Hanya kapabilitas bermakna yang diekspos sebagai `pub fn`. Axum HTTP handler hidup di crate `api-server`, bukan di crate modul.

### 17.4 Komunikasi Antar Modul (Provider → Receiver)

```text
Module 1 (Provider)
      │
      │ pub fn search(...)
      ▼
Axum handler /api/v1/search  (Receiver / Aplikasi)
      │  hasil: Vec<(&Species, f64)>
      ▼
Django API → Frontend
```

Hubungan opsional dengan modul lain (melalui **interface publik**, dikomposisikan di aplikasi `api-server`, bukan dependency crate):

```text
Modul 1 ──search results──▶ Modul 2 (spesies pusat, opsional)
Modul 1 ──species_ids──────▶ Modul 4 (kandidat perbandingan, opsional)
Modul 1 ──species scope────▶ Modul 5 (cakupan pencarian publikasi, opsional)
```

| Provider | `pub fn` | Receiver | Purpose | Data yang Dikirim | Priority |
| --- | --- | --- | --- | --- | --- |
| Modul 1 | `search` | Axum handler | Disimpan hasil pencarian untuk respons API | Query + semua spesies | **Required** (API) |
| Modul 1 | `search` | Modul 4 | Kandidat species_ids untuk perbandingan | `Vec<(&Species, f64)>` → species_ids | Optional |
| Modul 1 | `search` | Modul 2 | Spesies pusat untuk eksplorasi relasi | species (satu) | Optional |
| Modul 1 | `search` | Modul 5 | Cakupan spesies untuk eksplorasi publikasi | species_ids | Optional |

> **Selama pengembangan paralel:** Receiver (M2/M4/M5) boleh menggunakan **mock** hasil `search`. Setelah integrasi, mock diganti implementasi nyata melalui `pub fn` yang sama. Modul 1 tidak pernah mengonsumsi fungsi modul lain.

### 17.5 Contoh Komposisi di Lapisan Aplikasi (Axum handler)

```rust
// crates/api-server/src/routes/compare.rs (HYBRID)
async fn compare_from_search(query: Query<SearchAndCompare>) -> Json<Response> {
    // Modul 1: temukan kandidat (dikomposisikan di aplikasi, bukan dependency crate M4→M1)
    let results = species_search::search(&query.q, &all_species);
    let ids: Vec<u64> = results.into_iter().take(3).map(|(s, _)| s.id).collect();
    // Modul 4: bandingkan kandidat
    let comparison = species_comparison::compare_species(
        &ComparisonQuery { species_ids: ids },
        &all_species,
        &all_observations,
        &weights,
    )?;
    Json(Response::success(comparison))
}
```

### 17.6 Rustdoc — Modul 1

Rustdoc diwajibkan untuk **seluruh `pub fn`** dan **seluruh tipe publik** modul ini:

* `search` — lengkap dengan `# Arguments`, `# Returns`, dan contoh penggunaan.
* `recommend_related_queries` (jika diekspos) — deskripsi, parameter, dan contoh.
* Tipe publik yang muncul di signature `pub fn` (mis. `QueryFilters` jika diekspor ke modul lain).

Command verifikasi:

```bash
cargo doc --workspace --no-deps --open
cargo check
cargo test
```

Status saat ini di dokumen: **Rustdoc planned** (rutin dihasilkan setelah fungsi diimplementasikan, belum diklaim verified).

---

## 18. Rubrik — Evidence Modul 1

| Rubrik | Evidence di Planning Modul 1 | Bukti Implementasi yang Masih Diperlukan |
| --- | --- | --- |
| Repositori Github (15%) | Lokasi `crates/species-search` dan command build/test/rustdoc | Repositori dibuat & diakses dosen |
| Prioritas Modul (25%) | Prioritas fitur awal: parsing → filter → scoring → rekomendasi | Implementasi fitur prioritas |
| Rustdoc (20%) | Rustdoc diwajibkan untuk semua `pub fn` (Bagian 17.6) | Generate & aksesibel |
| Komunikasi Antar Module (40%) | Batas `mod`/`pub fn` (17.1–17.2), visibilitas (17.3), matriks komunikasi (17.4) | Interface diimplementasikan & diuji |
