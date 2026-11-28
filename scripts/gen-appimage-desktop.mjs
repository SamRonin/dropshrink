#!/usr/bin/env node
// Writes the .desktop entry that is placed inside the Linux AppImage.
//
// Tauri's generated .desktop cannot carry the app version, so this script
// writes the complete file at build time. Version, name and description are
// read from the Tauri configuration, so nothing is hardcoded per release.
// It runs from beforeBuildCommand in src-tauri/tauri.linux.conf.json, and the
// AppImage bundler copies the result over the default entry.
//
// The X-AppImage-* keys let AppImage managers (such as AppManager and
// AppImageUpdate) identify the app and find its update source.

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = 'SamRonin/dropshrink'
const HOMEPAGE = `https://github.com/${REPO}`
const ARCH = 'x86_64'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const srcTauri = join(root, 'src-tauri')
const outDir = join(srcTauri, 'linux', 'generated')
const outFile = join(outDir, 'DropShrink.desktop')

const readJson = (file) => JSON.parse(readFileSync(join(srcTauri, file), 'utf8'))
const tauriConf = readJson('tauri.conf.json')
const linuxConf = readJson('tauri.linux.conf.json')

const name = tauriConf.productName
const version = tauriConf.version
const comment = tauriConf.bundle?.shortDescription
if (!name || !version || !comment) {
  throw new Error('tauri.conf.json must define productName, version and bundle.shortDescription')
}

// Same MIME types as the .deb entry, taken from the Linux file associations.
const mimeTypes = [...new Set((linuxConf.bundle?.fileAssociations ?? []).map((a) => a.mimeType))]
if (mimeTypes.length === 0) {
  throw new Error('tauri.linux.conf.json must list file associations with mimeType')
}

const lines = [
  '[Desktop Entry]',
  'Categories=AudioVideo;Video;',
  `Comment=${comment}`,
  'Exec=dropshrink %F',
  'StartupWMClass=dropshrink',
  'Icon=dropshrink',
  `Name=${name}`,
  'Terminal=false',
  'Type=Application',
  `MimeType=${mimeTypes.join(';')};`,
  `X-AppImage-Version=${version}`,
  `X-AppImage-Name=${name}`,
  `X-AppImage-Arch=${ARCH}`,
  `X-AppImage-Homepage=${HOMEPAGE}`,
  `X-AppImage-UpdateURL=${HOMEPAGE}`,
]

mkdirSync(outDir, { recursive: true })
writeFileSync(outFile, lines.join('\n') + '\n', 'utf8')
console.log(`Wrote ${outFile} (version ${version})`)
