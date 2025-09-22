# 🚀 CodeHealth Analyzer  

[![Live Dashboard](https://img.shields.io/badge/Dashboard-Live-brightgreen?style=for-the-badge)](https://niazanas8.github.io/codehealth-analyzer/)  
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)  
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)  

**CodeHealth Analyzer** is a Rust-based developer tool that helps you track and manage **technical debt** in your codebase. It calculates key metrics like **Cyclomatic Complexity**, **LOC/KLOC**, and **Maintainability Index**, while also highlighting the **top offenders** in your project.  

It generates a **JSON report** and powers a **live GitHub Pages dashboard** for easy visualization.  

⚡ *Don’t let technical debt slow you down – analyze it, track it, and refactor it!*  

---

## 🌟 Features  

- 🔢 **Cyclomatic Complexity** – detect functions/files that are hard to maintain.  
- 📏 **LOC / KLOC** – measure codebase size.  
- 📉 **Maintainability Index** – track readability & maintainability (0–100 scale).  
- ⚠️ **Threshold Warnings** – fail CI/CD if complexity exceeds limits.  
- 📝 **JSON Export** – structured output with per-file & per-function details.  
- 📊 **Top Offenders List** – highlights the worst 5–20 functions by complexity.  
- 🌐 **Live Dashboard** – view charts on GitHub Pages.  

---

## 📊 Live Dashboard  

➡️ [**View Dashboard**](https://niazanas8.github.io/codehealth-analyzer/)  

Dashboard shows:  
- Summary (LOC, Functions, Complexity, Maintainability)  
- Pie chart: complexity distribution  
- Bar chart: top 10 complex functions  
- Table of offenders  

---

## 🖼️ Screenshots  

### Dashboard Example  
Here’s the live dashboard generated from `report.json`:  

![Dashboard Screenshot](docs/screenshot.png)  



---

## 🔧 Installation  

### Prerequisites  
- Rust (latest stable) installed on your system.  

### Build from Source  
```bash
git clone https://github.com/niazanas8/codehealth-analyzer.git
cd codehealth-analyzer/techdebt-tracker
cargo build --release

- **Syn**: For parsing Rust code and extracting valuable metrics. 📊

---

✨ **TechDebt-Tracker** – Track your tech debt before it tracks you! ✨
