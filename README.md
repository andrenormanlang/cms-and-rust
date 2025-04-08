# 🚀 CMS & Rust

A modern, fast, and secure Content Management System built with Rust and MySQL.

## ✨ Features

- 📝 Create, read, update, and delete blog posts
- 🎨 Rich Markdown support for content:
  - Text formatting (bold, italic, strikethrough)
  - Code blocks with syntax highlighting
  - Tables
  - Task lists
  - Footnotes
  - Automatic HTML sanitization
- ⚡ Fast performance with Rust
- 🔒 Secure by default
- 🎯 RESTful API endpoints
- 📱 Responsive web interface

## 🛠️ Tech Stack

- Backend: Rust (Axum framework)
- Database: MariaDB
- Template Engine: MiniJinja
- Frontend: HTML, CSS

## 🚦 Getting Started

### Prerequisites

- Rust (latest stable)
- MySQL
- Cargo

### 📥 Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/cms-and-go-rust.git
cd cms-and-go-rust
```

2. Set up your database configuration in `cms_rust_config.toml`

3. Build and run the CMS app:
```bash
cd cms-and-rust-app
cargo build
cargo run
```

4. Build and run the admin app:
```bash
cd ../cms-and-rust-admin
cargo build
cargo run
```

## 🌐 Usage

- Main CMS: Visit `http://localhost:8080`
- Admin Panel: Visit `http://localhost:8081`

## 📝 API Endpoints

- `GET /` - Home page with all posts
- `GET /post/:id` - View single post
- `POST /api/posts` - Create new post (Admin)
- `DELETE /api/posts/:id` - Delete post (Admin)

## 🔐 Security

Make sure to update your database credentials and keep your `cms_rust_config.toml` file secure.

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!
