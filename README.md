# trein

Select area → OCR → DeepL translate — in one quick shot on Wayland.

`trein` lets you drag a rectangle on your screen, OCR the pixels with Tesseract, send the text to DeepL for translation, and print (and optionally copy) the result.

---

## Demo (what it does)

1. You select a region.
2. `grim` saves a PNG to a temp dir.
3. `tesseract` OCRs the image (language chosen from your source language).
4. The text is translated by DeepL.
5. Output is printed; optionally copied to your Wayland clipboard (`wl-copy`).

Example output:

```
=== OCR (lang: EN / eng) ===
Some extracted text from your selection...

=== Translation → PL ===
Jakieś przetłumaczone zdanie...
```

---

## Requirements

* Wayland session (`$WAYLAND_DISPLAY` set).
* Tools:

  * [`slurp`](https://github.com/emersion/slurp) (region selector)
  * [`grim`](https://github.com/emersion/grim) (screenshot)
  * [`tesseract-ocr`](https://github.com/tesseract-ocr/tesseract) + relevant language data
  * Optional: [`wl-clipboard`](https://github.com/bugaevc/wl-clipboard) (`wl-copy`) for `--copy`
* A DeepL API key (Free or Pro).

> The released binaries are **statically linked (musl)** for Linux (`x86_64` and `aarch64`).
> They **still require** the tools above to be present in `PATH` at runtime.

---

## Install / Get binaries

### 1) Prebuilt (recommended)

Grab the static binaries from **GitHub Releases**:

* `trein-<version>-linux-x86_64`
* `trein-<version>-linux-aarch64`

Make it executable and run:

```bash
chmod +x trein-*-linux-*
./trein-*-linux-* --help
```

### 2) Nix (run or install)

Run directly from the flake:

```bash
nix run github:MathieuMoalic/trein
```

Install into your profile:

```bash
nix profile install github:MathieuMoalic/trein
```

### 3) Build from source (Nix)

```bash
# builds a static (musl) binary
nix build .#trein
./result/bin/trein --help
```

---

## Configuration

Set your DeepL credentials via environment variables:

```bash
# Required:
export DEEPL_API_KEY="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx"

# Optional (defaults to Free endpoint):
#  - Free: https://api-free.deepl.com
#  - Pro:  https://api.deepl.com
export DEEPL_API_BASE="https://api.deepl.com"
```

---

## Usage

```
trein [OPTIONS]

Options:
  -s, --source-lang <CODE>     DeepL source code (default: EN)
  -t, --target-lang <CODE>     DeepL target code (default: EN)
      --copy                   Also copy translation to Wayland clipboard
      --ocr-pack <tessdata>    Override Tesseract pack (e.g., chi_tra)
  -h, --help
  -V, --version
```

### Quick starts

```bash
# Japanese → English (copy to clipboard)
trein -s JA -t EN --copy

# Chinese (auto selects chi_sim pack) → English (UK)
trein -s ZH -t EN-GB

# German → Polish
trein -s DE -t PL

# Force a specific Tesseract pack (e.g., Traditional Chinese)
trein -s ZH -t EN --ocr-pack chi_tra
```

---

## Language codes

### DeepL source codes (strict)

`AR BG CS DA DE EL EN ES ET FI FR HE HU ID IT JA KO LT LV NB NL PL PT RO RU SK SL SV TH TR UK VI ZH`

> If you pass a target-only code (e.g., `EN-GB`, `PT-BR`, `ZH-HANT`) as **source**, `trein` will tell you to use the source variant instead (`EN`, `PT`, `ZH`).

### DeepL target codes

`AR BG CS DA DE EL EN EN-GB EN-US ES ES-419 ET FI FR HE HU ID IT JA KO LT LV NB NL PL PT PT-BR PT-PT RO RU SK SL SV TH TR UK VI ZH ZH-HANS ZH-HANT`

---

## How OCR language is chosen

By default, `trein` maps your **DeepL source** to a Tesseract language pack:

| Source | Tesseract |
| -----: | :-------- |
|     AR | ara       |
|     BG | bul       |
|     CS | ces       |
|     DA | dan       |
|     DE | deu       |
|     EL | ell       |
|     EN | eng       |
|     ES | spa       |
|     ET | est       |
|     FI | fin       |
|     FR | fra       |
|     HE | heb       |
|     HU | hun       |
|     ID | ind       |
|     IT | ita       |
|     JA | jpn       |
|     KO | kor       |
|     LT | lit       |
|     LV | lav       |
|     NB | nor       |
|     NL | nld       |
|     PL | pol       |
|     PT | por       |
|     RO | ron       |
|     RU | rus       |
|     SK | slk       |
|     SL | slv       |
|     SV | swe       |
|     TH | tha       |
|     TR | tur       |
|     UK | ukr       |
|     VI | vie       |
|     ZH | chi\_sim  |

Override with `--ocr-pack` if needed.

---

## Notes & tips

* Wayland only: the app exits if `$WAYLAND_DISPLAY` isn’t set.
* Text cleanup: soft hyphens are removed, hyphenated line breaks are joined, and whitespace is collapsed before translation.
* Clipboard: `--copy` requires `wl-copy`. If missing, the app prints a tip and continues.
* DeepL endpoint:

  * Free: default (`https://api-free.deepl.com`)
  * Pro: set `DEEPL_API_BASE=https://api.deepl.com`
* Region selection: Press <kbd>Esc</kbd> to cancel the `slurp` selection.

---

## Security & privacy

* Images are written to a temporary directory and deleted when the program exits.
* Text is sent to DeepL over HTTPS. Use discretion with sensitive content.

---

## Releases & CI

GitHub Actions builds **musl** binaries for `x86_64` and `aarch64` on each `v*` tag and attaches them to the run as artifacts (and to releases if you add a publish step). Names follow:

* `trein-<version>-linux-x86_64`
* `trein-<version>-linux-aarch64`
