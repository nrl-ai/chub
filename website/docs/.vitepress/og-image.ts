/**
 * Automatic OG thumbnail generation for VitePress pages.
 *
 * Uses satori (HTML-like → SVG) + @resvg/resvg-js (SVG → PNG) to produce
 * 1200×630 Open Graph images at build time. Each page gets a branded card
 * with its title and description.
 */

import { type SiteConfig } from 'vitepress'
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import satori from 'satori'
import { Resvg } from '@resvg/resvg-js'

const WIDTH = 1200
const HEIGHT = 630

const BRAND_COLOR = '#0ea5e9'
const BG_COLOR = '#1b1b1f' // VitePress dark bg
const TEXT_COLOR = '#ffffff'
const TEXT_MUTED = '#a1a1aa'

// Inline the logo SVG as a base64 data URI for the card
const logoSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64"><rect x="4" y="4" width="56" height="56" rx="16" fill="#0ea5e9"/><polygon points="38,10 18,32 30,32 26,54 46,32 34,32" fill="#fff"/></svg>`
const logoDataUri = `data:image/svg+xml;base64,${Buffer.from(logoSvg).toString('base64')}`

/** Load bundled Inter TTF fonts from .vitepress/fonts/ (satori needs ttf/otf). */
function loadInterFont(weight: 'Regular' | 'Bold'): ArrayBuffer {
  const fontPath = resolve(__dirname, 'fonts', `Inter-${weight}.ttf`)
  return readFileSync(fontPath).buffer as ArrayBuffer
}

function buildCard(title: string, description: string, siteName: string) {
  // Truncate long descriptions
  const desc = description.length > 160 ? description.slice(0, 157) + '...' : description

  return {
    type: 'div' as const,
    props: {
      style: {
        width: WIDTH,
        height: HEIGHT,
        display: 'flex',
        flexDirection: 'column' as const,
        justifyContent: 'space-between' as const,
        padding: '60px 72px',
        backgroundColor: BG_COLOR,
        fontFamily: 'Inter',
      },
      children: [
        // Top: logo + site name
        {
          type: 'div',
          props: {
            style: { display: 'flex', alignItems: 'center', gap: '16px' },
            children: [
              {
                type: 'img',
                props: {
                  src: logoDataUri,
                  width: 48,
                  height: 48,
                  style: {},
                },
              },
              {
                type: 'span',
                props: {
                  style: {
                    fontSize: 28,
                    fontWeight: 700,
                    color: BRAND_COLOR,
                  },
                  children: siteName,
                },
              },
            ],
          },
        },
        // Middle: title + description
        {
          type: 'div',
          props: {
            style: {
              display: 'flex',
              flexDirection: 'column' as const,
              gap: '16px',
              flex: 1,
              justifyContent: 'center' as const,
            },
            children: [
              {
                type: 'div',
                props: {
                  style: {
                    fontSize: title.length > 40 ? 42 : 52,
                    fontWeight: 700,
                    color: TEXT_COLOR,
                    lineHeight: 1.2,
                    letterSpacing: '-0.02em',
                  },
                  children: title,
                },
              },
              desc
                ? {
                    type: 'div',
                    props: {
                      style: {
                        fontSize: 22,
                        color: TEXT_MUTED,
                        lineHeight: 1.5,
                      },
                      children: desc,
                    },
                  }
                : {
                    type: 'div',
                    props: {
                      style: { display: 'none' },
                      children: '',
                    },
                  },
            ],
          },
        },
        // Bottom: accent bar
        {
          type: 'div',
          props: {
            style: {
              width: 120,
              height: 4,
              backgroundColor: BRAND_COLOR,
              borderRadius: 2,
            },
            children: '',
          },
        },
      ],
    },
  }
}

/**
 * Extract the first heading and first paragraph from markdown source.
 */
function extractMeta(md: string): { title: string; description: string } {
  // Strip frontmatter
  const body = md.replace(/^---[\s\S]*?---\s*/, '')

  const titleMatch = body.match(/^#\s+(.+)$/m)
  const title = titleMatch ? titleMatch[1].trim() : ''

  // First non-empty, non-heading paragraph line after the title (skip code blocks)
  const lines = body.split('\n')
  let description = ''
  let pastTitle = !titleMatch
  let inCodeBlock = false
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed.startsWith('```')) {
      inCodeBlock = !inCodeBlock
      continue
    }
    if (inCodeBlock) continue
    if (!pastTitle) {
      if (trimmed.startsWith('# ')) pastTitle = true
      continue
    }
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith(':::')) continue
    description = trimmed.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // strip md links
    break
  }

  return { title, description }
}

let fontRegular: ArrayBuffer | null = null
let fontBold: ArrayBuffer | null = null

function ensureFonts() {
  if (!fontRegular) {
    fontRegular = loadInterFont('Regular')
    fontBold = loadInterFont('Bold')
  }
}

export async function generateOgImage(
  title: string,
  description: string,
  siteName: string,
  outputPath: string
) {
  ensureFonts()

  const card = buildCard(title, description, siteName)

  const svg = await satori(card as any, {
    width: WIDTH,
    height: HEIGHT,
    fonts: [
      { name: 'Inter', data: fontRegular!, weight: 400, style: 'normal' as const },
      { name: 'Inter', data: fontBold!, weight: 700, style: 'normal' as const },
    ],
  })

  const resvg = new Resvg(svg, {
    fitTo: { mode: 'width' as const, value: WIDTH },
  })
  const png = resvg.render().asPng()

  mkdirSync(dirname(outputPath), { recursive: true })
  writeFileSync(outputPath, png)
}

/**
 * VitePress buildEnd hook — generates OG images for all pages.
 */
export async function buildEndGenerateOgImages(siteConfig: SiteConfig) {
  const outDir = siteConfig.outDir
  const pages = siteConfig.pages
  const siteName = siteConfig.site.title || 'Chub'
  const siteDescription = siteConfig.site.description || ''

  console.log(`\n🖼  Generating OG images for ${pages.length} pages...`)

  for (const page of pages) {
    const srcPath = resolve(siteConfig.srcDir, page)
    let md: string
    try {
      md = readFileSync(srcPath, 'utf-8')
    } catch {
      continue
    }

    // Check for home layout in frontmatter
    const fmMatch = md.match(/^---\s*\n([\s\S]*?)\n---/)
    const isHome = fmMatch && /layout:\s*home/m.test(fmMatch[1])

    let title: string
    let description: string

    if (isHome) {
      // Extract hero fields from frontmatter YAML
      const heroName = fmMatch![1].match(/^\s*name:\s*(.+)$/m)
      const heroText = fmMatch![1].match(/^\s*text:\s*(.+)$/m)
      title = heroName ? heroName[1].trim() : siteName
      description = heroText ? heroText[1].trim() : siteDescription
    } else {
      ;({ title, description } = extractMeta(md))
    }

    if (!title) continue

    // Output path: og/<page>.png  (e.g., og/guide/tracking.png)
    const slug = page.replace(/\.md$/, '')
    const pngPath = resolve(outDir, 'og', `${slug}.png`)

    await generateOgImage(title, description, siteName, pngPath)
  }

  console.log(`   Done — images written to ${resolve(outDir, 'og')}/`)
}

/**
 * VitePress transformHead hook — injects OG meta tags per page.
 */
export function transformHeadOgMeta(context: any): any[] {
  const page: string = context.pageData.relativePath
  const frontmatter = context.pageData.frontmatter || {}
  const siteData = context.siteData

  const slug = page.replace(/\.md$/, '')
  const ogImagePath = `/og/${slug}.png`

  // Read page source to extract title
  let title = context.pageData.title || ''
  const description = context.pageData.description || siteData.description || ''

  // For home page
  if (frontmatter.layout === 'home') {
    title = frontmatter.hero?.name || siteData.title || 'Chub'
  }

  if (!title) return []

  const fullTitle = title === siteData.title ? title : `${title} | ${siteData.title}`

  return [
    ['meta', { property: 'og:title', content: fullTitle }],
    ['meta', { property: 'og:description', content: description }],
    ['meta', { property: 'og:image', content: ogImagePath }],
    ['meta', { property: 'og:image:width', content: '1200' }],
    ['meta', { property: 'og:image:height', content: '630' }],
    ['meta', { property: 'og:type', content: 'article' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: fullTitle }],
    ['meta', { name: 'twitter:description', content: description }],
    ['meta', { name: 'twitter:image', content: ogImagePath }],
  ]
}
