# Planning Function — Modul 4: Comparative Species Explorer

**Mata Kuliah:** Pemrograman Fungsional  
**Bahasa:** Rust  
**Cakupan:** Pembangunan sistem perbandingan beberapa spesies di Kalimantan berdasarkan data taksonomi, atribut spesies, status konservasi, serta data observasi dan distribusi geografis untuk mengidentifikasi persamaan, perbedaan, karakteristik unik, dan tingkat kemiripan antarspesies.

**Repository:** Part of KalimantanBio monorepo workspace (`crates/species-comparison`)  
**API Integration:** Production KalimantanBio API (PostgreSQL via shared library)

> **PENTING**: Modul ini menggunakan **Shared Library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk arsitektur lengkap.

> **Catatan integrasi:** Production KalimantanBio database menyediakan data spesies dengan atribut `species_type`, `iucn`, `cites`, taksonomi lengkap (kingdom → genus), serta data observasi per kabupaten. Scope perbandingan disesuaikan dengan data yang benar-benar ada. Akses database melalui shared library.

---

## 1. Tujuan Modul

Modul ini bertujuan untuk membangun sistem perbandingan spesies yang memungkinkan pengguna memilih beberapa spesies di Kalimantan dan melihat persamaan, perbedaan, karakteristik unik, status konservasi, dan pola distribusi geografis berdasarkan data yang tersedia.

Modul ini bertanggung jawab untuk:

* Membandingkan beberapa spesies berdasarkan informasi taksonomi dan atribut spesies yang tersedia.
* Mengidentifikasi atribut yang sama dan atribut yang berbeda atau unik antar spesies.
* Mengidentifikasi karakteristik yang paling membedakan satu spesies dengan spesies lainnya.
* Membandingkan status konservasi (`iucn`, `cites`, `p106`) dan tipe spesies antar spesies.
* Menghitung skor kemiripan antarspesies berdasarkan taksonomi, status konservasi, tipe spesies, dan distribusi observasi.
* Membandingkan distribusi geografis berdasarkan data observasi per kabupaten.
* Menemukan spesies lain yang memiliki tingkat kemiripan tinggi dengan spesies tertentu.
* Menghasilkan hasil perbandingan dalam bentuk data terstruktur yang dapat digunakan untuk visualisasi (tabel, matriks atribut, dsb.).
* Menghasilkan ringkasan tekstual yang menjelaskan persamaan dan perbedaan utama antarspesies.

### Batasan Modul

Modul ini **tidak bertanggung jawab** terhadap:

* Pencarian berbasis bahasa natural atau fuzzy search (tanggung jawab Modul 1).
* Visualisasi jaringan/graf relasi antar-spesies (tanggung jawab Modul 2).
* Pembangunan pohon taksonomi hirarkis dan analisis taxonomic gap (tanggung jawab Modul 3).
* Eksplorasi publikasi ilmiah dan manajemen sitasi (tanggung jawab Modul 5).
* Menyimpan atau mengubah data spesies — modul bersifat read-only terhadap data.
* Inferensi ekologis di luar data yang tersedia (mis. hubungan predator-mangsa).

---

## 2. Shared Library Integration

Modul ini menggunakan **shared library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum yang digunakan bersama dengan modul lain.

### 2.1 Shared Types Used

```rust
use kalimantanbio_shared::core::{Species, Taxonomy, Observation};
```

- `Species` - Struktur data lengkap spesies dari production database
- `Taxonomy` - Struktur hierarki taksonomi (kingdom → genus)
- `Observation` - Data observasi per kabupaten (distribusi geografis)

### 2.2 Shared Functions Used

#### From `taxonomy` module:
```rust
use kalimantanbio_shared::taxonomy::{build_taxonomy_path, calculate_taxonomy_similarity};
```

- `build_taxonomy_path(taxonomy: &Taxonomy) -> String` - Menyusun string path taksonomi `"Kingdom > Phylum > Class > Order > Family > Genus"`
- `calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64` - Menghitung skor kesamaan taksonomi (0.0–1.0)

**Digunakan di**: Tahap 2 (Attribute Matrix), Tahap 4 (Similarity Scoring)

#### From `collections` module:
```rust
use kalimantanbio_shared::collections::jaccard_similarity;
```

- `jaccard_similarity<T>(a: &HashSet<T>, b: &HashSet<T>) -> f64` - Menghitung kesamaan set (untuk distribusi kabupaten)

**Digunakan di**: Tahap 4 (Similarity Scoring)

#### From `scoring` module:
```rust
use kalimantanbio_shared::scoring::{combine_weighted_scores, rank_by_score, top_n};
```

- `combine_weighted_scores(scores: &[(f64, f64)]) -> f64` - Menggabungkan skor dengan bobot
- `rank_by_score<T>(items: Vec<(T, f64)>) -> Vec<(T, f64)>` - Mengurutkan skor menurun
- `top_n<T>(items: Vec<(T, f64)>, n: usize) -> Vec<(T, f64)>` - Mengambil N tertinggi

**Digunakan di**: Tahap 4 (Similarity Scoring), Tahap 5 (Summary Generation)

#### From `validation` module:
```rust
use kalimantanbio_shared::validation::validate_id_list;
```

- `validate_id_list(ids: &[u64], min: usize, max: usize) -> Result<()>` - Memvalidasi jumlah ID minimal/maksimal, no duplikat

**Digunakan di**: Tahap 1 (Data Validation)

#### From `db` module:
```rust
use kalimantanbio_shared::db::{create_pool, fetch_all_species, fetch_observations_for_species};
```

- `create_pool(database_url: &str) -> Result<Pool<Postgres>>` - Membuat connection pool
- `fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>>` - Fetch semua spesies
- `fetch_observations_for_species(pool: &Pool<Postgres>, species_id: u64) -> Result<Vec<Observation>>` - Fetch observasi per spesies

**Digunakan di**: Tahap 5 (Integrasi Pipeline & Testing)

### 2.3 Module-Specific Types

Modul ini mendefinisikan tipe tambahan yang spesifik untuk perbandingan:

```rust
#[derive(Debug, Clone, Default)]
struct ComparisonQuery {
    species_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
struct ScoringWeights {
    taxonomy: f64,
    conservation: f64,
    species_type: f64,
    distribution: f64,
}

#[derive(Debug, Clone)]
struct SimilarityScore {
    species_a_id: u64,
    species_b_id: u64,
    taxonomy_score: f64,
    conservation_score: f64,
    type_score: f64,
    distribution_score: f64,
    total_score: f64,
}

#[derive(Debug, Clone)]
struct SpeciesAttributeRow {
    species_id: u64,
    scientific_name: String,
    common_name: String,
    taxonomy_path: String,
    species_type: String,
    iucn: String,
    cites: String,
    p106: String,
    observation_count: u32,
    kabupaten_list: Vec<String>,
    is_verified: bool,
}

#[derive(Debug, Clone, Default)]
struct ComparisonResult {
    query: ComparisonQuery,
    attribute_matrix: Vec<SpeciesAttributeRow>,
    shared_attributes: Vec<String>,
    unique_attributes: Vec<String>,
    distinguishing_characteristics: Vec<String>,
    similarity_scores: Vec<SimilarityScore>,
    summary: String,
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
    core::{Species, Taxonomy, Observation},
    taxonomy::{build_taxonomy_path, calculate_taxonomy_similarity},
    collections::jaccard_similarity,
    scoring::combine_weighted_scores,
    validation::validate_id_list,
};
use std::collections::HashSet;

pub fn calculate_pair_similarity(
    a: &Species,
    b: &Species,
    kabupaten_a: &HashSet<u64>,
    kabupaten_b: &HashSet<u64>,
    weights: &ScoringWeights,
) -> SimilarityScore {
    // Validasi jumlah spesies via shared library
    // (dipanggil sekali di level query)

    // Skor taksonomi via shared library
    let taxonomy_score = calculate_taxonomy_similarity(&a.taxonomy, &b.taxonomy);

    // Skor distribusi via shared library (Jaccard)
    let distribution_score = jaccard_similarity(kabupaten_a, kabupaten_b);

    // Gabungkan skor via shared library
    let total = combine_weighted_scores(&[
        (taxonomy_score, weights.taxonomy),
        (conservation_score, weights.conservation),
        (type_score, weights.species_type),
        (distribution_score, weights.distribution),
    ]);

    SimilarityScore {
        species_a_id: a.id,
        species_b_id: b.id,
        taxonomy_score,
        conservation_score,
        type_score,
        distribution_score,
        total_score: total,
    }
}
```

---

## 3. Struktur Data (Domain Model)

### 3.1 Tipe dari Shared Library

Modul ini menggunakan `Species`, `Taxonomy`, dan `Observation` dari shared library. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk definisi lengkap.

> **Catatan:** Struktur data sudah distandardisasi di shared library dan digunakan oleh semua modul. Tidak boleh ada modifikasi pada tipe ini tanpa koordinasi dengan project lead.

### 3.2 Module-Specific Types

Tipe berikut spesifik untuk Modul 4 (`ComparisonQuery`, `ScoringWeights`, `SimilarityScore`, `SpeciesAttributeRow`, `ComparisonResult`) sudah didefinisikan di bagian 2.3.

> **Catatan:** `taxonomy_score` menggunakan level kesamaan takson dari shared library. Total bobot `ScoringWeights` harus = 1.0.

### Data yang Digunakan

| Data | Tipe | Deskripsi |
| --------- | -------- | ----------- |
| `scientific_name`, `common_name`, `description` | `String` | Informasi identitas dan deskripsi spesies yang digunakan sebagai atribut pembanding. |
| `species_type` | `String` | Menunjukkan kategori spesies, seperti Endemic, Native, atau Introduced. |
| `iucn`, `cites`, `p106` | `String` | Informasi status konservasi spesies berdasarkan status IUCN, CITES, dan perlindungan nasional. |
| `taxonomy` | `Taxonomy` | Informasi klasifikasi taksonomi spesies, meliputi kingdom, phylum/division, class, order, family, dan genus. |
| `is_verified` | `bool` | Menunjukkan apakah data spesies telah diverifikasi. |
| `observation_count` | `u32` | Jumlah observasi yang tercatat untuk suatu spesies. |
| `recorded_individuals_total` | `u32` | Total individu spesies yang tercatat dalam data observasi. |
| `latest_observation_year` | `Option<u32>` | Tahun observasi terbaru yang tersedia untuk suatu spesies. |
| `Observation.kabupaten_id`, `kabupaten_name` | `u64` / `String` | Data distribusi kabupaten untuk perbandingan geografis antar spesies. |

---

## 4. Tahap 1 — Data Retrieval & Validation

Mengambil data spesies dari source data (production database via shared library atau fixture), memvalidasi input query pengguna, dan menyiapkan data yang siap diproses oleh tahap berikutnya.

| Fungsi | Signature | Deskripsi |
| -------------- | ----------------------------- | ------------------ |
| `validate_query` | `fn validate_query(query: &ComparisonQuery) -> Result<(), QueryError>` | **Uses SHARED `validate_id_list`**: Memvalidasi query: minimal 2 ID spesies, tidak ada duplikasi, tidak melebihi batas maksimal yang disepakati. Mengembalikan `QueryError::TooFewSpecies`, `QueryError::DuplicateId`, atau `QueryError::TooManySpecies` sesuai kondisi. |
| `retrieve_species` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::db::fetch_all_species` | Mengambil seluruh data spesies dari production database (atau fixture). |
| `fetch_species_by_ids` | `fn fetch_species_by_ids<'a>(ids: &[u64], all_species: &'a [Species]) -> Result<Vec<&'a Species>, DataError>` | **MODULE-SPECIFIC**: Mengambil spesies berdasarkan ID dari slice data yang tersedia menggunakan `.filter()`. Mengembalikan `DataError::SpeciesNotFound(id)` jika ada ID yang tidak ditemukan. |
| `fetch_observations_for_species` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::db::fetch_observations_for_species` | Mengambil semua data observasi untuk satu spesies. Mengembalikan slice kosong jika tidak ada observasi. |
| `build_kabupaten_set` | `fn build_kabupaten_set(observations: &[&Observation]) -> std::collections::HashSet<u64>` | **MODULE-SPECIFIC**: Mengekstrak set unik `kabupaten_id` dari observasi suatu spesies menggunakan `.map().collect()`. |

**Person in Charge:** **Anggota 1 — Data retrieval & query validation (nama menyusul).**

---

## 5. Tahap 2 — Attribute Matrix Construction

Membangun matriks atribut per spesies sebagai representasi terstruktur dari setiap spesies yang dibandingkan, termasuk menyusun taxonomy path yang mudah dibaca.

| Fungsi | Signature | Deskripsi |
| -------------- | ----------------------------- | ------------------ |
| `build_taxonomy_path` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::taxonomy::build_taxonomy_path` | Menyusun string path taksonomi menggunakan format `"Kingdom > Phylum > Class > Order > Family > Genus"` (mis. `"Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo"`). |
| `build_attribute_row` | `fn build_attribute_row(species: &Species, observations: &[&Observation]) -> SpeciesAttributeRow` | **MODULE-SPECIFIC**: Membangun satu baris atribut spesies termasuk taxonomy path (via shared library) dan daftar nama kabupaten observasi unik. |
| `build_attribute_matrix` | `fn build_attribute_matrix(species_list: &[&Species], all_observations: &[Observation]) -> Vec<SpeciesAttributeRow>` | **MODULE-SPECIFIC**: Menerapkan `build_attribute_row` ke setiap spesies menggunakan `.map().collect()` untuk menghasilkan matriks atribut lengkap. |
| `extract_attribute_values` | `fn extract_attribute_values(matrix: &[SpeciesAttributeRow], key: &str) -> Vec<String>` | **MODULE-SPECIFIC**: Mengekstrak nilai satu atribut tertentu (mis. `"iucn"`, `"species_type"`) dari seluruh baris matriks, untuk digunakan pada analisis kesamaan dan perbedaan. |

**Person in Charge:** **Anggota 2 — Attribute matrix construction & extraction (nama menyusul).**

---

## 6. Tahap 3 — Shared & Unique Attribute Analysis

Menganalisis matriks atribut untuk menemukan atribut yang sama di semua spesies, atribut yang unik per spesies, dan karakteristik paling membedakan.

| Fungsi | Signature | Deskripsi |
| -------------- | ----------------------------- | ------------------ |
| `is_attribute_shared` | `fn is_attribute_shared(values: &[String]) -> bool` | Pure function: mengembalikan `true` jika semua nilai dalam slice identik (tidak kosong). Diimplementasikan menggunakan `.windows(2).all(|w| w[0] == w[1])`. |
| `find_shared_attributes` | `fn find_shared_attributes(matrix: &[SpeciesAttributeRow]) -> Vec<String>` | Menemukan atribut (taksonomi, iucn, cites, p106, species_type) yang bernilai sama pada semua spesies menggunakan `extract_attribute_values` dan `is_attribute_shared`. Hasilnya dalam format `"key=value"` (mis. `"kingdom=Animalia"`, `"iucn=CR"`). |
| `find_unique_attributes` | `fn find_unique_attributes(matrix: &[SpeciesAttributeRow]) -> Vec<String>` | Menemukan atribut yang nilainya hanya dimiliki tepat satu spesies dalam kumpulan yang dibandingkan. Hasilnya dalam format `"species_id:key=value"` (mis. `"123:species_type=Endemic"`). |
| `find_distinguishing_characteristics` | `fn find_distinguishing_characteristics(matrix: &[SpeciesAttributeRow]) -> Vec<String>` | Mengidentifikasi atribut dengan variasi terbesar antarspesies — yaitu atribut paling membedakan. Diprioritaskan: iucn > species_type > family > order > class > phylum. Hasilnya berupa deskripsi teks (mis. `"iucn: CR vs LC vs VU"`). |

**Person in Charge:** **Anggota 3 — Shared/unique attribute & distinguishing analysis (nama menyusul).**

---

## 7. Tahap 4 — Similarity Scoring

Menghitung skor kemiripan untuk setiap pasang spesies yang dibandingkan berdasarkan empat dimensi: taksonomi, konservasi, tipe spesies, dan distribusi geografis.

> **Catatan**: Fungsi `calculate_taxonomy_similarity`, `jaccard_similarity`, `combine_weighted_scores`, dan `rank_by_score` menggunakan **shared library**.

| Fungsi | Signature | Deskripsi |
| -------------- | ----------------------------- | ------------------ |
| `score_taxonomy` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::taxonomy::calculate_taxonomy_similarity` | Menghitung skor taksonomi (0.0–1.0) berdasarkan level takson tertinggi yang sama. Skala dan formula ada di shared library. |
| `score_conservation` | `fn score_conservation(a: &Species, b: &Species) -> f64` | **MODULE-SPECIFIC**: Menghitung rata-rata kesamaan tiga atribut konservasi (`iucn`, `cites`, `p106`). Tiap atribut: 1.0 jika identik, 0.5 jika salah satu bernilai `"-"`, 0.0 jika berbeda. Rata-rata dari ketiga nilai. |
| `score_species_type` | `fn score_species_type(a: &Species, b: &Species) -> f64` | **MODULE-SPECIFIC**: Mengembalikan 1.0 jika `species_type` identik (case-insensitive), 0.0 jika berbeda. |
| `score_distribution` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::collections::jaccard_similarity` | Menghitung Jaccard Similarity dari set kabupaten observasi: `\|intersection\| / \|union\|`. Mengembalikan 0.0 jika salah satu atau keduanya kosong (menghindari division by zero). |
| `calculate_similarity_score` | **Uses SHARED `combine_weighted_scores`**: `fn calculate_similarity_score(a: &Species, b: &Species, obs_a: &std::collections::HashSet<u64>, obs_b: &std::collections::HashSet<u64>, weights: &ScoringWeights) -> SimilarityScore` | Menggabungkan keempat skor dengan bobot dari `weights` menjadi `total_score` menggunakan shared library. |
| `calculate_all_pairs` | `fn calculate_all_pairs(species_list: &[&Species], observations_map: &std::collections::HashMap<u64, std::collections::HashSet<u64>>, weights: &ScoringWeights) -> Vec<SimilarityScore>` | **MODULE-SPECIFIC**: Menghasilkan semua kombinasi pasang spesies (n*(n-1)/2) dan menghitung skor tiap pasang. Menggunakan nested iterator dengan `.enumerate().flat_map()`. |
| `find_most_similar` | **Uses SHARED `rank_by_score` / `top_n`**: `fn find_most_similar(target_id: u64, scores: &[SimilarityScore]) -> Option<SimilarityScore>` | Menemukan pasang dengan `total_score` tertinggi yang melibatkan `target_id` menggunakan `rank_by_score`/`max_by`. Mengembalikan `None` jika tidak ada pasang yang ditemukan. |

**Person in Charge:** **Anggota 4 — Similarity scoring & ranking (nama menyusul).**

---

## 8. Tahap 5 — Summary Generation & Integration Pipeline

Menggabungkan seluruh output tahap sebelumnya menjadi `ComparisonResult` lengkap dan menghasilkan ringkasan teks yang menjelaskan persamaan dan perbedaan utama antarspesies. Data spesies dan observasi dibaca dari production database melalui shared library.

| Fungsi / Komponen | Signature / Bentuk | Deskripsi |
| ----------------- | ------------------ | ----------- |
| `generate_summary` | `fn generate_summary(result: &ComparisonResult) -> String` | **MODULE-SPECIFIC**: Menghasilkan ringkasan teks perbandingan dari `shared_attributes`, `unique_attributes`, `distinguishing_characteristics`, dan `similarity_scores` tertinggi. Pure function, tidak melakukan I/O. |
| `compare_species` | `fn compare_species(query: &ComparisonQuery, all_species: &[Species], all_observations: &[Observation], weights: &ScoringWeights) -> Result<ComparisonResult, ComparisonError>` | **MODULE ENTRY POINT**: Entry point pipeline utama: merangkai Tahap 1–4 menjadi `ComparisonResult` lengkap. |
| `fetch_all_species` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::db::fetch_all_species` | Fetch semua spesies dari production database. |
| `fetch_observations_for_species` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::db::fetch_observations_for_species` | Fetch observasi per spesies dari production database. |
| Unit Tests | `mod tests { ... }` | Menguji setiap fungsi dari Tahap 1–4 secara independen menggunakan data dummy `Vec<Species>` dan `Vec<Observation>`. |
| Pipeline Validation | `cargo test` | Menjalankan seluruh skenario pengujian secara otomatis termasuk edge cases. |

**Person in Charge:** **Semua anggota — summary generation, pipeline validation, dan end-to-end testing (koordinasi Anggota 1).**

> **Catatan:** Setiap PIC tetap bertanggung jawab terhadap pengujian fungsi yang mereka implementasikan. PIC tahap ini berfokus pada pengujian antar-komponen dan pengujian end-to-end.

---

## 9. Komposisi Pipeline Utama

Setelah seluruh tahap tersedia, fungsi utama modul menggabungkan proses menjadi satu pipeline.

```rust
use kalimantanbio_shared::{
    taxonomy::calculate_taxonomy_similarity,
    collections::jaccard_similarity,
    scoring::combine_weighted_scores,
};

fn compare_species(
    query: &ComparisonQuery,
    all_species: &[Species],
    all_observations: &[Observation],
    weights: &ScoringWeights,
) -> Result<ComparisonResult, ComparisonError> {
    validate_query(query)?;   // module-specific wrapper (uses SHARED validate_id_list)
    let species_list = fetch_species_by_ids(&query.species_ids, all_species)?;
    let observations_map: HashMap<u64, HashSet<u64>> = species_list
        .iter()
        .map(|s| {
            let obs = fetch_observations_for_species(s.id, all_observations);
            let kabupaten_set = build_kabupaten_set(&obs);
            (s.id, kabupaten_set)
        })
        .collect();

    let attribute_matrix = build_attribute_matrix(&species_list, all_observations);

    let shared_attributes = find_shared_attributes(&attribute_matrix);
    let unique_attributes = find_unique_attributes(&attribute_matrix);
    let distinguishing_characteristics = find_distinguishing_characteristics(&attribute_matrix);

    let similarity_scores = calculate_all_pairs(&species_list, &observations_map, weights);

    let mut result = ComparisonResult {
        query: query.clone(),
        attribute_matrix,
        shared_attributes,
        unique_attributes,
        distinguishing_characteristics,
        similarity_scores,
        summary: String::new(),
    };
    result.summary = generate_summary(&result);

    Ok(result)
}
```

Pipeline konseptual:

```text
ComparisonQuery (species_ids)
  |
[Tahap 1: Data Retrieval & Validation]
  | Vec<&Species> + HashMap<u64, HashSet<u64>>
[Tahap 2: Attribute Matrix Construction]
  | Vec<SpeciesAttributeRow>
[Tahap 3: Shared & Unique Attribute Analysis]
  | shared_attributes, unique_attributes, distinguishing_characteristics
[Tahap 4: Similarity Scoring]
  | Vec<SimilarityScore>
[Tahap 5: Summary Generation & Assembly]
  |
ComparisonResult (output lengkap)
```

Fungsi `compare_species` adalah **entry point** modul dan menjadi contoh penerapan **function composition** — menggabungkan fungsi-fungsi shared library dan fungsi module-specific menjadi satu proses besar.

---

## 10. Prinsip Functional Programming yang Perlu Dipegang Tim

### Pure Functions

Fungsi sebaiknya tidak mengubah state eksternal dan hanya bergantung pada input yang diberikan.

```text
Input -> Function -> Output
```

Untuk input yang sama, fungsi idealnya menghasilkan output yang sama. `compare_species` merupakan pengecualian wajar jika melibatkan I/O (network call ke API) — sebaiknya dipisahkan tegas dari fungsi-fungsi murni lainnya di pipeline.

### Immutability

Hindari memodifikasi data input secara langsung.

Gunakan reference/borrow seperti `&T` ketika data hanya perlu dibaca dan hasil transformasi dikembalikan sebagai data baru apabila diperlukan.

### Higher-Order Functions

Manfaatkan iterator dan fungsi seperti:

```rust
.map()
.filter()
.fold()
.flat_map()
.all()
.any()
.max_by()
.collect()
```

untuk melakukan transformasi, filtering, dan agregasi data. Hindari `for` loop manual di logika inti analitik.

### Function Composition

Pecah proses utama menjadi fungsi-fungsi kecil yang dapat dikombinasikan.

```text
validate_query
    |
fetch_species_by_ids
    |
build_attribute_matrix
    |
find_shared_attributes / find_unique_attributes / find_distinguishing_characteristics
    |
calculate_all_pairs
    |
generate_summary
```

Setiap fungsi sebaiknya memiliki satu tanggung jawab yang jelas dan dapat diuji secara independen.

---

## 11. Pembagian Kerja — 4 Anggota

| # | Tahap | Fungsi / Tanggung Jawab Utama | PIC | Status |
| - | --------- | ------------------------------------- | --- | ------------- |
| 1 | Tahap 1: Data Retrieval & Validation | `validate_query` (via shared `validate_id_list`), `fetch_species_by_ids`, `build_kabupaten_set` (data via shared `fetch_all_species`) | Anggota 1 | Belum dimulai |
| 2 | Tahap 2: Attribute Matrix Construction | `build_attribute_row`, `build_attribute_matrix`, `extract_attribute_values` (taxonomy path via shared `build_taxonomy_path`) | Anggota 2 | Belum dimulai |
| 3 | Tahap 3: Shared & Unique Attribute Analysis | `is_attribute_shared`, `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics` | Anggota 3 | Belum dimulai |
| 4 | Tahap 4: Similarity Scoring | `score_conservation`, `score_species_type`, `calculate_all_pairs`, `find_most_similar` (via shared `calculate_taxonomy_similarity`, `jaccard_similarity`, `combine_weighted_scores`) | Anggota 4 | Belum dimulai |
| 5 | Tahap 5: Integration & Testing | `generate_summary`, `compare_species`, unit tests, pipeline validation — **dikerjakan bersama seluruh anggota** (koordinasi Anggota 1) | Seluruh anggota | Belum dimulai |

> **Catatan:** Fungsi `build_taxonomy_path`, `calculate_taxonomy_similarity`, `jaccard_similarity`, `combine_weighted_scores`, `rank_by_score`, `validate_id_list`, `fetch_all_species`, dan `fetch_observations_for_species` berasal dari **shared library** (`kalimantanbio-shared`) dan **tidak perlu diimplementasikan** di modul ini.

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

* Struktur `Species`, `Taxonomy`, `Observation` (dari shared library) serta tipe module-specific `ComparisonQuery`, `ScoringWeights`, `SimilarityScore`, dan `ComparisonResult`.
* Arti setiap field, terutama field yang berasal dari singkatan (`iucn`, `cites`, `p106`).
* Jumlah maksimal spesies yang dapat dibandingkan sekaligus dalam satu `ComparisonQuery`.
* Input dan output setiap fungsi, termasuk format string `shared_attributes` dan `unique_attributes`.
* Function signature final untuk setiap tahap.
* Formula dan bobot scoring pada `ScoringWeights` (default: taxonomy=0.40, conservation=0.30, type=0.15, distribution=0.15).
* Formula `score_taxonomy` menggunakan shared library (`calculate_taxonomy_similarity`).
* Formula `score_distribution`: Jaccard Similarity dari set kabupaten observasi (shared library).
* Format `shared_attributes` (mis. `"iucn=CR"`) dan `unique_attributes` (mis. `"123:species_type=Endemic"`).
* Format output `summary` (teks bebas atau template terstruktur).
* Strategi testing (unit test per fungsi dengan data dummy, tanpa koneksi DB aktif).

Tujuannya adalah memastikan fungsi yang dikembangkan oleh anggota berbeda tetap dapat dikombinasikan tanpa perubahan besar pada interface masing-masing.

---

## 13. Independensi Modul

Modul ini dikembangkan sebagai komponen independen dalam KalimantanBio.

Prinsip yang digunakan:

* Modul dapat dikembangkan secara mandiri, terlepas dari status Modul 1, 2, 3, dan 5.
* Modul dapat diuji secara mandiri menggunakan data dummy `Vec<Species>` dan `Vec<Observation>`, tanpa harus terkoneksi ke database.
* Modul hanya bergantung pada **shared library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum.
* Gunakan shared concepts (`Species`, `Taxonomy`) sebagai data convention yang disetujui bersama.
* Integrasi dengan modul lain bersifat opsional — misalnya hasil pencarian Modul 1 dapat digunakan sebagai input `species_ids` ke Modul 4 melalui interface publik (`pub fn`) yang dikomposisikan di lapisan aplikasi (lihat Bagian 17.4).
* Jangan mengasumsikan dependency terhadap modul lain tanpa kebutuhan teknis yang jelas.

```text
          KalimantanBio
               │
               │  ((shared))  kalimantanbio-shared
               │
          ┌────┴────┐
          │         │
     Modul ini    Modul lain
       (M4)       (M1, M2, M3, M5)
```

Modul ini (M4) hanya bergantung pada shared library, bukan pada modul lain.

Diagram di atas menggambarkan hubungan **konseptual**, bukan dependency teknis.

---

## 14. Kriteria Selesai Modul

Modul dianggap siap untuk tahap akhir apabila:

* [ ] Seluruh fungsi utama telah diimplementasikan.
* [ ] Fungsi-fungsi dari **shared library** digunakan, bukan diduplikasi.
* [ ] Setiap fungsi memiliki unit test yang relevan.
* [ ] Pipeline `compare_species()` dapat berjalan end-to-end dengan data dummy maupun data dari database (melalui shared library).
* [ ] Input `ComparisonQuery` dapat diproses sesuai spesifikasi, termasuk validasi jumlah spesies.
* [ ] Output menghasilkan `ComparisonResult` dengan semua field terisi sesuai format yang disepakati.
* [ ] `SimilarityScore` menghasilkan nilai dalam rentang 0.0-1.0 untuk semua dimensi dan total skor.
* [ ] Tidak terdapat dependency yang tidak diperlukan (tidak bergantung langsung ke Modul 1, 2, 3, atau 5).
* [ ] Tidak terdapat state global yang tidak diperlukan.
* [ ] Dokumentasi fungsi dan struktur data tersedia.
* [ ] Interface publik (`pub fn compare_species`) didefinisikan jelas dan teruji — lihat Bagian 17.
* [ ] Seluruh `pub fn` memiliki dokumentasi rustdoc lengkap.
* [ ] Terdapat demonstrasi penggunaan modul (contoh query dengan 2-3 spesies dan hasilnya).
* [ ] Modul dapat dijalankan secara independen.

---

## 15. Contoh Skenario Pengujian

### Skenario 1 — Membandingkan Dua Spesies Orangutan

**Input:**

```text
ComparisonQuery { species_ids: [1, 2] }
Spesies 1: Pongo pygmaeus (Orangutan Kalimantan, Endemic, iucn=CR, cites="Appendix I", p106="Dilindungi")
Spesies 2: Pongo abelii   (Orangutan Sumatera, Native,   iucn=CR, cites="Appendix I", p106="Dilindungi")
```

**Expected Output:**

```text
ComparisonResult {
  shared_attributes: [
    "kingdom=Animalia", "iucn=CR", "cites=Appendix I",
    "p106=Dilindungi", "taxonomy.genus=Pongo"
  ],
  unique_attributes: [
    "1:common_name=Orangutan Kalimantan", "1:species_type=Endemic",
    "2:common_name=Orangutan Sumatera",   "2:species_type=Native"
  ],
  distinguishing_characteristics: [
    "species_type: Endemic vs Native",
    "Distribusi kabupaten observasi berbeda"
  ],
  similarity_scores: [
    SimilarityScore {
      species_a_id: 1, species_b_id: 2,
      taxonomy_score: 1.0,     // genus sama (Pongo)
      conservation_score: 1.0, // iucn=CR, cites=Appendix I, p106=Dilindungi semua sama
      type_score: 0.0,         // Endemic != Native
      distribution_score: 0.2, // sedikit overlap kabupaten observasi (contoh)
      total_score: 0.79,       // 1.0*0.40 + 1.0*0.30 + 0.0*0.15 + 0.2*0.15
    }
  ],
  summary: "Kedua spesies termasuk genus yang sama (Pongo) dengan status konservasi identik (IUCN CR, CITES Appendix I, Dilindungi). Perbedaan utama: tipe spesies (Endemic vs Native) dan distribusi observasi yang berbeda."
}
```

**Fungsi yang diuji:**

* `validate_query` (via `validate_id_list` shared), `fetch_species_by_ids`
* `build_attribute_row` (via `build_taxonomy_path` shared), `build_attribute_matrix`
* `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics`
* `score_conservation`, `score_species_type`
* `calculate_similarity_score` (via `calculate_taxonomy_similarity`, `jaccard_similarity`, `combine_weighted_scores` shared), `generate_summary`

---

### Skenario 2 — Membandingkan Tiga Spesies dari Kingdom Berbeda

**Input:**

```text
ComparisonQuery { species_ids: [10, 20, 30] }
Spesies 10: Rafflesia arnoldii  (Plantae, Endemic, iucn=CR)
Spesies 20: Pongo pygmaeus      (Animalia, Endemic, iucn=CR)
Spesies 30: Agathis borneensis  (Plantae, Endemic, iucn=EN)
```

**Expected Output:**

```text
ComparisonResult {
  shared_attributes: ["species_type=Endemic"],
  unique_attributes: [
    "10:kingdom=Plantae", "10:iucn=CR",
    "20:kingdom=Animalia", "20:iucn=CR",
    "30:kingdom=Plantae", "30:iucn=EN"
  ],
  distinguishing_characteristics: [
    "kingdom: Plantae vs Animalia vs Plantae",
    "iucn: CR vs CR vs EN"
  ],
  similarity_scores: [
    // Pasang 10-20: taxonomy_score=0.0 (kingdom berbeda), conservation beda
    // Pasang 10-30: taxonomy_score=0.2 (phylum sama: Tracheophyta)
    // Pasang 20-30: taxonomy_score=0.0 (kingdom berbeda)
    ...
  ],
  summary: "Ketiga spesies sama-sama berstatus Endemic. Spesies 10 dan 30 berasal dari Kingdom Plantae sedangkan Spesies 20 dari Animalia. Perbedaan utama terletak pada kingdom dan status IUCN."
}
```

**Fungsi yang diuji:**

* `calculate_all_pairs` (3 kombinasi pasang: 10-20, 10-30, 20-30)
* `find_shared_attributes` dengan 3 baris matriks
* `find_distinguishing_characteristics`

---

### Skenario 3 — Menemukan Spesies Paling Mirip dengan Target

**Input:**

```text
target_id: 1
scores: [semua SimilarityScore dari dataset]
```

**Expected Output:**

```text
Some(SimilarityScore { ... total_score: <nilai tertinggi yang melibatkan id=1> })
```

**Fungsi yang diuji:**

* `find_most_similar`

---

### Edge Cases

| Case | Input | Expected Result |
| --- | --- | --- |
| Hanya 1 ID spesies | `species_ids: [1]` | `validate_query` mengembalikan `Err(QueryError::TooFewSpecies)`. |
| ID spesies tidak ditemukan | `species_ids: [1, 99999]` | `fetch_species_by_ids` mengembalikan `Err(DataError::SpeciesNotFound(99999))`. |
| ID duplikat | `species_ids: [1, 1]` | `validate_query` mengembalikan `Err(QueryError::DuplicateId)`. |
| Spesies tanpa data observasi | Spesies dengan `observation_count=0` | `build_kabupaten_set` mengembalikan `HashSet` kosong; `score_distribution` mengembalikan `0.0` tanpa panic. |
| Semua spesies identik | Dua spesies dengan semua atribut identik | `shared_attributes` berisi semua atribut; `unique_attributes` kosong; `total_score = 1.0`. |
| Field opsional null | `latest_observation_year: None` | Pipeline tetap berjalan tanpa panic, menggunakan `Option<T>` dan `.unwrap_or`. |
| Melebihi batas maksimal | `species_ids: [1,2,3,4,5,6]` (jika max=5) | `validate_query` mengembalikan `Err(QueryError::TooManySpecies)`. |
| Satu spesies tanpa observasi, satu ada | obs_a kosong, obs_b tidak kosong | `score_distribution` mengembalikan `0.0` (union tidak kosong, intersection kosong). |

---

## 16. Langkah Selanjutnya

1. Verifikasi bahwa **shared library** (`kalimantanbio-shared`) sudah tersedia; jika belum, blokir pengerjaan sampai siap.
2. Finalisasi domain model bersama project lead (tipe `Species`, `Taxonomy`, `Observation` sudah standar).
3. Sepakati nilai default bobot scoring dan formula untuk setiap dimensi skor.
4. Sepakati batas maksimal spesies yang dapat dibandingkan sekaligus dalam satu `ComparisonQuery`.
5. Sepakati format string untuk `shared_attributes`, `unique_attributes`, dan `distinguishing_characteristics`.
6. Tentukan pembagian fungsi berdasarkan PIC (isi tabel Bagian 11).
7. Setiap PIC membuat signature dan dokumentasi singkat fungsi masing-masing.
8. Review interface bersama sebelum implementasi logic dimulai.
9. Siapkan data dummy `Vec<Species>` dan `Vec<Observation>` (independen dari DB) untuk unit test.
10. Implementasikan fungsi secara paralel sesuai pembagian kerja; gunakan fungsi shared library.
11. Setiap PIC membuat unit test untuk fungsi masing-masing.
12. Gabungkan seluruh tahap ke dalam pipeline `compare_species()`.
13. Lakukan pengujian end-to-end menggunakan data dummy dan kemudian data production (via shared library).
14. Dokumentasikan hasil dan contoh penggunaan modul.
15. Review akhir sebelum modul dianggap selesai, termasuk kelengkapan rustdoc pada seluruh `pub fn`.

---

## 17. Interface Publik & Komunikasi Antar Modul

Bagian ini menjelaskan batas `mod` dan `pub fn` Modul 4 sebagai bukti pemenuhan aspek **Komunikasi Antar Module** pada rubrik penilaian (40%).

### 17.1 Batas Modul (Owns / Does Not Own / Internal / Public)

| Aspek | Isi |
| --- | --- |
| **Owns** | Validasi query perbandingan, konstruksi matriks atribut, analisis shared/unique/distinguishing, similarity scoring multi-dimensi, dan ringkasan perbandingan. |
| **Does Not Own** | Data spesies/observasi/taksonomi (shared library), pencarian spesies (M1), relasi antarspesies (M2), analisis taksonomi mendalam (M3), publikasi (M5). |
| **Internal** | `fetch_species_by_ids`, `fetch_observations_for_species`, `build_attribute_row`, `build_attribute_matrix`, `extract_attribute_values`, `is_attribute_shared`, `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics`, `best_score`, `score_conservation`, `score_species_type`, `score_distribution`, `calculate_all_pairs`, `find_most_similar`, `generate_summary`. |
| **Publicly Exposes** | `compare_species` (main) dan `calculate_pair_similarity` (pasangan, opsional) — kapabilitas yang dapat digunakan Axum handler maupun modul lain. |

### 17.2 Fungsi Publik (`pub fn`) — Modul 4

| `pub fn` | Provider | Consumer Potensial | Purpose | Input | Output | Why Needed |
| --- | --- | --- | --- | --- | --- | --- |
| `compare_species` | Modul 4 | Axum handler `/api/v1/compare` | Pipeline utama Tahap 1–5 | `&ComparisonQuery`, globals | `Result<ComparisonResult, ModuleError>` | Kapabilitas utama modul untuk aplikasi dan modul lain. |
| `calculate_pair_similarity` *(sudah `pub` di contoh)* | Modul 4 | Internal `calculate_all_pairs`; opsional publik untuk modul lain | Menghitung skor kesamaan sepasang spesies | dua spesies + observasi | `SimilarityScore` | Hitungan inti yang dapat diuji dan digunakan ulang. |

```rust
/// Membandingkan beberapa spesies sekaligus.
///
/// # Arguments
///
/// * `query` - Query perbandingan (daftar species_ids yang sudah divalidasi).
/// * `fetch_species` / `fetch_observations` - Adapter data (dari shared library).
///
/// # Returns
///
/// Atribut bersama, atribut unik, karakteristik pembeda, skor kesamaan, dan ringkasan.
pub fn compare_species(
    query: &ComparisonQuery,
    species: &[Species],
    observations: &[Observation],
) -> Result<ComparisonResult, ModuleError>
```

### 17.3 Visibilitas Fungsi

| Fungsi | Visibilitas | Lapisan | Konsumen |
| --- | --- | --- | --- |
| `fetch_species_by_ids`, `fetch_observations_for_species`, `build_attribute_row`, `build_attribute_matrix`, `extract_attribute_values` | private | Internal helper (module-specific) | Pipeline |
| `is_attribute_shared`, `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics` | private | Internal helper (module-specific) | Pipeline |
| `best_score`, `score_conservation`, `score_species_type`, `score_distribution` | private | Internal helper (module-specific) | `calculate_pair_similarity` |
| `calculate_all_pairs`, `find_most_similar`, `generate_summary` | private | Internal helper (module-specific) | Pipeline |
| `calculate_pair_similarity` | `pub fn` | Module API (opsional) | Internal; opsional modul lain |
| `compare_species` | `pub fn` | Module API | Axum handler; opsional modul lain |

### 17.4 Komunikasi Antar Modul (Provider → Receiver)

```text
Modul 4 (Provider)
      │
      │ pub fn compare_species(...)
      ▼
Axum handler /api/v1/compare  (Receiver / Aplikasi)
      │  hasil: ComparisonResult
      ▼
Django API → Frontend
```

Hubungan opsional (interface publik, bukan dependency crate):

```text
Modul 1 ──species_ids──▶ Modul 4 (RECEIVER dari hasil pencarian, opsional)
Modul 2 ──species_ids──▶ Modul 4 (penerusan ID untuk perbandingan, opsional)
Modul 3 ──DiversityStats─▶ Modul 4 (konteks keanekaragaman, opsional)
```

| Provider | `pub fn` | Receiver | Purpose | Data yang Dikirim | Priority |
| --- | --- | --- | --- | --- | --- |
| Modul 4 | `compare_species` | Axum handler | Respons API perbandingan | `ComparisonQuery` | **Required** (API) |
| Modul 1 | `search` | Modul 4 | Kandidat species_ids untuk perbandingan | `Vec<(&Species, f64)>` → species_ids | Optional |
| Modul 2 | `explore_relationships` | Modul 4 | Penerusan ID spesies untuk perbandingan | species_ids | Optional |
| Modul 3 | `calculate_diversity` | Modul 4 | Statistik keanekaragaman sebagai konteks | `DiversityStats` | Optional |

> **Selama pengembangan paralel:** Modul 4 menerima input `species_ids`; asal ID dapat berupa input manual pengguna, hasil Modul 1, atau hasil Modul 2. Selama Modul 1/2 belum siap, modul memakai **mock** atau input langsung. Modul 4 tidak bergantung pada implementasi internal modul lain.

### 17.5 Rustdoc — Modul 4

Rustdoc diwajibkan untuk **seluruh `pub fn`** dan **seluruh tipe publik** modul ini:

* `compare_species`, `calculate_pair_similarity`.
* Tipe publik yang muncul di signature `pub fn` (mis. `ComparisonQuery`, `ComparisonResult`, `SimilarityScore`, `AttributeRow`, `ComparisonError`).

Command verifikasi:

```bash
cargo doc --workspace --no-deps --open
cargo check
cargo test
```

Status saat ini di dokumen: **Rustdoc planned** (belum diklaim verified).

---

## 18. Rubrik — Evidence Modul 4

| Rubrik | Evidence di Planning Modul 4 | Bukti Implementasi yang Masih Diperlukan |
| --- | --- | --- |
| Repositori Github (15%) | Lokasi `crates/species-comparison` dan command build/test/rustdoc | Repositori dibuat & diakses dosen |
| Prioritas Modul (25%) | Prioritas fitur: validasi → matriks → shared/unique → similarity → similar-species | Implementasi fitur prioritas |
| Rustdoc (20%) | Rustdoc diwajibkan untuk semua `pub fn` (Bagian 17.5) | Generate & aksesibel |
| Komunikasi Antar Module (40%) | Batas `mod`/`pub fn` (17.1–17.2), visibilitas (17.3), matriks komunikasi (17.4) termasuk relasi M1/M2/M3 dari Bagian 13 | Interface diimplementasikan & diuji |
