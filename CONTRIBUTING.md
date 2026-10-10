<div dir="rtl">

# مشارکت در DropShrink 🤝

ممنون از علاقه‌تان به بهتر شدن DropShrink! هر کمکی — از گزارش باگ تا کدنویسی و ترجمه — ارزشمند است. این سند را مطالعه کنید تا کار همه ما راحت‌تر شود.

## رفتار ما

- محترم و سازنده باشید؛ همه در این پروژه داوطلبانه و از سر علاقه کار می‌کنند.
- انتقاد را به کد و ایده بچسبانید، نه به آدم‌ها.

## گزارش باگ 🐞

قبل از ثبت گزارش، [اینجا](https://github.com/SamRonin/dropshrink/issues) جستجو کنید تا مطمئن شوید تکراری نیست. سپس از قالب آماده **Bug report** استفاده کنید و این‌ها را بنویسید:

- نسخه DropShrink و سیستم‌عامل شما
- نسخه و منبع FFmpeg (در پایین پنجره برنامه نمایش داده می‌شود)
- مراحل دقیق بازتولید مشکل و پیام خطای کامل

## پیشنهاد ویژگی 💡

ایده‌های تازه خوش‌آمدند! از قالب **Feature request** استفاده کنید و توضیح دهید:

- چه مشکلی را حل می‌کند؟ (سناریوی واقعی بهتر از راه‌حل پیشنهادی است)
- چرا به‌نظرتان به DropShrink می‌آید؟

## شروع توسعه 🛠

پیش‌نیازها:

- **Rust** (stable، از طریق [rustup](https://rustup.rs))
- **Node.js ≥ 18** + npm
- در لینوکس:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

راه‌اندازی:

```bash
git clone https://github.com/SamRonin/dropshrink.git
cd dropshrink
npm install
npm run tauri dev
```

ساخت کامل:

```bash
npm run build        # بررسی تایپ + build فرانت‌اند
npm run tauri build  # نصب‌گرها در src-tauri/target/release/bundle/
```

## نکات فنی 📌

- **فرانت‌اند:** React 18 + TypeScript + Tailwind CSS؛ وضعیت با Zustand.
- **بک‌اند:** Rust (Tauri v2)؛ قرارداد فرمان‌ها و رخدادها در `src/types.ts` مستند است — هر تغییر در قرارداد باید در هر دو سمت اعمال شود.
- **ترجمه/i18n:** هر رشته جدید باید **هم در `src/i18n/fa.json` و هم در `src/i18n/en.json`** اضافه شود؛ برای فارسی از «» در نقل‌قول‌ها استفاده کنید.
- **RTL:** از utilityهای منطقی Tailwind (مثل `ms-*`/`me-*` به‌جای `ml-*`/`mr-*`) استفاده کنید تا رابط در هر دو جهت درست باشد.
- فایل‌های باینری (نمونه ویدیو، فونت جدید و…) را بدون هماهنگی کامیت نکنید.

## ارسال Pull Request 📝

1. یک شاخه برای کار خودتان بسازید: `git checkout -b feat/my-feature`
2. تغییرات را با کامیت‌های کوچک و شفاف ثبت کنید (ترجیحاً به سبک Conventional Commits مانند `feat:`, `fix:`, `docs:`).
3. مطمئن شوید `npm run build` بدون خطا اجرا می‌شود.
4. PR را با قالب آماده باز کنید و آن را کوچک و متمرکز نگه دارید؛ PRهای بزرگ را به چند PR کوچک تقسیم کنید.

پیش از ارسال، این چک‌لیست را مرور کنید:

- [ ] `npm run build` پاس می‌شود
- [ ] هر دو فایل زبان (`fa.json` و `en.json`) به‌روز شده‌اند
- [ ] تغییرات روی سیستم‌عامل خودم تست شده‌اند
- [ ] مستندات (README ها) در صورت نیاز به‌روز شده‌اند
- [ ] تغییرات با RTL و LTR هر دو سازگارند

</div>

---

# Contributing to DropShrink (English)

Thanks for your interest in improving DropShrink! Every kind of contribution — bug reports, code, translations — is welcome.

## Code of conduct

- Be respectful and constructive; everyone here volunteers their time.
- Critique code and ideas, not people.

## Reporting bugs 🐞

Search [existing issues](https://github.com/SamRonin/dropshrink/issues) first to avoid duplicates, then use the **Bug report** template and include:

- Your DropShrink version and OS
- The FFmpeg version & source (shown at the bottom of the app window)
- Exact reproduction steps and the full error message

## Suggesting features 💡

Use the **Feature request** template and explain:

- The problem it solves (a real-world scenario beats a proposed solution)
- Why it belongs in DropShrink

## Development setup 🛠

Prerequisites: **Rust** (stable via [rustup](https://rustup.rs)), **Node.js ≥ 18** + npm, and on Linux the webkit dependencies listed above.

```bash
git clone https://github.com/SamRonin/dropshrink.git
cd dropshrink
npm install
npm run tauri dev
```

Production build:

```bash
npm run build        # type-check + frontend build
npm run tauri build  # installers in src-tauri/target/release/bundle/
```

## Technical notes 📌

- **Frontend:** React 18 + TypeScript + Tailwind CSS; state via Zustand.
- **Backend:** Rust (Tauri v2); the command/event contract lives in `src/types.ts` — keep both sides in sync.
- **i18n:** every new string goes into **both** `src/i18n/fa.json` and `src/i18n/en.json`; use «» quotes for Persian.
- **RTL:** prefer logical Tailwind utilities (`ms-*`/`me-*` over `ml-*`/`mr-*`) so the UI works in both directions.
- Don't commit binary files (sample videos, new fonts…) without prior agreement.

## Submitting a Pull Request 📝

1. Create a branch: `git checkout -b feat/my-feature`
2. Commit in small, clear steps (Conventional Commits style: `feat:`, `fix:`, `docs:`…).
3. Make sure `npm run build` passes.
4. Open the PR using the provided template; keep it small and focused.

Checklist before submitting:

- [ ] `npm run build` passes
- [ ] Both locale files (`fa.json` and `en.json`) are updated
- [ ] Changes tested on my platform
- [ ] Docs (READMEs) updated when needed
- [ ] Works in both RTL and LTR

## License 📄

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
