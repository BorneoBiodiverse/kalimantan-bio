# AI Context Handover: Intelligent Multilingual Species Search & Query Parser

> **Target Audience:** AI Coding Assistants / Collaborator Developers  
> **Topic:** Fitur Intelligent Multilingual Species Search & Taxonomy/Trait Query Parser  
> **Repositories Involved:**  
> 1. `kalimantan-bio` (Rust backend core & query parser engine) - Branch: `module1/feat/intelligent-species-search` (PR #29)  
> 2. `biodiversitas_borneo_production` (Django 5 web portal & HTMX consumer) - Branch: `production`  
> **Tanggal Update:** 1 November 2026

---

## 1. Executive Summary & Architecture

Fitur ini mengintegrasikan **Query Parser cerdas berbasis Rust** dari proyek `kalimantan-bio` ke dalam portal web Django `biodiversitas_borneo_production`. 

Sistem dapat menerima pencarian teks bebas (*natural language*), mengekstrak atribut taksonomi, status konservasi, regulasi perlindungan hukum, dan sifat biologis secara multi-bahasa (Indonesia & Inggris), lalu memetakan kriteria tersebut ke query database Django ORM secara *sub-millisecond*.

```
+-----------------------------------------------------------------------------------+
| USER QUERY (e.g. "mamalia kritis dilindungi", "karnivora", "bunga", "p106:bebas") |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| [DJANGO LAYER] repository/search_parser.py                                        |
| 1. Mencari Rust binary `species_search_parser` di PATH atau bin/                  |
| 2. Menjalankan subproses (atau otomatis beralih ke `_python_fallback_parser`)    |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| [RUST ENGINE] kalimantan-bio: crates/species-search/src/parser.rs                 |
| 1. Tokenisasi & Normalisasi teks (`crates/shared/src/text.rs`)                    |
| 2. Ekstraksi Multibahasa & Prefix Syntax (`TokenKind`)                            |
| 3. Output JSON `QueryFilters`                                                     |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| [DJANGO ORM & HTMX] apply_species_search()                                        |
| 1. Memfilter QuerySet: Kingdom, Class, Order, IUCN, P.106, Origin Type            |
| 2. Semantic Biological Expansion (Diet, Reproduksi, Habitat, Organ Tumbuhan)     |
| 3. Relevance Scoring Annotation & Order By                                        |
| 4. Rendering partial HTML + Badges aktif (`⚡ rust_binary` / `🐍 python_fallback`) |
+-----------------------------------------------------------------------------------+
```

---

## 2. Rincian Perubahan di Repo Rust (`kalimantan-bio`)

### A. File yang Ditambahkan (Added)
1. **`crates/species-search/src/bin/species_search_parser.rs`**
   - Binary CLI mandiri agar proses eksternal (seperti Django/Python) dapat memanggil parser Rust secara langsung tanpa harus menjalankan server HTTP Axum.
   - Mengambil argumen `raw_query`, menjalankan `species_search::parse_query(&raw_query)`, dan mencetak output JSON ke `stdout`.

### B. File yang Diubah (Modified)
1. **`crates/species-search/src/types.rs`**
   - Menambahkan derive `Serialize, Deserialize` pada `QueryFilters`.
   - Menambahkan kolom taksonomi string ke `QueryFilters`:
     ```rust
     pub kingdom: Option<String>,
     pub class: Option<String>,
     pub order: Option<String>,
     pub family: Option<String>,
     pub genus: Option<String>,
     ```
2. **`crates/species-search/src/parser.rs`**
   - Mengimplementasikan kamus multi-bahasa untuk:
     - **Kelas Taksonomi**: `Mammalia`, `Reptilia`, `Aves`, `Amphibia`, `Actinopterygii`, `Insecta`, `Arachnida`, `Magnoliopsida`, `Liliopsida`, `Pinopsida`.
     - **Kingdom**: `Animalia`, `Plantae`, `Fungi`.
     - **IUCN Status**: `CR`, `EN`, `VU`, `NT`, `LC`, `DD`, `EW`, `EX` (istilah ID: `kritis`, `genting`, `terancam`, `rentan`, `hampir terancam`, `risiko rendah`, `punah di alam`, `punah`).
     - **P.106**: `Dilindungi` (`protected`), `Tidak Dilindungi` / `Bebas` (`unprotected`).
     - **Tipe Asal**: `Endemic` (`endemik`), `Native` (`asli`), `Introduced` (`introduksi`).
     - **Prefixed Syntax**: Mendukung `kategori:`, `kelas:`, `class:`, `kingdom:`, `kerajaan:`, `ordo:`, `order:`, `famili:`, `family:`, `genus:`, `status:`, `iucn:`, `p106:`, `perlindungan:`, `tipe:`, `cites:`.
   - Menambahkan fungsi publik:
     ```rust
     pub fn parse_query(raw_query: &str) -> QueryFilters
     ```
   - Menambahkan unit test lengkap untuk ekstraksi multi-bahasa, kategori, kingdom, dan arachnid.
3. **`crates/species-search/src/filter.rs`**
   - Memperluas fungsi `filter_species` agar mencocokkan field string `kingdom`, `class`, `order`, `family`, dan `genus` dari `QueryFilters` dengan `Species.taxonomy`.
4. **`crates/species-search/src/lib.rs`**
   - Mengekspos modul `pub mod types;` dan `pub mod parser;`.
   - Mengekspor `pub use parser::parse_query;` dan `pub use types::QueryFilters;`.
5. **`crates/species-search/Cargo.toml`**
   - Menambahkan dependensi `serde = { version = "1.0", features = ["derive"] }` dan `serde_json = "1.0"`.
6. **`crates/shared/src/text.rs`**
   - Mengimplementasikan fungsi normalisasi dan tokenisasi string (sebelumnya `todo!()`).
   - Merapikan `sanitize_label` sesuai standar `rustfmt`.
7. **`crates/api-server/src/routes/search.rs` & `crates/api-server/src/main.rs`**
   - Menambahkan route REST API: `GET /api/v1/search/parse` yang mengembalikan hasil serialisasi `QueryFilters`.

### C. File yang Dihapus / Diganti (Removed / Replaced)
- Menghapus stub `todo!("shared text phase 1: normalize_text")`, `todo!("shared text phase 2: tokenize")`, dan `todo!("shared text phase 3: sanitize_label")`.

---

## 3. Rincian Perubahan di Repo Portal Django (`biodiversitas_borneo_production`)

### A. File yang Ditambahkan (Added)
1. **`borneo_biodiversity_portal/apps/repository/search_parser.py`**
   - **`ParsedQuery` (Dataclass)**: Struktur data kriteria pencarian hasil parsing.
   - **`_find_rust_binary()` & `_call_rust_binary()`**: Runner otomatis binary compiled Rust parser.
   - **`_python_fallback_parser()`**: Fallback murni Python jika binary Rust tidak ada / error.
   - **`apply_species_search()`**: QuerySet filter engine untuk Django:
     - Filter relasi taksonomi ORM (`genus__family__order__class_field__phylum_division__kingdom`).
     - Filter status konservasi (`iucn`), perlindungan (`p106`), dan asal (`type`).
     - **Semantic Biological Expansion** pada `free_text`:
       * *Diet/Taraf Trofik*: `karnivora`/`predator` (Ordo Carnivora & teks predator), `herbivora` (teks pemakan tumbuhan/biji/daun/buah & ordo Proboscidea), `omnivora` (Primates).
       * *Reproduksi*: `vivipar`/`melahirkan` (Mammalia), `ovipar`/`bertelur` (Reptilia, Aves, Amphibia, Actinopterygii, Insecta).
       * *Anatomi*: `vertebrata` (Filum Chordata), `invertebrata` (Non-Chordata), `arachnid` (Arachnida).
       * *Habitat*: `darat` (terestrial), `laut`/`akuatik`/`air`/`rawa`/`mangrove`, `udara`/`terbang`/`arboreal`.
       * *Botani*: `bunga`/`berbunga` (Magnoliophyta/Angiospermae), `buah`, `pohon`, `kayu`, `daun`.
     - **Relevance Scoring**: Memberikan anotasi bobot relevansi (`exact name > startswith > contains name > description/notes`).
2. **`borneo_biodiversity_portal/apps/repository/tests/test_search_parser.py`**
   - 13 unit tests otomatis untuk menguji parser Rust, parser fallback Python, integrasi Django ORM filter, trait biologis, dan view HTTP HTMX.
3. **`bin/species_search_parser`**
   - Compiled Linux binary executable dari `kalimantan-bio/target/release/species_search_parser`.

### B. File yang Diubah (Modified)
1. **`borneo_biodiversity_portal/apps/repository/views.py` (`species_list_view`)**
   - Mengambil parameter `q`, memparsing dengan `parse_species_query`, lalu memfilter queryset dengan `apply_species_search`.
   - Mengirimkan `parsed_query` ke context template.
2. **`borneo_biodiversity_portal/templates/repository/species/_species_results.html`**
   - Menampilkan badge visual interaktif untuk filter aktif yang terdeteksi (e.g. `[Kingdom: Animalia]`, `[Kategori / Kelas: Mammalia]`, `[IUCN: Kritis (CR)]`, `[Kata Kunci: ...]`).
   - Menampilkan indikator engine parser aktif: `⚡ rust_binary` atau `🐍 python_fallback`.
3. **`borneo_biodiversity_portal/templates/repository/species/index.html`**
   - Memperbarui placeholder kotak pencarian: `"Cari spesies (e.g. mamalia kritis)..."`.
4. **`docker-compose.no-torch.yml` & `apps/identifier/services.py`**
   - Menambahkan named volume docker untuk `.venv` agar storage hemat dan tidak duplikasi.
   - Menambahkan fallback aman `try...except ImportError` untuk PyTorch agar modul pencarian tetap dapat berjalan mandiri tanpa PyTorch.

---

## 4. Spesifikasi Kontrak Data JSON (Rust $\rightarrow$ Django)

Ketika binary CLI dijalankan:
```bash
./bin/species_search_parser "mamalia kritis dilindungi"
```

Output JSON standar yang dihasilkan:
```json
{
  "free_text": [],
  "kingdom_id": null,
  "class_id": null,
  "kingdom": null,
  "class": "Mammalia",
  "order": null,
  "family": null,
  "genus": null,
  "iucn": "CR",
  "cites": null,
  "p106": "Dilindungi",
  "species_type": null,
  "is_verified": null
}
```

Jika query berisi kata kunci campuran (misal `"kategori:reptil varanus"`):
```json
{
  "free_text": ["varanus"],
  "kingdom_id": null,
  "class_id": null,
  "kingdom": null,
  "class": "Reptilia",
  "order": null,
  "family": null,
  "genus": null,
  "iucn": null,
  "cites": null,
  "p106": null,
  "species_type": null,
  "is_verified": null
}
```

---

## 5. Status CI/CD & Testing

| Komponen | Status | Hasil Test |
| :--- | :--- | :--- |
| **Rust Crate `species-search`** | ✅ PASS | 36 unit tests passed, 5 integration tests passed |
| **Rust Workspace `kalimantan-bio`** | ✅ PASS | `cargo test --workspace` passed 100% |
| **GitHub Actions CI (PR #29)** | ✅ PASS | `fmt, clippy, test` Green (SUCCESS) |
| **PR Mergeability (PR #29)** | ✅ MERGEABLE | Tidak ada conflict dengan branch `main` |
| **Django Repository Tests** | ✅ PASS | 13 test di `test_search_parser.py` dan `test_species_filters.py` lulus 100% |
