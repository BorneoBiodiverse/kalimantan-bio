# Planning Function - Modul 5: Biodiversity Knowledge & Citation Explorer

**Mata Kuliah:** Pemrograman Fungsional  
**Bahasa:** Rust  
**Cakupan:** Eksplorasi publikasi ilmiah, topik riset, lokasi penelitian, timeline riset, coverage analysis, dan rekomendasi sitasi biodiversitas Kalimantan.

**Repository:** Part of KalimantanBio monorepo workspace (`crates/knowledge-citations`)  
**API Integration:** Production KalimantanBio API (PostgreSQL via shared library)

> **PENTING**: Modul ini menggunakan **Shared Library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum. Lihat [MASTERPLAN.md](../MASTERPLAN.md) untuk arsitektur lengkap.

> Catatan integrasi: Production KalimantanBio database menyediakan data spesies, taksonomi, observasi, lokasi, dan institusi. Data publikasi ilmiah pada tahap awal dapat dibaca dari fixture JSON/CSV atau adapter terpisah. Integrasi endpoint publikasi dapat ditambahkan tanpa mengubah fungsi analitik murni.

---

## 1. Tujuan Modul

Modul ini membangun sistem eksplorasi pengetahuan ilmiah yang menghubungkan spesies biodiversitas Kalimantan dengan publikasi, topik riset, lokasi penelitian, peneliti, dan referensi sitasi.

Modul ini bertanggung jawab untuk:

* Menemukan publikasi yang berhubungan dengan spesies atau kelompok taksonomi.
* Mengeksplorasi publikasi berdasarkan topik biodiversitas dan lokasi penelitian.
* Menganalisis perkembangan penelitian dari waktu ke waktu.
* Mengukur coverage penelitian dan menemukan spesies atau takson yang understudied.
* Memberikan rekomendasi referensi serta mengekspor sitasi ke format akademik umum.

### Batasan Modul

Modul ini **tidak bertanggung jawab** terhadap:

* Menyimpan atau mengubah master data spesies dan taksonomi.
* Menentukan kebenaran ilmiah isi publikasi secara otomatis.
* Mengunduh teks penuh yang dibatasi hak aksesnya.
* Membuat visualisasi UI secara langsung; modul hanya menghasilkan data graph dan report.
* Menggantikan proses peer review atau validasi bibliografis oleh ahli.

---

## 2. Shared Library Integration

Modul ini menggunakan **shared library** (`kalimantanbio-shared`) untuk tipe data dan fungsi umum yang digunakan bersama dengan modul lain.

### 2.1 Shared Types Used

```rust
use kalimantanbio_shared::core::Species;
```

- `Species` - Struktur data lengkap spesies dari production database (untuk mencocokkan publikasi dengan spesies)

### 2.2 Shared Functions Used

#### From `text` module:
```rust
use kalimantanbio_shared::text::normalize_text;
```

- `normalize_text(input: &str) -> String` - Menormalkan teks (trim, huruf kecil, rapikan spasi) untuk pencocokan judul/topik/lokasi

**Digunakan di**: Tahap 1 (Normalization), Tahap 2 (Query Explorer)

#### From `stats` module:
```rust
use kalimantanbio_shared::stats::{build_timeline, count_by_key, calculate_coverage_percentage};
```

- `build_timeline<T>(items: &[T], year_fn: impl Fn(&T) -> u16) -> BTreeMap<u16, usize>` - Menghitung jumlah publikasi per tahun
- `count_by_key<T, K>(items: &[T], key_fn: impl Fn(&T) -> K) -> HashMap<K, usize>` - Menghitung distribusi berdasarkan topik/lokasi
- `calculate_coverage_percentage(available: usize, total: usize) -> f64` - Menghitung skor coverage (0.0–1.0)

**Digunakan di**: Tahap 3 (Timeline & Coverage Analysis)

#### From `scoring` module:
```rust
use kalimantanbio_shared::scoring::{rank_by_score, filter_by_threshold, top_n};
```

- `rank_by_score<T>(items: Vec<(T, f64)>) -> Vec<(T, f64)>` - Mengurutkan publikasi berdasarkan skor relevansi
- `filter_by_threshold<T>(items: Vec<(T, f64)>, threshold: f64) -> Vec<(T, f64)>` - Menyaring di bawah threshold
- `top_n<T>(items: Vec<(T, f64)>, n: usize) -> Vec<(T, f64)>` - Mengambil N referensi teratas

**Digunakan di**: Tahap 4 (Citation Ranking & Recommendation)

### 2.3 Module-Specific Types

Modul ini mendefinisikan tipe tambahan yang spesifik untuk publikasi dan sitasi:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
struct Publication {
    id: String,
    title: String,
    abstract_text: Option<String>,
    publication_year: u16,
    authors: Vec<Author>,
    venue: Option<String>,
    doi: Option<String>,
    url: Option<String>,
    source_type: SourceType,
    species_ids: Vec<u64>,
    taxon_names: Vec<String>,
    topics: Vec<ResearchTopic>,
    locations: Vec<ResearchLocation>,
    citation_count: Option<u32>,
}

#[derive(Debug, Clone, Default)]
struct PublicationQuery {
    text: Option<String>,
    species_id: Option<u64>,
    taxon_name: Option<String>,
    topic: Option<ResearchTopic>,
    location: Option<String>,
    year_from: Option<u16>,
    year_to: Option<u16>,
    source_type: Option<SourceType>,
}

#[derive(Debug, Clone, Default)]
struct CoverageRecord {
    entity_name: String,
    publication_count: usize,
    species_count: usize,
    first_year: Option<u16>,
    latest_year: Option<u16>,
}
```

(Struktur `Author`, `ProfessionalLocation`, `ResearchTopic`, `SourceType`, dan `LocationType` tetap didefinisikan di modul ini — lihat Bagian 3.)

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
    text::normalize_text,
    stats::{build_timeline, count_by_key, calculate_coverage_percentage},
    scoring::{rank_by_score, top_n},
};

pub fn analyze_timeline(items: &[Publication]) -> BTreeMap<u16, usize> {
    // Gunakan shared library untuk timeline
    build_timeline(items, |p| p.publication_year)
}

pub fn analyze_topic_distribution(items: &[Publication]) -> HashMap<ResearchTopic, usize> {
    // Gunakan shared library untuk counting
    count_by_key(items, |p| p.topics.first().cloned().unwrap_or_default())
}

pub fn rank_citations<'a>(
    items: &[&'a Publication],
    query: &PublicationQuery,
    limit: usize,
) -> Vec<&'a Publication> {
    let scored: Vec<(&Publication, f64)> = items
        .iter()
        .map(|p| (**p, score_publication_relevance(p, query)))
        .collect();

    // Gunakan shared library untuk ranking dan limit
    let ranked = rank_by_score(scored.into_iter().map(|(p, s)| (p, s)).collect());
    top_n(ranked, limit).into_iter().map(|(p, _)| p).collect()
}
```

---

## 3. Struktur Data (Domain Model)

Struktur data tambahan yang spesifik untuk Modul 5 (selain tipe di Bagian 2.3):

```rust
#[derive(Debug, Clone, serde::Deserialize)]
struct Author {
    id: Option<String>,
    name: String,
    affiliation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SourceType {
    JournalArticle,
    ConferencePaper,
    Thesis,
    Report,
    BookChapter,
    Dataset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ResearchTopic {
    Taxonomy,
    Ecology,
    Conservation,
    Ethnobotany,
    Habitat,
    SpeciesIdentification,
    Other(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ResearchLocation {
    name: String,
    province: Option<String>,
    regency: Option<String>,
    location_type: LocationType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LocationType {
    Province,
    Regency,
    Forest,
    ConservationArea,
    Other(String),
}
```

> **Catatan:** Tipe `Species` sudah distandardisasi di shared library. Tipe `Publication`, `Author`, `ResearchTopic`, `ResearchLocation`, dan enum terkait bersifat spesifik untuk Modul 5.

### Data yang Digunakan

| Data | Tipe | Deskripsi |
| --- | --- | --- |
| `Publication` | `struct` | Metadata publikasi dan hubungan ilmiahnya. |
| `species_ids` | `Vec<u64>` | ID spesies yang dibahas dalam publikasi. |
| `taxon_names` | `Vec<String>` | Genus, famili, atau takson lain yang berkaitan. |
| `topics` | `Vec<ResearchTopic>` | Kategori topik biodiversitas penelitian. |
| `locations` | `Vec<ResearchLocation>` | Lokasi studi di Kalimantan. |
| `publication_year` | `u16` | Tahun terbit untuk timeline dan analisis tren. |
| `citation_count` | `Option<u32>` | Jumlah sitasi jika tersedia dari sumber data. |

---

## 4. Tahap 1 - Data Ingestion, Normalization & Validation

Membaca data publikasi dari JSON/CSV/API, menormalisasi metadata, dan menghapus atau menandai record yang tidak valid.

> **Catatan**: Fungsi `normalize_text` menggunakan **shared library**.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `parse_publications` | `fn parse_publications(raw: &str) -> Result<Vec<Publication>, ParseError>` | **MODULE-SPECIFIC**: Mendeserialisasi input publikasi terstruktur. |
| `normalize_publication` | `fn normalize_publication(publication: Publication) -> Publication` | **Uses SHARED `normalize_text`**: Menormalkan judul, DOI, nama penulis, topik, dan lokasi. |
| `validate_publication` | `fn validate_publication(publication: &Publication) -> Result<(), ValidationError>` | **MODULE-SPECIFIC**: Memeriksa ID, judul, tahun, dan metadata minimum. |
| `deduplicate_publications` | `fn deduplicate_publications(items: &[Publication]) -> Vec<Publication>` | **MODULE-SPECIFIC**: Menghapus duplikasi berdasarkan ID, DOI, atau fingerprint judul. |
| `build_species_index` | `fn build_species_index(items: &[Publication]) -> HashMap<u64, Vec<String>>` | **MODULE-SPECIFIC**: Membuat indeks spesies ke ID publikasi. |

**Person in Charge:** **[Nama anggota]**

---

## 5. Tahap 2 - Species, Topic & Location Explorer

Menyediakan query murni untuk menemukan publikasi berdasarkan spesies, topik, lokasi, dan kombinasi filter.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `publications_for_species` | `fn publications_for_species<'a>(items: &'a [Publication], species_id: u64) -> Vec<&'a Publication>` | **MODULE-SPECIFIC**: Mengambil publikasi yang terkait dengan spesies. |
| `publications_for_taxon` | `fn publications_for_taxon<'a>(items: &'a [Publication], taxon: &str) -> Vec<&'a Publication>` | **MODULE-SPECIFIC**: Mencari publikasi berdasarkan nama genus/famili/takson. |
| `publications_for_topic` | `fn publications_for_topic<'a>(items: &'a [Publication], topic: &ResearchTopic) -> Vec<&'a Publication>` | **MODULE-SPECIFIC**: Memfilter publikasi berdasarkan topik riset. |
| `publications_for_location` | `fn publications_for_location<'a>(items: &'a [Publication], location: &str) -> Vec<&'a Publication>` | **MODULE-SPECIFIC**: Memfilter publikasi berdasarkan lokasi studi. |
| `query_publications` | `fn query_publications<'a>(items: &'a [Publication], query: &PublicationQuery) -> Vec<&'a Publication>` | **MODULE-SPECIFIC**: Menggabungkan seluruh filter query menggunakan iterator predicates. |

**Person in Charge:** **[Nama anggota]**

---

## 6. Tahap 3 - Timeline & Research Coverage Analysis

Mengubah kumpulan publikasi menjadi statistik perkembangan dan cakupan penelitian.

> **Catatan**: Fungsi `build_timeline`, `count_by_key`, dan `calculate_coverage_percentage` menggunakan **shared library**.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `build_research_timeline` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::stats::build_timeline` | Menghitung jumlah publikasi per tahun. |
| `calculate_topic_distribution` | **Uses SHARED `count_by_key`**: `fn calculate_topic_distribution(items: &[Publication]) -> HashMap<ResearchTopic, usize>` | Menghitung distribusi publikasi berdasarkan topik. |
| `calculate_location_distribution` | **Uses SHARED `count_by_key`**: `fn calculate_location_distribution(items: &[Publication]) -> HashMap<String, usize>` | Menghitung jumlah publikasi per lokasi. |
| `calculate_entity_coverage` | `fn calculate_entity_coverage(items: &[Publication], species_ids: &[u64]) -> Vec<CoverageRecord>` | **MODULE-SPECIFIC**: Membandingkan jumlah publikasi antarspesies. |
| `find_understudied_species` | **Uses SHARED `filter_by_threshold`**: `fn find_understudied_species(coverage: &[CoverageRecord], threshold: usize) -> Vec<&CoverageRecord>` | Menemukan spesies dengan jumlah publikasi di bawah threshold. |
| `calculate_coverage_score` | **FROM SHARED LIBRARY**: `kalimantanbio_shared::stats::calculate_coverage_percentage` | Menghasilkan skor coverage dalam rentang 0.0 hingga 1.0. |

**Person in Charge:** **[Nama anggota]**

> Aturan threshold dan definisi `expected` harus disepakati tim. Contoh awal: spesies dengan 0-1 publikasi masuk kategori understudied.

---

## 7. Tahap 4 - Citation Ranking, Recommendation & Export

Mengurutkan referensi yang paling relevan dan menghasilkan format sitasi yang dapat digunakan dalam tulisan akademik.

> **Catatan**: Fungsi `rank_by_score`, `filter_by_threshold`, dan `top_n` menggunakan **shared library**.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `score_publication_relevance` | `fn score_publication_relevance(publication: &Publication, query: &PublicationQuery) -> f64` | **MODULE-SPECIFIC**: Menghitung skor berdasarkan kecocokan spesies, topik, lokasi, tahun, dan teks. |
| `rank_publications` | **Uses SHARED `rank_by_score`**: `fn rank_publications<'a>(items: &[&'a Publication], query: &PublicationQuery) -> Vec<(&'a Publication, f64)>` | Mengurutkan publikasi berdasarkan skor relevansi. |
| `recommend_citations` | **Uses SHARED `top_n`**: `fn recommend_citations<'a>(items: &'a [Publication], query: &PublicationQuery, limit: usize) -> Vec<&'a Publication>` | Menghasilkan rekomendasi referensi teratas. |
| `format_apa` | `fn format_apa(publication: &Publication) -> String` | **MODULE-SPECIFIC**: Mengekspor satu publikasi ke format APA sederhana. |
| `format_bibtex` | `fn format_bibtex(publication: &Publication) -> String` | **MODULE-SPECIFIC**: Mengekspor satu publikasi ke format BibTeX. |
| `export_citations` | `fn export_citations(items: &[Publication], format: CitationFormat) -> String` | **MODULE-SPECIFIC**: Menghasilkan kumpulan sitasi dalam format yang dipilih. |

**Person in Charge:** **[Nama anggota]**

> Citation ranking hanya merekomendasikan referensi berdasarkan metadata. Hasilnya tetap perlu diverifikasi pengguna sebelum digunakan sebagai sitasi akademik.

---

## 8. Tahap 5 - Knowledge Network & Integrasi

Membangun representasi relasi antarspesies, publikasi, topik, lokasi, dan peneliti, lalu menguji seluruh pipeline.

| Fungsi / Komponen | Signature / Bentuk | Deskripsi |
| --- | --- | --- |
| `build_knowledge_graph` | `fn build_knowledge_graph(items: &[Publication]) -> KnowledgeGraph` | Membuat node dan edge dari relasi publikasi. |
| `related_publications` | `fn related_publications<'a>(graph: &KnowledgeGraph, publication_id: &str, items: &'a [Publication]) -> Vec<&'a Publication>` | Mencari publikasi yang berbagi spesies, topik, lokasi, atau penulis. |
| `fetch_publications` | `async fn fetch_publications(base_url: &str) -> Result<Vec<Publication>, SourceError>` | Adapter I/O untuk sumber publikasi eksternal. |
| `explore_knowledge` | `fn explore_knowledge(input: &KnowledgeInput) -> Result<KnowledgeReport, PipelineError>` | Entry point integrasi seluruh tahap. |
| Unit tests | `mod tests { ... }` | Menguji parsing, filtering, timeline, coverage, ranking, dan export. |

**Person in Charge:** **[Nama anggota]**

Knowledge graph minimal:

```text
Species ---- discussed_in ---- Publication ---- written_by ---- Researcher
   |                              |
   |                              +---- about ---- Topic
   |
   +----------------------------- studied_at ---- Location
```

---

## 9. Komposisi Pipeline Utama

```rust
use kalimantanbio_shared::{
    text::normalize_text,
    stats::{build_timeline, count_by_key, calculate_coverage_percentage},
    scoring::{rank_by_score, top_n},
};

fn explore_knowledge(input: &KnowledgeInput) -> Result<KnowledgeReport, PipelineError> {
    let parsed = parse_publications(&input.raw_publications)?;
    let normalized = parsed
        .into_iter()
        .map(normalize_publication)
        .collect::<Vec<_>>();
    let publications = deduplicate_publications(&normalized);

    let matching = query_publications(&publications, &input.query);
    let ranked = rank_publications(&matching, &input.query);
    let timeline = build_research_timeline(&publications);
    let coverage = calculate_entity_coverage(&publications, &input.species_ids);
    let understudied = find_understudied_species(&coverage, input.understudied_threshold);
    let graph = build_knowledge_graph(&publications);

    Ok(KnowledgeReport {
        ranked_publications: ranked,
        timeline,
        coverage,
        understudied,
        graph,
    })
}
```

Pipeline konseptual:

```text
Raw Publications
  ↓
Parsing, Normalization & Validation
  ↓
Deduplication & Indexing
  ↓
Species / Topic / Location Query
  ↓
Timeline & Coverage Analysis
  ↓
Ranking & Citation Recommendation
  ↓
Knowledge Graph + Citation Export
```

Fungsi `explore_knowledge` menjadi entry point modul dan menggabungkan fungsi-fungsi murni dengan adapter I/O yang dipisahkan pada batas integrasi.

---

## 10. Prinsip Functional Programming yang Perlu Dipegang Tim

### Pure Functions

Parsing, filtering, scoring, agregasi, graph construction, dan formatting harus menghasilkan output hanya berdasarkan inputnya. Network request pada `fetch_publications` dipisahkan sebagai fungsi I/O.

### Immutability

Jangan memodifikasi koleksi publikasi sumber. Gunakan reference untuk membaca data dan kembalikan koleksi baru untuk hasil filter, ranking, atau agregasi.

### Higher-Order Functions

Gunakan `.map()`, `.filter()`, `.fold()`, `.flat_map()`, `.group_by()` atau pola iterator Rust yang sesuai untuk transformasi dan agregasi.

### Function Composition

Pertahankan pipeline yang jelas:

```text
parse → normalize → deduplicate → query → analyze → rank → export
```

Setiap tahap harus dapat diuji tanpa harus menjalankan database atau service eksternal.

### Shared Functions adalah Pure Functions

Fungsi dari shared library (`normalize_text`, `build_timeline`, `count_by_key`, `calculate_coverage_percentage`, `rank_by_score`, `filter_by_threshold`, `top_n`) adalah **pure functions** yang diuji satu kali di library. Modul ini cukup memanggil dan memesan komposisinya, tanpa mengubah implementasinya.

---

## 11. Pembagian Kerja - 5 Anggota

| # | Tahap | Fungsi / Tanggung Jawab Utama | PIC | Status |
| --- | --- | --- | --- | --- |
| 1 | Ingestion & Validation | `parse_publications`, `normalize_publication`, `deduplicate_publications` (via shared `normalize_text`) | | Belum dimulai |
| 2 | Explorers | Query spesies, takson, topik, dan lokasi | | Belum dimulai |
| 3 | Timeline & Coverage | Timeline, distribusi, coverage, understudied explorer (via shared `build_timeline`, `count_by_key`, `calculate_coverage_percentage`) | | Belum dimulai |
| 4 | Citation | Ranking, recommendation, APA, BibTeX export (via shared `rank_by_score`, `top_n`) | | Belum dimulai |
| 5 | Integration | Knowledge graph, adapter API, pipeline, integration tests | | Belum dimulai |

> **Catatan:** Fungsi `normalize_text`, `build_timeline`, `count_by_key`, `calculate_coverage_percentage`, `rank_by_score`, `filter_by_threshold`, dan `top_n` berasal dari **shared library** (`kalimantanbio-shared`) dan **tidak perlu diimplementasikan** di modul ini.

Setiap PIC bertanggung jawab terhadap implementasi, unit test, dokumentasi, dan menjaga signature yang telah disepakati.

---

## 12. Kesepakatan Antaranggota

Sebelum implementasi dimulai, seluruh anggota perlu menyepakati:

* Format sumber data publikasi: JSON, CSV, API, atau gabungan.
* Field minimum yang wajib dimiliki setiap publikasi.
* Cara mencocokkan publikasi dengan species ID dan nama takson.
* Kosakata resmi topik penelitian.
* Standar penamaan dan hierarki lokasi.
* Definisi coverage dan threshold understudied.
* Formula ranking dan bobot setiap sinyal relevansi.
* Penanganan publikasi duplikat tanpa DOI.
* Format citation export yang wajib didukung.
* Batas antara data mentah, fungsi analitik, dan adapter I/O.

---

## 13. Independensi Modul

Modul ini dikembangkan sebagai komponen independen dalam KalimantanBio.

Prinsip yang digunakan:

* Modul dapat dikembangkan dan diuji secara mandiri menggunakan fixture publikasi.
* Modul hanya bergantung pada **shared library** (`kalimantanbio-shared`) untuk tipe data (`Species`) dan fungsi umum (`normalize_text`, `build_timeline`, `count_by_key`, `calculate_coverage_percentage`, `rank_by_score`).
* Publikasi ilmiah dimodelkan sebagai data milik modul ini; `Species` dari shared library digunakan untuk mencocokkan publikasi dengan spesies.
* Tidak ada ketergantungan pada implementasi internal Modul 1, 2, 3, atau 4.
* Modul hanya mengekspos **interface publik (`pub fn`)** yang dibutuhkan — lihat Bagian 17.
* Integrasi dengan modul lain bersifat opsional dan dilakukan melalui interface yang telah disepakati, dikomposisikan di lapisan aplikasi (Axum api-server) bukan sebagai dependency crate.

## 14. Kriteria Selesai Modul

* [ ] Data publikasi dapat diparse dari fixture lokal.
* [ ] Data publikasi tervalidasi, dinormalisasi, dan dideduplikasi.
* [ ] Species-to-publication explorer berjalan.
* [ ] Research topic explorer berjalan.
* [ ] Research location explorer berjalan.
* [ ] Species research timeline dapat dihasilkan.
* [ ] Research coverage analysis dapat membandingkan entitas.
* [ ] Understudied species explorer menghasilkan hasil berdasarkan threshold yang terdokumentasi.
* [ ] Citation recommendation menghasilkan ranking deterministik.
* [ ] Citation export mendukung minimal APA dan BibTeX.
* [ ] Knowledge network menghasilkan node dan edge yang valid.
* [ ] Setiap fungsi inti memiliki unit test.
* [ ] Interface publik (`pub fn explore_knowledge`) didefinisikan jelas dan teruji — lihat Bagian 17.
* [ ] Seluruh `pub fn` memiliki dokumentasi rustdoc lengkap.
* [ ] Pipeline dapat berjalan tanpa service eksternal menggunakan dummy fixture.
* [ ] Adapter sumber publikasi dapat diuji secara terpisah dari fungsi murni.

---

## 15. Contoh Skenario Pengujian

### Skenario 1 - Species-to-Publication Explorer

**Input:** species ID `4`.

**Expected:** hanya publikasi yang memuat species ID `4`, terurut berdasarkan skor relevansi atau tahun sesuai query.

### Skenario 2 - Research Topic Explorer

**Input:** topik `Conservation`.

**Expected:** semua publikasi bertopik konservasi, tanpa mengubah data sumber.

### Skenario 3 - Research Location Explorer

**Input:** lokasi `Kalimantan Barat`.

**Expected:** publikasi yang memiliki lokasi studi di provinsi tersebut atau lokasi turunannya.

### Skenario 4 - Coverage dan Understudied Species

**Input:** daftar species ID dan threshold `1`.

**Expected:** spesies dengan nol atau satu publikasi masuk daftar understudied secara deterministik.

### Skenario 5 - Citation Export

**Input:** satu publikasi dengan penulis, tahun, judul, venue, dan DOI.

**Expected:** output APA dan BibTeX berisi metadata yang sama serta dapat diproses sebagai teks.

### Skenario 6 - Knowledge Network

**Input:** dua publikasi yang memiliki spesies atau topik yang sama.

**Expected:** graph memiliki node publikasi dan edge relasi yang sesuai, tanpa edge duplikat.

---

## 16. Langkah Selanjutnya

1. Verifikasi bahwa **shared library** (`kalimantanbio-shared`) sudah tersedia; jika belum, blokir pengerjaan sampai siap.
2. Finalisasi domain model bersama project lead (tipe `Species` sudah standar; tipe `Publication`, `ResearchTopic`, `ResearchLocation` spesifik modul).
3. Sepakati format sumber data publikasi (JSON/CSV/fixture) dan field minimum wajib.
4. Sepakati kosakata resmi topik penelitian dan hierarki lokasi studi.
5. Sepakati formula ranking relevansi dan bobot setiap sinyal (spesies, topik, lokasi, tahun, teks).
6. Sepakati definisi coverage dan threshold understudied species.
7. Tentukan pembagian fungsi berdasarkan PIC (isi tabel Bagian 11).
8. Setiap PIC membuat signature dan dokumentasi singkat fungsi masing-masing.
9. Review interface bersama sebelum implementasi logic dimulai.
10. Siapkan data dummy `Vec<Publication>` (independen dari DB) untuk unit test.
11. Implementasikan fungsi secara paralel sesuai pembagian kerja; gunakan fungsi shared library.
12. Setiap PIC membuat unit test untuk fungsi masing-masing.
13. Gabungkan seluruh tahap ke dalam pipeline `explore_knowledge()`.
14. Lakukan pengujian end-to-end menggunakan data dummy dan kemudian data production (via shared library).
15. Dokumentasikan hasil dan contoh penggunaan modul.
16. Review akhir sebelum modul dianggap selesai, termasuk kelengkapan rustdoc pada seluruh `pub fn`.

---

## 17. Interface Publik & Komunikasi Antar Modul

Bagian ini menjelaskan batas `mod` dan `pub fn` Modul 5 sebagai bukti pemenuhan aspek **Komunikasi Antar Module** pada rubrik penilaian (40%).

### 17.1 Batas Modul (Owns / Does Not Own / Internal / Public)

| Aspek | Isi |
| --- | --- |
| **Owns** | Data `Publication`, parsing/normalisasi/deduplikasi, eksplorasi spesies→publikasi, eksplorasi topik/lokasi, timeline, coverage, understudied species, citation recommendation, citation export, dan knowledge network. |
| **Does Not Own** | Data `Species` (shared library), pencarian spesies (M1), relasi antarspesies (M2), taksonomi (M3), perbandingan (M4). |
| **Internal** | `parse_publications`, `normalize_publication`, `validate_publication`, `deduplicate_publications`, `build_species_index`, `publications_for_species/topic/taxon/location`, `query_publications`, `build_research_timeline`, `calculate_topic_distribution`, `calculate_location_distribution`, `calculate_entity_coverage`, `find_understudied_species`, `calculate_coverage_score`, `score_publication_relevance`, `rank_publications`, `format_apa`, `format_bibtex`, `build_knowledge_graph`, `related_publications`. |
| **Publicly Exposes** | `explore_knowledge` (main), `recommend_citations` (opsional), dan `export_citations` (opsional) — kapabilitas yang dapat digunakan Axum handler maupun modul lain. |

### 17.2 Fungsi Publik (`pub fn`) — Modul 5

| `pub fn` | Provider | Consumer Potensial | Purpose | Input | Output | Why Needed |
| --- | --- | --- | --- | --- | --- | --- |
| `explore_knowledge` | Modul 5 | Axum handler `/api/v1/knowledge/*` | Entry point pipeline Tahap 1–5 | query (spesies/topik/lokasi), dataset publikasi | `Result<KnowledgeReport, ModuleError>` | Kapabilitas utama modul untuk aplikasi dan modul lain. |
| `recommend_citations` *(opsional)* | Modul 5 | Axum handler `/api/v1/knowledge/citations` | Merekomendasikan publikasi relevan | query, publikasi | `Vec<Publication>` terurut | Rekomendasi sitasi untuk pengguna/modul lain. |
| `export_citations` *(opsional)* | Modul 5 | Axum handler `/api/v1/knowledge/export` | Menyusun string APA dan BibTeX | daftar publikasi | `CitationExport` | Membuka konsumsi sitasi ke luar sistem. |

```rust
/// Menjelajahi pengetahuan dan sitasi terkait spesies/topik/lokasi.
///
/// # Arguments
///
/// * `query` - Kriteria eksplorasi (spesies, topik, lokasi, tahun).
/// * `publications` - Dataset publikasi dari adapter/fixture.
///
/// # Returns
///
/// Publikasi, timeline, distribusi, coverage, dan understudied species.
pub fn explore_knowledge(
    query: &KnowledgeQuery,
    publications: &[Publication],
) -> Result<KnowledgeReport, ModuleError>
```

### 17.3 Visibilitas Fungsi

| Fungsi | Visibilitas | Lapisan | Konsumen |
| --- | --- | --- | --- |
| `parse_publications`, `normalize_publication`, `validate_publication`, `deduplicate_publications` | private | Internal helper (module-specific) | Pipeline |
| `build_species_index`, `publications_for_species/topic/taxon/location`, `query_publications` | private | Internal helper (module-specific) | Pipeline |
| `build_research_timeline`, `calculate_topic_distribution`, `calculate_location_distribution`, `calculate_entity_coverage`, `find_understudied_species`, `calculate_coverage_score` | private | Internal helper (module-specific) | Pipeline |
| `score_publication_relevance`, `rank_publications` | private | Internal helper (module-specific) | `recommend_citations` |
| `format_apa`, `format_bibtex` | private | Internal helper (module-specific) | `export_citations` |
| `build_knowledge_graph`, `related_publications` | private | Internal helper (module-specific) | Pipeline |
| `recommend_citations` | `pub fn` | Module API (opsional) | Axum handler; opsional modul lain |
| `export_citations` | `pub fn` | Module API (opsional) | Axum handler; opsional modul lain |
| `explore_knowledge` | `pub fn` | Module API | Axum handler; opsional modul lain |

### 17.4 Komunikasi Antar Modul (Provider → Receiver)

```text
Modul 5 (Provider)
      │
      │ pub fn explore_knowledge(...)
      ▼
Axum handler /api/v1/knowledge/*  (Receiver / Aplikasi)
      │  hasil: KnowledgeReport
      ▼
Django API → Frontend
```

Hubungan opsional (interface publik, bukan dependency crate):

```text
Modul 1 ──species scope──▶ Modul 5 (RECEIVER dari hasil pencarian, opsional)
Modul 2 ──species_ids────▶ Modul 5 (membuka referensi spesies, opsional)
Modul 3 ──GapReport──────▶ Modul 5 (spesies understudied, opsional)
```

| Provider | `pub fn` | Receiver | Purpose | Data yang Dikirim | Priority |
| --- | --- | --- | --- | --- | --- |
| Modul 5 | `explore_knowledge` | Axum handler | Respons API eksplorasi pengetahuan | `KnowledgeQuery` | **Required** (API) |
| Modul 1 | `search` | Modul 5 | Cakupan spesies untuk eksplorasi publikasi | species_ids | Optional |
| Modul 2 | `explore_relationships` | Modul 5 | Membuka referensi spesies pada relasi | species_ids | Optional |
| Modul 3 | `analyze_taxonomic_gap` | Modul 5 | Spesies understudied untuk prioritas riset | `GapReport` | Optional |

> **Selama pengembangan paralel:** Modul 5 dapat berjalan berdasarkan `KnowledgeQuery` mandiri (spesies/topik/lokasi) dan memakai **mock** hasil Modul 1/2/3 bila dihubungkan. Modul 5 tidak bergantung pada implementasi internal modul lain.

### 17.5 Rustdoc — Modul 5

Rustdoc diwajibkan untuk **seluruh `pub fn`** dan **seluruh tipe publik** modul ini:

* `explore_knowledge`, `recommend_citations`, `export_citations`.
* Tipe publik yang muncul di signature `pub fn` (mis. `KnowledgeQuery`, `KnowledgeReport`, `Publication`, `ResearchTopic`, `ResearchLocation`, `KnowledgeNetwork`, `CitationExport`).

Command verifikasi:

```bash
cargo doc --workspace --no-deps --open
cargo check
cargo test
```

Status saat ini di dokumen: **Rustdoc planned** (belum diklaim verified).

---

## 18. Rubrik — Evidence Modul 5

| Rubrik | Evidence di Planning Modul 5 | Bukti Implementasi yang Masih Diperlukan |
| --- | --- | --- |
| Repositori Github (15%) | Lokasi `crates/knowledge-citations` dan command build/test/rustdoc | Repositori dibuat & diakses dosen |
| Prioritas Modul (25%) | Prioritas fitur: species↔publication → topic → location → coverage → understudied | Implementasi fitur prioritas |
| Rustdoc (20%) | Rustdoc diwajibkan untuk semua `pub fn` (Bagian 17.5) | Generate & aksesibel |
| Komunikasi Antar Module (40%) | Batas `mod`/`pub fn` (17.1–17.2), visibilitas (17.3), matriks komunikasi (17.4) termasuk relasi M1/M2/M3 dari Bagian 13 | Interface diimplementasikan & diuji |
