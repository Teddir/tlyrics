# TLyrics — Synchronized Audio Text & Touch Bar Companion for macOS

<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="TLyrics Logo" width="100" />
</p>

<p align="center">
  <strong>A lightweight, menu-bar-only macOS companion that displays real-time synchronized text and lyrics on your screen and physical MacBook Touch Bar.</strong>
</p>

<p align="center">
  <a href="https://github.com/Teddir/tlyrics/releases"><img src="https://img.shields.io/badge/platform-macOS%2013%2B-blue?logo=apple" alt="Platform: macOS 13+"></a>
  <a href="https://github.com/Teddir/tlyrics"><img src="https://img.shields.io/badge/arch-Apple%20Silicon%20%7C%20Intel-success" alt="Apple Silicon & Intel"></a>
  <a href="https://tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8DB?logo=tauri" alt="Tauri 2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust%202024-DEA584?logo=rust" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="MIT License"></a>
</p>

---

> ### 🌿 Catatan Fiqih & Niat Pengembang (Islamic Perspective)
> 
> *Bismillahirahmanirrahim.*
>
> Proyek ini dibangun dari sudut pandang seorang Muslim yang berpegang teguh pada pandangan mayoritas ulama salaf dan empat madzhab (*jumhur ulama*) mengenai **keharaman alat-alat musik (*ma'azif*)**. 
>
> **TLyrics** diciptakan sebagai sebuah **wasilah teknologi (alat netral)** untuk membantu pengguna mengikuti kata demi kata dari audio yang bermanfaat—seperti **nasyid tanpa instrumen musik (vocal only / acapella)**, **rekaman kajian ilmiah / khutbah**, **podcast edukatif**, **audiobook**, maupun **pembelajaran bahasa**—langsung melalui physical Touch Bar MacBook dan jendela pendamping minimalis.
>
> Meskipun TLyrics secara teknis mendukung protokol Now Playing macOS dan database LRC (seperti LRCLIB) yang umum digunakan untuk lagu/musik, pengembang berlepas diri dari penggunaan aplikasi ini untuk hal-hal yang melanggar syariat Allah ﷻ. Setiap pendengaran dan perbuatan manusia akan dimintai pertanggungjawaban di hadapan-Nya:
>
> <p align="center">
>   <strong>إِنَّ السَّمْعَ وَالْبَصَرَ وَالْفُؤَادَ كُلُّ أُولَٰئِكَ كَانَ عَنْهُ مَسْئُولًا</strong><br>
>   <em>"Sesungguhnya pendengaran, penglihatan, dan hati, semuanya itu akan dimintai pertanggungjawabannya."</em><br>
>   <strong>(QS. Al-Isra [17]: 36)</strong>
> </p>

---

## 📖 Ringkasan / Overview

**TLyrics** adalah aplikasi latar belakang (*background agent*) untuk macOS yang menyinkronkan teks dan lirik audio secara real-time. Dibangun menggunakan **Tauri 2**, **Rust**, **AppKit / Swift**, dan web standards (Vanilla JS/CSS/HTML), TLyrics menghadirkan tampilan teks tersinkronisasi pada **MacBook Touch Bar** dan *floating companion window* tanpa membebani daya baterai dan memori.

TLyrics berjalan sebagai aplikasi **macOS Menu Bar Agent** murni (`LSUIElement = true`): tanpa ikon di macOS Dock, tidak mengganggu produktivitas, dan memantau pemutaran audio di latar belakang.

---

## 🎧 Pemanfaatan Konten yang Direkomendasikan (Supported & Encouraged Content)

Aplikasi ini ditujukan untuk mempermudah membaca dan memahami konten audio yang bermanfaat:

* **Nasyid & Qasidah Tanpa Musik (Vocal Only / Acapella):** Menghafal dan menyimak syair-syair Islami yang baik.
* **Kajian Ilmiah, Khutbah & Ceramah:** Mengikuti transkrip atau poin penting materi kajian.
* **Podcast Edukasi & Diskusi:** Mengikuti percakapan audio tanpa harus berpindah jendela aplikasi.
* **Audiobook & Buku Suara:** Membantu konsentrasi membaca bersamaan dengan mendengarkan.
* **Pembelajaran Bahasa Asing:** Mencocokkan pelafalan (*listening*) dengan teks tertulis secara tepat waktu.

---

## ✨ Fitur Utama (Features)

| Fitur | Deskripsi |
| :--- | :--- |
| **🎵 Universal Now Playing Detection** | Integrasi mendalam dengan Spotube dan macOS System Media (Apple Music, Spotify, Browser media) via native `MediaRemote`. |
| **📜 Sinkronisasi LRC & LRCLIB** | Pengambilan lirik otomatis, pembersihan subtitle, pemilahan baris kronologis, dan cache lokal di `~/.cache/tlyrics/lyrics`. |
| **💻 Global Physical Touch Bar Display** | Integrasi native AppKit / Swift (`DFRFoundation` & `NSTouchBar`) menampilkan baris teks saat ini dan baris berikutnya secara terus-menerus melintasi seluruh aplikasi (Safari, VS Code, Finder, Terminal). |
| **🔌 IPC untuk Third-Party Tools** | Mengalirkan baris lirik real-time ke file `~/.cache/tlyrics/current_lyric.txt` dan `touchbar.json` untuk pengguna **BetterTouchTool (BTT)**, **MTMR**, atau **Pock**. |
| **🎛️ Menu Bar Only (Bebas Dock)** | Berjalan sebagai agen latar belakang murni (`LSUIElement = true`). Menutup jendela utama hanya menyembunyikannya (*hide*), aplikasi tetap hidup di Menu Bar. |
| **🎨 Tampilan Companion 3-Line Minimalis** | Desain modern gelap khas macOS dengan fokus 3 baris: baris sebelumnya (*faint*), baris aktif (*prominent glow*), dan baris berikutnya. Dilengkapi pengaturan ukuran font dan timing offset (+/- ms). |
| **⚡ Sangat Ringan & Efisien** | Ditulis dalam Rust murni tanpa React/Electron, penggunaan CPU mendekati 0%, pembaruan DOM hanya dilakukan saat ada pergantian baris/lagu. |

---

## 🏛️ Arsitektur & Alur Kerja

```
┌─────────────────────────────────────────────────────────────────┐
│                    Sumber Audio / Pemutar Audio                 │
│         Spotube  │  Apple Music  │  Spotify  │  Browser         │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
               macOS Native MediaRemote & Now Playing
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                   TLyrics Rust Backend (Tauri 2)                │
│  • PlaybackManager (Spotube-First Priority Routing)             │
│  • LyricsEngine (LRCLIB Client + Chronological Parser + Cache)  │
│  • SettingsManager (Persistent Preferences & Launch at Login)   │
└───────────────┬─────────────────┬───────────────────┬───────────┘
                │                 │                   │
                ▼                 ▼                   ▼
      ┌──────────────────┐ ┌───────────────┐ ┌───────────────────┐
      │ Physical TouchBar│ │ Background IPC│ │ Minimal Companion │
      │   (Swift/AppKit) │ │ (BTT/MTMR/Pock│ │   (Vanilla UI)    │
      └──────────────────┘ └───────────────┘ └───────────────────┘
```

---

## 💻 Kebutuhan Sistem (Requirements)

* **Sistem Operasi:** macOS 13.0 (Ventura), macOS 14.0 (Sonoma), atau macOS 15.0 (Sequoia) ke atas.
* **Arsitektur:** Apple Silicon (M1/M2/M3/M4) maupun Intel (x86_64).
* **Perangkat Keras:** MacBook Pro dengan physical Touch Bar (opsional, tetap berfungsi sebagai companion di layar untuk Mac tanpa Touch Bar).
* **Alat Pengembang:** Rust toolchain, Xcode Command Line Tools, dan Tauri CLI v2.

---

## 🚀 Panduan Kloning & Kompilasi (Clone & Build Guide)

Ikuti langkah-langkah di bawah ini untuk mengkloning dan membangun TLyrics dari sumber kode asli:

### 1. Persiapan Alat Pengembang (Prerequisites)

Buka aplikasi **Terminal** di Mac Anda dan pastikan dependensi berikut telah terpasang:

```bash
# 1. Pasang Xcode Command Line Tools
xcode-select --install

# 2. Pasang Rust toolchain (jika belum terpasang)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 3. Pasang Tauri CLI v2
cargo install tauri-cli --version "^2.0"
```

### 2. Kloning Repositori (Clone)

```bash
git clone https://github.com/Teddir/tlyrics.git
cd tlyrics
```

### 3. Menjalankan Mode Pengembangan (Development)

Untuk menjalankan TLyrics dalam mode pengujian langsung dengan hot-reload:

```bash
cargo tauri dev
```

*Catatan: Pada mode pengujian, activation policy secara otomatis menyembunyikan ikon di Dock dan menempatkan TLyrics di Menu Bar.*

### 4. Menjalankan Unit Tests

TLyrics dilengkapi dengan pengujian otomatis (*unit tests*) untuk memastikan parsing LRC, cache, dan deteksi Now Playing bekerja sempurna:

```bash
cd src-tauri
cargo test -- --nocapture
```

### 5. Membangun Paket Aplikasi macOS (`.app`)

Untuk menghasilkan bundel aplikasi `.app` siap pakai dengan konfigurasi `LSUIElement = true` (Menu Bar Agent):

```bash
# Versi Debug (cepat untuk dicoba)
cargo tauri build -b app --debug

# Versi Rilis / Release (dioptimasi penuh dengan LTO)
cargo tauri build -b app
```

Hasil bundel aplikasi berada di:
* Rilis: `src-tauri/target/release/bundle/macos/tlyrics.app`
* Debug: `src-tauri/target/debug/bundle/macos/tlyrics.app`

### 6. Pemasangan ke `/Applications`

Salin bundel aplikasi langsung ke folder aplikasi Mac:

```bash
cp -r src-tauri/target/release/bundle/macos/tlyrics.app /Applications/
open /Applications/tlyrics.app
```

---

## ⚙️ Penggunaan & Konfigurasi

### Menu Bar Tray
Klik ikon **TLyrics** di Menu Bar macOS untuk mengakses menu interaktif:
* **Open Lyrics:** Membuka dan memfokuskan jendela lirik 3-baris.
* **Show/Hide Window:** Menyembunyikan atau menampilkan jendela secara cepat.
* **Current Track Information:** Menampilkan judul audio dan artis yang sedang diputar secara real-time.
* **Settings:** Mengatur offset waktu lirik (+/- ms), ukuran font, start hidden, dan launch at login.
* **Enable/Disable Touch Bar:** Menghidupkan atau mematikan tampilan lirik di Touch Bar fisik.
* **Restart Lyrics Engine:** Membersihkan memori/cache dan mengambil ulang lirik dari penyedia.
* **Quit TLyrics:** Menutup seluruh proses latar belakang secara tuntas dan bersih.

### Integrasi BetterTouchTool (BTT) / MTMR
Jika Anda memakai kustomisasi Touch Bar pihak ketiga, TLyrics secara konstan menulis lirik aktif ke:
* **Teks aktif:** `~/.cache/tlyrics/current_lyric.txt`
* **Format JSON:** `~/.cache/tlyrics/touchbar.json`

**Skrip Shell untuk Widget BTT:**
```bash
cat ~/.cache/tlyrics/current_lyric.txt 2>/dev/null || echo "♪ TLyrics"
```

---

## 📜 Dalil Fiqih Mengenai Musik (The Scholarly Evidence)

Sebagai pengingat bagi sesama Muslim, berikut adalah beberapa dalil yang mendasari pandangan kami mengenai alat-alat musik:

1. **Hadits Shahih Al-Bukhari (No. 5590):**
   > Rasulullah ﷺ bersabda:
   > 
   > **لَيَكُونَنَّ مِنْ أُمَّتِي أَقْوَامٌ يَسْتَحِلُّونَ الْحِرَ وَالْحَرِيرَ وَالْخَمْرَ وَالْمَعَازِفَ**
   > 
   > *"Sungguh akan ada di kalangan umatku, kaum yang menghalalkan zina, sutra (bagi laki-laki), khamr, dan alat-alat musik (al-ma'azif)."*

2. **Tafsir Sahabat Mengenai "Lahwal Hadits" (QS. Luqman [31]: 6):**
   > Sahabat mulia Abdullah bin Mas'ud رضي الله عنه bersumpah:
   > 
   > **الْغِنَاءُ، وَاللَّهِ الَّذِي لَا إِلَهَ إِلَّا هُوَ**
   > 
   > *"Demi Allah yang tidak ada sesembahan yang berhak disembah selain Dia, yang dimaksud adalah nyanyian (dengan alat musik)."*

Maka hendaknya setiap Muslim berhati-hati dalam menjaga indra pendengarannya dan mengarahkan teknologinya untuk menyimak perkataan yang bermanfaat dan diridhai Allah ﷻ.

---

## 🌱 Niat & Doa Pengembang (Purpose & Sincere Intention)

Aplikasi ini dibangun dengan niat:
* **Ilmu yang Bermanfaat (عِلْمٌ نَافِعٌ):** Menjadikan kata-kata yang baik dan bermanfaat lebih mudah disimak dan dipahami.
* **Khidmah (خِدْمَةٌ):** Menghadirkan teknologi yang membantu kebaikan umat.
* **Ikhlas (إِخْلَاصٌ):** Dikerjakan semata-mata mengharapkan wajah Allah ﷻ.
* **Amal Jariyah (صَدَقَةٌ جَارِيَةٌ):** Berharap kebaikan teknis dari proyek ini menjadi tabungan amal shalih yang terus mengalir.

<p align="center">
  <strong>اللَّهُمَّ اجْعَلْ هَذَا الْعَمَلَ عِلْمًا نَافِعًا وَعَمَلًا صَالِحًا</strong><br>
  <em>"Ya Allah, jadikanlah karya ini sebagai ilmu yang bermanfaat dan amal yang shalih."</em>
</p>

---

## 📄 Lisensi & Hak Cipta

Didistribusikan di bawah **MIT License**. Lihat berkas [`LICENSE`](file:///Users/mac/Documents/code/tlyrics/LICENSE) untuk rincian lengkap.

Hak Cipta © 2026 **Teddi_r**. All rights reserved.

<p align="center">
  <strong>Dibangun dengan niat yang lurus. Dirancang untuk kebaikan. Dibagikan untuk kemanfaatan sesama.</strong>
</p>
