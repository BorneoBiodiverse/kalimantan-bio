# Planning Function — Modul 3: Taxonomy & Classification Explorer

**Mata Kuliah:** Pemrograman Fungsional  
**Bahasa:** Rust  
**Cakupan:** Pembangunan pohon taksonomi hirarkis, eksplorasi berbasis takson, analisis keanekaragaman (diversity), analisis coverage/gap, dan eksplorasi taksonomi endemik Kalimantan.  

**Repository:** Part of KalimantanBio monorepo workspace (`crates/taxonomy`)  
**API Integration:** Production KalimantanBio API (PostgreSQL via shared library)

> **PENTING**: Modul ini menggunakan **Shared Library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk arsitektur lengkap.

---

## 1. Tujuan Modul

Modul ini bertujuan untuk memproses data mentah klasifikasi biologis dan mengubahnya menjadi struktur hirarkis (pohon taksonomi) yang dapat dieksplorasi, dianalisis statistiknya, dan dibandingkan dengan data referensi global untuk menemukan celah data (*taxonomic gap*).

Modul ini bertanggung jawab untuk:

* Membangun *Taxonomic Tree* dari data spesies dan takson.
* Menghitung statistik keanekaragaman (jumlah family, genus, spesies) untuk kelompok taksonomi tertentu.
* Menganalisis *Taxonomic Gap* (data yang hilang) dengan membandingkannya terhadap *checklist* referensi.
* Mengidentifikasi dan memfilter taksonomi yang berstatus endemik Kalimantan/Borneo.

### Batasan Modul

Modul ini **tidak bertanggung jawab** terhadap:

* Pencarian berbasis *natural language* atau *fuzzy search* (Tanggung jawab Modul 1).
* Perbandingan visual morfologi antar spesies secara *side-by-side* (Tanggung jawab Modul 4).
* Penyimpanan data sitasi, jurnal ilmiah, dan publikasi (Tanggung jawab Modul 5).

---

## 2. Shared Library Integration

Modul ini menggunakan **shared library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum yang digunakan bersama dengan modul lain.

### 2.1 Shared Types Used

```rust
use kalimantanbio_shared::core::{Species, Taxonomy, TaxonomicRank};
```

**Species** dan **Taxonomy** adalah tipe standar yang digunakan oleh semua modul. Definisi lengkap ada di shared library.

- `Species` - Struktur data lengkap spesies dari production database
- `Taxonomy` - Struktur hierarki taksonomi (kingdom → genus)
- `TaxonomicRank` - Enum untuk level taksonomi

### 2.2 Shared Functions Used

#### From `collections` module:
```rust
use kalimantanbio_shared::collections::set_difference;
```

- `set_difference<T>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T>` - Mencari elemen di A yang tidak ada di B (untuk gap analysis)

**Digunakan di**: Tahap 4 (Coverage & Gap Analysis)

#### From `stats` module:
```rust
use kalimantanbio_shared::stats::{count_by_key, frequency_distribution, calculate_coverage_percentage};
```

- `count_by_key<T, K>(items: &[T], key_fn: impl Fn(&T) -> K) -> HashMap<K, usize>` - Menghitung frekuensi per key
- `frequency_distribution<T, K>(items: &[T], key_fn: impl Fn(&T) -> K) -> Vec<(K, usize)>` - Distribusi frekuensi terurut
- `calculate_coverage_percentage(available: usize, total: usize) -> f64` - Menghitung persentase coverage

**Digunakan di**: Tahap 3 (Diversity & Endemic Analytics), Tahap 4 (Coverage & Gap Analysis)

#### From `db` module:
```rust
use kalimantanbio_shared::db::{create_pool, fetch_all_species};
```

- `create_pool(database_url: &str) -> Result<Pool<Postgres>>` - Membuat connection pool
- `fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>>` - Fetch semua spesies

**Digunakan di**: Tahap 5 (Integrasi Pipeline & Testing)

### 2.3 Module-Specific Types

Modul ini mendefinisikan tipe tambahan yang spesifik untuk taxonomy exploration:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TaxonId(pub u64);

#[derive(Debug, Clone)]
struct Taxon {
    id: TaxonId,
    name: String,
    rank: TaxonomicRank,
    parent_id: Option<TaxonId>,
}

#[derive(Debug, Clone, Default)]
struct DiversityStats {
    total_families: usize,
    total_genera: usize,
    total_species: usize,
    endemic_species_count: usize,
}

#[derive(Debug, Clone, Default)]
struct BioDataInput {
    raw_taxons: Vec<RawTaxonRecord>,
    raw_species: Vec<RawSpeciesRecord>,
    reference_genera: std::collections::HashSet<String>,
}

#[derive(Debug, Clone)]
struct GapReport {
    missing_taxa: Vec<String>,
    coverage_score: f64,
}

#[derive(Debug, Clone)]
struct ExplorationReport {
    endemic_genera: Vec<Taxon>,
    richness_rank: Vec<(Taxon, usize)>,
    missing_taxa: Vec<String>,
    coverage_score: f64,
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
    collections::set_difference,
    stats::{count_by_key, calculate_coverage_percentage},
};
use std::collections::HashSet;

pub fn analyze_taxonomic_gap(
    our_genera: &HashSet<String>,
    reference_genera: &HashSet<String>,
) -> GapReport {
    // Gunakan shared library untuk set difference
    let missing = set_difference(reference_genera, our_genera);
    
    // Gunakan shared library untuk coverage calculation
    let coverage = calculate_coverage_percentage(
        our_genera.len(),
        reference_genera.len()
    );
    
    GapReport {
        missing_taxa: missing,
        coverage_score: coverage,
    }
}

pub fn calculate_diversity(species_list: &[Species]) -> DiversityStats {
    // Gunakan shared library untuk counting
    let families = count_by_key(
        species_list,
        |s| s.taxonomy.family.clone()
    );
    
    let genera = count_by_key(
        species_list,
        |s| s.taxonomy.genus.clone()
    );
    
    DiversityStats {
        total_families: families.len(),
        total_genera: genera.len(),
        total_species: species_list.len(),
        endemic_species_count: species_list.iter()
            .filter(|s| s.species_type == "Endemic")
            .count(),
    }
}
```

---

## 3. Struktur Data (Domain Model)

### 3.1 Tipe dari Shared Library

Modul ini menggunakan `Species`, `Taxonomy`, dan `TaxonomicRank` dari shared library. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk definisi lengkap.

> **Catatan:** Struktur data sudah distandardisasi di shared library dan digunakan oleh semua modul. Tidak boleh ada modifikasi pada tipe ini tanpa koordinasi dengan project lead.

### 3.2 Data yang Digunakan

| Data | Tipe | Deskripsi |
| --- | --- | --- |
| `raw_taxons` | `Vec<RawTaxonRecord>` | Data mentah node taksonomi (Ordo, Famili, Genus, dll). |
| `raw_species` | `Vec<RawSpeciesRecord>` | Data mentah daftar spesies yang terdaftar di KalimantanBio. |
| `reference_genera` | `HashSet<String>` | Daftar nama genus dari literatur global untuk Gap Analysis. |

---

## 4. Tahap 1 — Data Ingestion & Parsing

Membaca data mentah, memvalidasi, dan memetakan ke dalam Domain Model tanpa mutasi state.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `parse_taxons` | `fn parse_taxons(raw: &[RawTaxonRecord]) -> Result<Vec<Taxon>, ParseError>` | **MODULE-SPECIFIC**: Memvalidasi dan memetakan data mentah takson menjadi `Vec<Taxon>`. |
| `parse_species` | `fn parse_species(raw: &[RawSpeciesRecord]) -> Result<Species, ParseError>` | **MODULE-SPECIFIC**: Memvalidasi data spesies dan memastikan `genus_id` valid. |
| `validate_hierarchy` | `fn validate_hierarchy(taxons: &[Taxon]) -> bool` | **MODULE-SPECIFIC**: Pure function untuk mengecek tidak adanya cyclic reference atau orphan node. |

**Person in Charge:** **Anggota 1 — Data ingestion, parsing, dan validasi hierarki (nama menyusul).**

---

## 5. Tahap 2 — Taxonomic Tree Construction

Membangun struktur pohon (hirarki) menggunakan pendekatan FP (rekursi, folding, dan pengelompokan data).

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `build_taxonomy_map` | `fn build_taxonomy_map(taxons: &[Taxon]) -> HashMap<TaxonId, Vec<TaxonId>>` | **MODULE-SPECIFIC**: Mengelompokkan `parent_id` ke `children_ids` menggunakan `.fold()` atau `.group_by()`. |
| `get_lineage` | `fn get_lineage(taxon_id: &TaxonId, map: &HashMap<TaxonId, Taxon>) -> Vec<Taxon>` | **MODULE-SPECIFIC**: Menelusuri ke atas (iteratif) untuk mendapatkan Kingdom -> ... -> Genus dari satu node. |
| `get_subtree_species` | `fn get_subtree_species(target: &TaxonId, map: &HashMap<TaxonId, Vec<TaxonId>>, species: &[Species]) -> Vec<Species>` | **MODULE-SPECIFIC**: Mengumpulkan semua spesies yang berada di bawah target takson. |
| `extract_our_genera` | `fn extract_our_genera(taxons: &[Taxon]) -> HashSet<String>` | **MODULE-SPECIFIC**: Mengekstrak semua nama Genus yang ada di database lokal untuk keperluan analisis gap. |

**Person in Charge:** **Anggota 2 — Konstruksi pohon taksonomi dan traversal (nama menyusul).**

---

## 6. Tahap 3 — Diversity & Endemic Analytics

> **Catatan**: Fungsi `count_by_key` dan `frequency_distribution` menggunakan **shared library**.

Melakukan agregasi data untuk menghitung statistik dan memfilter spesies endemik menggunakan Higher-Order Functions.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `calculate_diversity` | `fn calculate_diversity(species_subset: &[Species]) -> DiversityStats` | **Uses SHARED `count_by_key`**: Menghitung total genus, famili, dan jumlah spesies endemik. Counting keluarga/genus diserahkan ke shared library. |
| `filter_endemic_taxa` | `fn filter_endemic_taxa(taxons: &[Taxon], species: &[Species]) -> Vec<Taxon>` | **MODULE-SPECIFIC**: Memfilter hanya Genus/Family yang memiliki spesies endemik Kalimantan menggunakan `.filter()`. |
| `rank_taxon_richness` | `fn rank_taxon_richness(map: &HashMap<TaxonId, Vec<TaxonId>>, species: &[Species]) -> Vec<(Taxon, usize)>` | **Uses SHARED `frequency_distribution`**: Mengurutkan Famili berdasarkan jumlah spesies terbanyak (Species Richness). |
| `get_subtree_species` | `fn get_subtree_species(target: &TaxonId, map: &HashMap<TaxonId, Vec<TaxonId>>, species: &[Species]) -> Vec<Species>` | **MODULE-SPECIFIC**: Mengumpulkan semua spesies yang berada di bawah target takson. |

**Person in Charge:** **Anggota 3 — Analisis keanekaragaman dan endemik (nama menyusul).**

---

## 7. Tahap 4 — Coverage & Gap Analysis

> **Catatan**: Fungsi `set_difference` dan `calculate_coverage_percentage` menggunakan **shared library**.

Membandingkan data internal dengan referensi eksternal untuk menemukan Taxonomic Gap menggunakan Set Operations.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `find_missing_taxa` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::collections::set_difference` | Menggunakan Set Difference untuk mencari Genus yang belum ada di database. |
| `calculate_coverage_score` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::stats::calculate_coverage_percentage` | Menghitung persentase cakupan data (0.0 hingga 1.0). |
| `extract_our_genera` | `fn extract_our_genera(taxons: &[Taxon]) -> HashSet<String>` | **MODULE-SPECIFIC**: Mengekstrak semua nama Genus yang ada di database lokal. |
| `generate_gap_report` | `fn generate_gap_report(missing: &[String], score: f64) -> GapReport` | **MODULE-SPECIFIC**: Menyusun hasil analisis gap menjadi struktur laporan. |

**Person in Charge:** **Anggota 4 — Analisis coverage dan gap (nama menyusul).**

---

## 8. Tahap 5 — Integrasi Pipeline & Testing

> **Catatan**: Fungsi database menggunakan **shared library**.

Menggabungkan fungsi-fungsi di atas menjadi entry point dan melakukan End-to-End testing.

| Fungsi / Komponen | Signature / Bentuk | Deskripsi |
| --- | --- | --- |
| `generate_taxonomy_report` | `fn generate_taxonomy_report(input: &BioDataInput) -> Result<ExplorationReport, PipelineError>` | **MODULE ENTRY POINT**: Entry point yang merangkai Tahap 1 hingga Tahap 4 menjadi satu laporan utuh. |
| `fetch_all_species` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::db::fetch_all_species` | Fetch semua spesies dari production database. |
| Unit Tests | `mod tests { ... }` | Menguji `get_lineage`, `find_missing_taxa`, dan `calculate_diversity` dengan dummy data. |
| Pipeline Validation | `cargo test` | Menjalankan seluruh skenario pengujian secara otomatis. |

**Person in Charge:** **Anggota 5 — Integrasi pipeline, unit testing, dan dokumentasi (nama menyusul).**

> **Catatan:** Setiap PIC tetap bertanggung jawab terhadap pengujian fungsi yang mereka implementasikan. Jika tahap ini merupakan tahap integrasi, PIC berfokus pada pengujian antar-komponen dan pengujian end-to-end.

---

## 9. Komposisi Pipeline Utama

Setelah seluruh tahap tersedia, fungsi utama modul menggabungkan proses menjadi satu pipeline.

```rust
use kalimantanbio_shared::{
    collections::set_difference,
    stats::calculate_coverage_percentage,
};

fn generate_taxonomy_report(input: &BioDataInput) -> Result<ExplorationReport, PipelineError> {
    let step_1_taxons = parse_taxons(&input.raw_taxons)?;
    let step_1_species = parse_species(&input.raw_species)?;
    
    let step_2_map = build_taxonomy_map(&step_1_taxons);
    let our_genera = extract_our_genera(&step_1_taxons);
    
    let step_3_endemic = filter_endemic_taxa(&step_1_taxons, &step_1_species);
    let step_3_richness = rank_taxon_richness(&step_2_map, &step_1_species);
    
    // Shared library: set_difference
    let step_4_missing = set_difference(&input.reference_genera, &our_genera);
    // Shared library: calculate_coverage_percentage
    let step_4_score = calculate_coverage_percentage(
        our_genera.len(),
        input.reference_genera.len()
    );
    
    Ok(ExplorationReport {
        endemic_genera: step_3_endemic,
        richness_rank: step_3_richness,
        missing_taxa: step_4_missing,
        coverage_score: step_4_score,
    })
}
```

Pipeline konseptual:

```text
Input
  ↓
[Stage 1: Parsing & Validation]        (module-specific)
  ↓
[Stage 2: Tree Construction]           (module-specific)
  ↓
[Stage 3: Diversity & Endemic Filtering] (uses SHARED count_by_key)
  ↓
[Stage 4: Set Operations for Gap Analysis] (uses SHARED set_difference)
  ↓
Output (ExplorationReport)
```

Fungsi utama merupakan **entry point** modul dan menjadi contoh penerapan **function composition**, yaitu menggabungkan fungsi shared library dengan fungsi module-specific menjadi satu proses yang lebih besar.

---

## 10. Prinsip Functional Programming yang Perlu Dipegang Tim

### Pure Functions

Fungsi sebaiknya tidak mengubah state eksternal dan hanya bergantung pada input yang diberikan.

```text
Input → Function → Output
```

Untuk input yang sama, fungsi idealnya menghasilkan output yang sama.

### Immutability

Hindari memodifikasi data input secara langsung.

Gunakan reference/borrow seperti `&T` ketika data hanya perlu dibaca dan hasil transformasi dikembalikan sebagai data baru apabila diperlukan.

### Higher-Order Functions

Manfaatkan iterator dan fungsi seperti:

```rust
.map()
.filter()
.fold()
.sum()
```

untuk melakukan transformasi, filtering, dan agregasi data.

Gunakan pendekatan iterator ketika lebih sesuai dengan karakteristik operasi yang dilakukan.

### Function Composition

Pecah proses utama menjadi fungsi-fungsi kecil yang dapat dikombinasikan.

```text
Function A
    ↓
Function B
    ↓
Function C
    ↓
Function D
```

Setiap fungsi sebaiknya memiliki satu tanggung jawab yang jelas dan dapat diuji secara independen.

---

## 11. Pembagian Kerja — 5 Anggota

| Anggota | Tahap yang Dipegang | Fokus Tanggung Jawab |
| --- | --- | --- |
| Anggota 1 | Tahap 1 | Data ingestion, parsing, dan validasi hierarki. |
| Anggota 2 | Tahap 2 | Konstruksi pohon taksonomi dan traversal. |
| Anggota 3 | Tahap 3 | Analisis keanekaragaman dan endemik. |
| Anggota 4 | Tahap 4 | Analisis coverage dan gap. |
| Anggota 5 | Tahap 5 | Integrasi pipeline, unit testing, dan dokumentasi. |

| # | Tahap | Fungsi / Tanggung Jawab Utama | PIC | Status |
| --- | --- | --- | --- | --- |
| 1 | Tahap 1 | `parse_taxons`, `parse_species`, `validate_hierarchy` | Anggota 1 | Belum dimulai |
| 2 | Tahap 2 | `build_taxonomy_map`, `get_lineage`, `get_subtree_species` | Anggota 2 | Belum dimulai |
| 3 | Tahap 3 | `calculate_diversity`, `filter_endemic_taxa`, `rank_taxon_richness` (counting via shared) | Anggota 3 | Belum dimulai |
| 4 | Tahap 4 | `extract_our_genera`, `generate_gap_report` (set ops via shared) | Anggota 4 | Belum dimulai |
| 5 | Tahap 5 | `generate_taxonomy_report` & Unit Testing | Anggota 5 | Belum dimulai |

> **Catatan:** Fungsi `set_difference`, `calculate_coverage_percentage`, `count_by_key`, dan `frequency_distribution` berasal dari **shared library** (`kalimantanbio-shared`) dan **tidak perlu diimplementasikan** di modul ini. Data diambil dari production database melalui `fetch_all_species` (shared library).

### Pembagian Tanggung Jawab

Setiap PIC bertanggung jawab terhadap:

* Implementasi fungsi yang ditugaskan.
* Unit testing fungsi tersebut.
* Dokumentasi fungsi.
* Menjaga signature/interface yang telah disepakati.
* Melaporkan perubahan yang dapat memengaruhi komponen lain.

---

## 12. Kesepakatan Antaranggota

Sebelum implementasi dimulai, seluruh anggota perlu menyepakati:

* Struktur domain model.
* Arti setiap field.
* Input dan output setiap fungsi.
* Function signature.
* Format data yang digunakan antar-tahap.
* Aturan validasi data.
* Aturan transformasi data.
* Metode perhitungan atau scoring jika digunakan.
* Format output akhir.
* Strategi testing.

Tujuannya adalah memastikan fungsi yang dikembangkan oleh anggota berbeda tetap dapat dikombinasikan tanpa perubahan besar pada interface masing-masing.

---

## 13. Independensi Modul

Modul ini dikembangkan sebagai komponen independen dalam KalimantanBio.

Prinsip yang digunakan:

* Modul dapat dikembangkan secara mandiri.
* Modul dapat diuji secara mandiri.
* Modul tidak boleh bergantung pada implementasi internal modul lain.
* Modul hanya bergantung pada **shared library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum.
* Modul hanya mengekspos **interface publik (`pub fn`)** yang dibutuhkan — lihat Bagian 17.
* Integrasi dengan modul lain bersifat opsional dan dilakukan melalui interface publik yang telah disepakati, dikomposisikan di lapisan aplikasi (Axum api-server) — bukan sebagai dependency crate.
* Jangan mengasumsikan dependency terhadap modul lain tanpa kebutuhan teknis yang jelas.

```text
          KalimantanBio
               │
               │  ((shared))  kalimantanbio-shared
               │
          ┌────┴────┐
          │         │
     Modul ini    Modul lain
       (M3)       (M1, M2, M4, M5)
```

Modul ini (M3) hanya bergantung pada shared library, bukan pada modul lain.

Diagram di atas menggambarkan hubungan **konseptual**, bukan dependency teknis.

---

## 14. Kriteria Selesai Modul

Modul dianggap siap untuk tahap akhir apabila:

* [ ] Seluruh fungsi utama telah diimplementasikan.
* [ ] Fungsi-fungsi dari **shared library** digunakan, bukan diduplikasi.
* [ ] Setiap fungsi memiliki unit test yang relevan.
* [ ] Pipeline utama dapat berjalan end-to-end.
* [ ] Input dapat diproses sesuai spesifikasi.
* [ ] Output menghasilkan format yang telah disepakati.
* [ ] Interface publik (`pub fn generate_taxonomy_report`) didefinisikan jelas dan teruji — lihat Bagian 17.
* [ ] Seluruh `pub fn` memiliki dokumentasi rustdoc lengkap.
* [ ] Tidak terdapat dependency yang tidak diperlukan.
* [ ] Tidak terdapat state global yang tidak diperlukan.
* [ ] Dokumentasi fungsi dan struktur data tersedia.
* [ ] Terdapat demonstrasi penggunaan modul.
* [ ] Modul dapat dijalankan secara independen.

---

## 15. Contoh Skenario Pengujian

### Skenario 1 — Menghitung Keanekaragaman Familia Tertentu

**Input:**

```text
target = "Dipterocarpaceae", species_list = [meranti, kapur, ...]
```

**Expected Output:**

```text
DiversityStats { total_families: 1, total_genera: 15, total_species: 120, endemic_species_count: 45 }
```

**Fungsi yang diuji:**

* `get_subtree_species`
* `calculate_diversity`
* `rank_taxon_richness`

---

### Skenario 2 — Mencari Taxonomic Gap

**Input:**

```text
our_db = ["Genus A", "Genus B"], reference = ["Genus A", "Genus B", "Genus C"]
```

**Expected Output:**

```text
vec!["Genus C"] (Missing Taxa)
```

**Fungsi yang diuji:**

* `extract_our_genera`
* `generate_gap_report`
* `kalimantanbio_shared::collections::set_difference` (shared)
* `kalimantanbio_shared::stats::calculate_coverage_percentage` (shared)

---

### Edge Cases

| Case | Input | Expected Result |
| --- | --- | --- |
| Empty input | `species_list = []` | Mengembalikan `DiversityStats` dengan semua nilai 0. |
| Invalid input | `genus_id` merujuk pada ID yang tidak ada | `parse_species` mengembalikan `ParseError`. |
| No matching result | Tidak ada spesies endemik di subtree | `filter_endemic_taxa` mengembalikan `Vec` kosong `[]`. |
| Multiple matches | Beberapa Genus memiliki nama yang sama di referensi | Semua Genus yang hilang tetap terdeteksi dan dihitung skor cakupan dengan benar. |

---

## 16. Langkah Selanjutnya

1. Verifikasi bahwa **shared library** (`kalimantanbio-shared`) sudah tersedia; jika belum, blokir pengerjaan sampai siap.
2. Finalisasi domain model bersama project lead (tipe `Species`, `Taxonomy` sudah standar).
3. Sepakati input, output, dan signature setiap fungsi yang spesifik untuk modul.
4. Siapkan data dummy atau test fixtures (gunakan fixtures dari shared library bila tersedia).
5. Implementasikan fungsi modul; gunakan fungsi shared library untuk set operations dan stats.
6. Buat unit test untuk setiap fungsi modul.
7. Gabungkan seluruh tahap ke dalam pipeline utama.
8. Lakukan pengujian end-to-end (termasuk koneksi production database via shared library).
9. Dokumentasikan hasil dan contoh penggunaan modul.
10. Review akhir sebelum modul dianggap selesai.

---

## 17. Interface Publik & Komunikasi Antar Modul

Bagian ini menjelaskan batas `mod` dan `pub fn` Modul 3 sebagai bukti pemenuhan aspek **Komunikasi Antar Module** pada rubrik penilaian (40%).

### 17.1 Batas Modul (Owns / Does Not Own / Internal / Public)

| Aspek | Isi |
| --- | --- |
| **Owns** | Ingestion data takson, konstruksi pohon taksonomi, analisis keanekaragaman/endemik, dan analisis cakupan/gap. |
| **Does Not Own** | Data spesies/taksonomi dasar (shared library), pencarian spesies (M1), relasi antarspesies (M2), perbandingan (M4), publikasi (M5). |
| **Internal** | `parse_taxons`, `parse_species`, `validate_hierarchy`, `build_taxonomy_map`, `get_lineage`, `get_subtree_species`, `extract_our_genera`, `filter_endemic_taxa`, `rank_taxon_richness`, `generate_gap_report`. |
| **Publicly Exposes** | `generate_taxonomy_report` (main), `calculate_diversity` (keanekaragaman), dan `analyze_taxonomic_gap` (gap) — kapabilitas yang dapat digunakan Axum handler maupun modul lain. |

### 17.2 Fungsi Publik (`pub fn`) — Modul 3

| `pub fn` | Provider | Consumer Potensial | Purpose | Input | Output | Why Needed |
| --- | --- | --- | --- | --- | --- | --- |
| `generate_taxonomy_report` | Modul 3 | Axum handler `/api/v1/taxonomy/*` | Entry point pipeline Tahap 1–4 | `&BioDataInput` | `Result<ExplorationReport, PipelineError>` | Kapabilitas utama modul untuk aplikasi. |
| `calculate_diversity` *(sudah `pub` di contoh)* | Modul 3 | Axum handler; opsional M2/M4 (statistik keanekaragaman) | Menghitung statistik keanekaragaman | `&[Species]` | `DiversityStats` | Statistik ringkas yang dapat dipakai modul lain. |
| `analyze_taxonomic_gap` *(sudah `pub` di contoh)* | Modul 3 | Axum handler `/api/v1/taxonomy/gaps`; opsional M5 (isi penelitian) | Membandingkan genus lokal vs referensi | `&HashSet<String>` × 2 | `GapReport` | Analisis gap yang bernilai ilmiah untuk proyek. |

```rust
/// Menghasilkan laporan taksonomi lengkap (pohon, keanekaragaman, endemik, gap).
///
/// # Arguments
///
/// * `input` - Data takson mentah, data spesies mentah, dan genus referensi.
///
/// # Returns
///
/// Laporan berisi daftar genus endemik, ranking richness, taxa hilang, dan skor cakupan.
pub fn generate_taxonomy_report(input: &BioDataInput) -> Result<ExplorationReport, PipelineError>
```

### 17.3 Visibilitas Fungsi

| Fungsi | Visibilitas | Lapisan | Konsumen |
| --- | --- | --- | --- |
| `parse_taxons`, `parse_species`, `validate_hierarchy` | private | Internal helper (module-specific) | Pipeline |
| `build_taxonomy_map`, `get_lineage`, `get_subtree_species` | private | Internal helper (module-specific) | Pipeline |
| `extract_our_genera`, `filter_endemic_taxa`, `rank_taxon_richness` | private | Internal helper (module-specific) | Pipeline |
| `generate_gap_report` | private | Internal helper (module-specific) | `analyze_taxonomic_gap` |
| `generate_taxonomy_report` | `pub fn` | Module API | Axum handler; opsional modul lain |
| `calculate_diversity` | `pub fn` | Module API | Axum handler; opsional M2/M4 |
| `analyze_taxonomic_gap` | `pub fn` | Module API | Axum handler; opsional M5 |

### 17.4 Komunikasi Antar Modul (Provider → Receiver)

```text
Modul 3 (Provider)
      │
      │ pub fn generate_taxonomy_report(...)
      ▼
Axum handler /api/v1/taxonomy/*  (Receiver / Aplikasi)
      │  hasil: ExplorationReport
      ▼
Django API → Frontend
```

Hubungan opsional (interface publik, bukan dependency crate):

```text
Modul 3 ──DiversityStats────▶ Modul 2 (konteks keanekaragaman, opsional)
Modul 3 ──DiversityStats────▶ Modul 4 (konteks perbandingan, opsional)
Modul 3 ──GapReport─────────▶ Modul 5 (spesies understudied, opsional)
```

| Provider | `pub fn` | Receiver | Purpose | Data yang Dikirim | Priority |
| --- | --- | --- | --- | --- | --- |
| Modul 3 | `generate_taxonomy_report` | Axum handler | Respons API taksonomi | `BioDataInput` | **Required** (API) |
| Modul 3 | `calculate_diversity` | Modul 2/4 | Statistik keanekaragaman sebagai konteks | `DiversityStats` | Optional |
| Modul 3 | `analyze_taxonomic_gap` | Modul 5 | Spesies yang kurang terwakili untuk prioritas riset | `GapReport` | Optional |

> **Selama pengembangan paralel:** Receiver boleh memakai **mock** output Modul 3. Modul 3 tidak pernah bergantung pada fungsi modul lain selain shared library.

### 17.5 Rustdoc — Modul 3

Rustdoc diwajibkan untuk **seluruh `pub fn`** dan **seluruh tipe publik** modul ini:

* `generate_taxonomy_report`, `calculate_diversity`, `analyze_taxonomic_gap`.
* Tipe publik yang muncul di signature `pub fn` (mis. `BioDataInput`, `ExplorationReport`, `DiversityStats`, `GapReport`, `Taxon`, `RawTaxonRecord`).

Command verifikasi:

```bash
cargo doc --workspace --no-deps --open
cargo check
cargo test
```

Status saat ini di dokumen: **Rustdoc planned** (belum diklaim verified).

---

## 18. Rubrik — Evidence Modul 3

| Rubrik | Evidence di Planning Modul 3 | Bukti Implementasi yang Masih Diperlukan |
| --- | --- | --- |
| Repositori Github (15%) | Lokasi `crates/taxonomy` dan command build/test/rustdoc | Repositori dibuat & diakses dosen |
| Prioritas Modul (25%) | Prioritas fitur: parsing → tree → diversity/endemic → coverage/gap | Implementasi fitur prioritas |
| Rustdoc (20%) | Rustdoc diwajibkan untuk semua `pub fn` (Bagian 17.5) | Generate & aksesibel |
| Komunikasi Antar Module (40%) | Batas `mod`/`pub fn` (17.1–17.2), visibilitas (17.3), matriks komunikasi (17.4) | Interface diimplementasikan & diuji |
