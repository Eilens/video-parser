# Tauri Video Parser

A powerful, cross-platform desktop application enabling seamless parsing and downloading of videos and images from popular social media platforms. Built with modern technologies for performance and user experience.

![Project Preview](./preview.png)

## 🚀 Features

- **Multi-Platform Support**:
  - **Douyin (抖音)**: Parse videos and image galleries without watermarks.
  - **Xiaohongshu (小红书)**: Extract high-quality images and videos.
  - **Weibo (微博)**: Support for video posts and massive image galleries.
  - **Pipixia (皮皮虾)**: Video extraction support.
  - **YouTube**: (Planned)
- **High Quality**: Always fetches the highest quality media available (1080p+, original images).
- **Smart Parsing**:
  - Automatically handles short links and redirects.
  - bypasses hotlink protections (Referer checks) specifically for Weibo.
  - **Video Caching**: Secure verification and playback of restricted videos via local caching.
- **Modern UI/UX**:
  - Built with **React** & **Tailwind CSS**.
  - Responsive and beautiful interface.
  - Built-in media preview (Video player & Image gallery with zoom).
  - Internationalization (i18n) support (Chinese/English).
- **Privacy & Performance**:
  - Lightweight Architecture using **Tauri v2**.
  - Rust-based backend for high-performance network requests and parsing.
  - No heavy webview scraping where possible (uses direct API analysis).

## 🛠️ Tech Stack

- **Core**: [Tauri v2](https://v2.tauri.app/) (Rust)
- **Frontend**: 
  - React 18
  - TypeScript
  - Vite
  - Tailwind CSS
  - Framer Motion (Animations)
  - Lucide React (Icons)
  - React Photo View
- **Backend (Rust)**:
  - `reqwest` (HTTP Client)
  - `tokio` (Async Runtime)
  - `serde` (Serialization)
  - `wrapper` & custom parsers

## 📦 Installation & Development

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/).
- **Node.js**: LSD version recommended.
- **Package Manager**: pnpm (recommended), npm, or yarn.

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/tauri-parse-video.git
   cd tauri-parse-video
   ```

2. **Install Frontend Dependencies**
   ```bash
   npm install
   # or
   pnpm install
   ```

3. **Run in Development Mode**
   ```bash
   npm run tauri dev
   # or
   pnpm tauri dev
   ```
   This will start the frontend server and the Tauri application window.

4. **Build for Production**
   ```bash
   npm run tauri build
   ```
   The executable will be located in `app/src-tauri/target/release/`.

## 💡 Usage

1. **Copy a link** from a supported platform (e.g., Douyin share link, Weibo post URL).
2. **Paste** the link into the input box in the application.
3. Click **Parse** (or press Enter).
4. Wait for the magic! ✨
5. **Preview** the content directly in the app.
6. Click **Download** on specific videos or images to save them to your device.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## ⚠️ Disclaimer

This project is for **educational and research purposes only**. Please respect the copyright and intellectual property rights of the content creators and platforms. Do not use this tool for any illegal activities or commercial distribution of copyrighted content.

## 📄 License

[MIT License](LICENSE)
