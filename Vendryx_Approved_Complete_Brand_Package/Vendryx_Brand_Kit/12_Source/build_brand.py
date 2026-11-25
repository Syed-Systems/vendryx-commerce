from pathlib import Path
from io import BytesIO
import os, math, json, csv, hashlib, zipfile, shutil, textwrap, re, colorsys
import cairosvg
from PIL import Image,ImageOps
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from xml.etree import ElementTree as ET
from reportlab.pdfgen import canvas
from reportlab.lib.colors import HexColor
from reportlab.lib.utils import ImageReader

OUT=Path('/mnt/data/Vendryx_Approved_2026_Brand_Kit')
if OUT.exists(): shutil.rmtree(OUT)
OUT.mkdir(parents=True)
ROOT=OUT
ORANGE='#FF7A00'; AMBER='#FFAF24'; DEEP='#F45605'; BURN='#DB4408'; CHAR='#0F1115'; PURE='#080A0C'; WHITE='#FFFCF8'; SLATE='#64748B'; PALE='#E5E7EB'; DARKALT='#1B1E24'
FONT_FILES={'ExtraBold':'/usr/share/fonts/opentype/inter/InterDisplay-ExtraBold.otf', 'Bold':'/usr/share/fonts/opentype/inter/InterDisplay-Bold.otf', 'Medium':'/usr/share/fonts/opentype/inter/InterDisplay-Medium.otf'}
fonts={w:TTFont(p) for w,p in FONT_FILES.items()}
files=[]

def save(relative, data):
 p=OUT/relative; p.parent.mkdir(parents=True,exist_ok=True)
 if isinstance(data,bytes): p.write_bytes(data)
 else: p.write_text(data,encoding='utf-8')
 files.append(str(relative)); return p

def txt_svg(s,x,y,width,fill=CHAR,weight='ExtraBold',spacing_units=-22):
 """Glyphs are outlined, so no runtime font dependency."""
 font=fonts[weight]; gs=font.getGlyphSet(); cmap=font.getBestCmap(); hmtx=font['hmtx'].metrics
 glyphs=[]; cursor=0
 for ch in s:
  name=cmap.get(ord(ch)); name=name or '.notdef'
  pen=SVGPathPen(gs); gs[name].draw(pen)
  glyphs.append((cursor,pen.getCommands()))
  cursor+=hmtx[name][0]+spacing_units
 maxadv=max(cursor-spacing_units,1); scale=width/maxadv
 baseline=y+font['OS/2'].sCapHeight*scale
 parts=[]
 for off,path in glyphs:
  if path: parts.append(f'<path d="{path}" transform="translate({x+off*scale:.3f} {baseline:.3f}) scale({scale:.6f} {-scale:.6f})" fill="{fill}"/>')
 return ''.join(parts),font['OS/2'].sCapHeight*scale,scale

def defs():
 return f'''<defs>
  <linearGradient id="orange-main" x1="0%" y1="0%" x2="85%" y2="100%"><stop offset="0" stop-color="{AMBER}"/><stop offset="0.48" stop-color="{ORANGE}"/><stop offset="1" stop-color="{DEEP}"/></linearGradient>
  <linearGradient id="wing-top" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0" stop-color="#FFB328"/><stop offset="0.46" stop-color="{ORANGE}"/><stop offset="1" stop-color="{DEEP}"/></linearGradient>
  <linearGradient id="wing-lower" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0" stop-color="{ORANGE}"/><stop offset="1" stop-color="{BURN}"/></linearGradient>
  <linearGradient id="fold" x1="0%" y1="0%" x2="90%" y2="100%"><stop offset="0" stop-color="#CF3B09"/><stop offset="0.58" stop-color="{DEEP}"/><stop offset="1" stop-color="{ORANGE}"/></linearGradient>
  <linearGradient id="icon-bg" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0" stop-color="#191C22"/><stop offset="0.66" stop-color="{CHAR}"/><stop offset="1" stop-color="{PURE}"/></linearGradient>
  <linearGradient id="cover-glow" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0" stop-color="#FFA52C"/><stop offset="1" stop-color="#F05408"/></linearGradient>
 </defs>'''

# Custom designed vector interpretation of the approved folded-wing orange symbol.
# Built in a fixed 300 x 250 coordinate system; shapes share a deliberate centre fold.
SYMBOL_GEOMETRY = '''
 <path d="M 14 31 H 111 C 132 31 145 41 155 62 L 179 110 L 156 149 C 146 130 137 111 127 92 L 115 72 C 110 65 104 64 91 64 H 58 C 37 64 29 57 22 44 Z" fill="url(#wing-top)"/>
 <path d="M 42 89 H 106 C 123 89 133 96 142 111 L 157 145 H 90 C 70 145 58 137 51 118 Z" fill="url(#wing-lower)"/>
 <path d="M 226 31 H 293 L 210 190 C 192 224 168 231 142 231 H 96 L 167 112 C 177 96 190 71 201 53 C 208 40 215 31 226 31 Z" fill="url(#orange-main)"/>
 <path d="M 138 134 C 147 132 158 121 167 112 L 183 87 C 182 106 171 122 158 146 L 148 164 Z" fill="url(#fold)" opacity="0.90"/>
 <path d="M 155 148 C 169 145 181 158 190 173 L 206 198 C 189 224 168 231 142 231 H 96 Z" fill="#F3630B" opacity="0.42"/>
'''
# Slight simplified mark, accessible at very small resolutions
SMALL_MARK = '''
 <path d="M 25 49 H 117 C 138 49 150 65 161 87 L 174 111 L 153 144 L 125 94 C 121 86 117 83 104 83 H 72 C 49 83 37 70 25 49 Z" fill="url(#orange-main)"/>
 <path d="M 221 49 H 277 L 204 190 C 188 217 167 225 144 225 H 107 L 182 97 C 198 66 205 49 221 49 Z" fill="url(#orange-main)"/>
'''

def mark(x,y,w, flat=None, small=False):
 h=w*250/300
 if flat:
  geometry=(SMALL_MARK if small else SYMBOL_GEOMETRY)
  geometry=re.sub(r'fill="url\(#[^)]+\)"',f'fill="{flat}"',geometry)
  geometry=re.sub(r'fill="#[0-9A-Fa-f]{6}" opacity="[0-9.]+"',f'fill="{flat}" opacity="1"',geometry)
  geometry=re.sub(r' opacity="[0-9.]+"','',geometry)
 else: geometry=SMALL_MARK if small else SYMBOL_GEOMETRY
 return f'<g transform="translate({x:.2f} {y:.2f}) scale({w/300:.6f})">{geometry}</g>'

def tile(x,y,s,mode='dark',simple=False):
 bg='url(#icon-bg)' if mode=='dark' else WHITE
 outline=f'<rect x="{x}" y="{y}" width="{s}" height="{s}" rx="{s*.205:.3f}" fill="{bg}"/>'
 # mark extent is 279x200 centered in square (300x250 coordinates)
 mw=s*0.78; mx=x+(s-mw)/2; my=y+(s-mw*250/300)/2
 return outline+mark(mx,my,mw,small=simple)

def svg(w,h,body,bg=None,desc='Vendryx logo by Syed Systems'):
 front=(f'<rect width="100%" height="100%" fill="{bg}"/>' if bg else '')
 return f'<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-label="{desc}"><title>{desc}</title>{defs()}{front}{body}</svg>'

def emit(name,content,png_widths=(),png_exact=None):
 path=save(name+'.svg',content)
 if png_exact:
  ww,hh=png_exact
  raw=cairosvg.svg2png(bytestring=content.encode(),output_width=ww,output_height=hh)
  save(name+'.png',raw)
 for width in png_widths:
  raw=cairosvg.svg2png(bytestring=content.encode(),output_width=width)
  save(f'{name}_{width}w.png',raw)
 return path

def primary(dark=False,tag=True):
 W,H=1120,310
 icon=tile(26,28,210)
 word,_,_=txt_svg('Vendryx',263,49,820,fill=WHITE if dark else CHAR)
 other=''
 if tag:
  phrase,_,_=txt_svg('B Y  S Y E D  S Y S T E M S',267,235,445,fill='#BAC4D1' if dark else SLATE,weight='Medium',spacing_units=65)
  other=phrase
 return svg(W,H,icon+word+other,desc=f'Vendryx primary {"dark" if dark else "light"} {"with" if tag else "without"} tagline')

def stacked(dark=False,tag=True):
 W,H=660,740 if tag else 675
 icon=tile(198,36,265)
 word,_,_=txt_svg('Vendryx',64,349,533,WHITE if dark else CHAR)
 extra=''
 if tag:
  extra=txt_svg('B Y  S Y E D  S Y S T E M S',139,504,378,'#BAC4D1' if dark else SLATE,'Medium',spacing_units=55)[0]
 return svg(W,H,icon+word+extra,desc='Vendryx stacked logo')

def wordmark(dark=False):
 W,H=1000,245
 return svg(W,H,txt_svg('Vendryx',21,25,955,WHITE if dark else CHAR)[0],desc='Vendryx wordmark')

def wordmark_only_tag(dark=False):
 W,H=1000,315
 ww=txt_svg('Vendryx',21,15,955,WHITE if dark else CHAR)[0]
 tg=txt_svg('B Y  S Y E D  S Y S T E M S',33,236,490,'#BAC4D1' if dark else SLATE,'Medium',spacing_units=45)[0]
 return svg(W,H,ww+tg,desc='Vendryx wordmark with company signature')

for color_mode in ['light','dark']:
 dark=color_mode=='dark'
 for tag in [True,False]:
  tail='with_by_syed_systems' if tag else 'no_tagline'
  emit(f'01_Primary_Logos/vendryx_primary_{color_mode}_{tail}',primary(dark,tag),png_widths=[600,1200,2400] if tag else [600,1200])
  emit(f'02_Secondary_Logos/vendryx_stacked_{color_mode}_{tail}',stacked(dark,tag),png_widths=[512,1024])
 emit(f'03_Wordmark/vendryx_wordmark_{color_mode}',wordmark(dark),png_widths=[512,1024,2048])
 emit(f'03_Wordmark/vendryx_wordmark_tagline_{color_mode}',wordmark_only_tag(dark),png_widths=[1024])

# Monochrome icons and wordmark black/white, useful for printing
for shade,col in [('black',CHAR),('white','#FFFFFF')]:
 m=svg(600,500,mark(0,0,600,flat=col),desc=f'Vendryx {shade} mark')
 emit(f'04_Symbol/vendryx_mark_{shade}_transparent',m,png_widths=[256,512,1024])

colored_mark=svg(600,500,mark(0,0,600),desc='Vendryx gradient lettermark transparent')
emit('04_Symbol/vendryx_lettermark_orange_transparent',colored_mark,png_widths=[128,256,512,1024])
flat_mark=svg(600,500,mark(0,0,600,flat=ORANGE),desc='Vendryx flat orange lettermark transparent')
emit('04_Symbol/vendryx_lettermark_flat_orange',flat_mark,png_widths=[256,512,1024])

for mode in ['dark','light']:
 t=svg(512,512,tile(0,0,512,mode),desc=f'Vendryx {mode} app tile')
 emit(f'04_Symbol/vendryx_tile_{mode}',t,png_widths=[128,256,512,1024])

# standard and optimized tiny favicons
favicon_svg=svg(128,128,tile(0,0,128,'dark',simple=True),desc='Vendryx small favicon')
save('05_Favicons/favicon.svg',favicon_svg)
frames=[]
for size in (16,32,48,64,96):
 data=cairosvg.svg2png(bytestring=favicon_svg.encode(),output_width=size,output_height=size)
 p=save(f'05_Favicons/favicon-{size}x{size}.png',data)
 frames.append(Image.open(BytesIO(data)).convert('RGBA'))
ico=BytesIO();frames[-1].save(ico,format='ICO',sizes=[(16,16),(32,32),(48,48),(64,64),(96,96)],append_images=frames)
save('05_Favicons/favicon.ico',ico.getvalue())
save('05_Favicons/browserconfig.xml','<?xml version="1.0"?><browserconfig><msapplication><tile><square150x150logo src="/icons/icon-192.png"/><TileColor>#0F1115</TileColor></tile></msapplication></browserconfig>')

for mode in ['dark','light']:
 for sz in [180,192,256,512,1024]:
  src=svg(sz,sz,tile(0,0,sz,mode),desc=f'Vendryx {mode} app icon {sz}px')
  emit(f'06_App_PWA/app-icon-{mode}-{sz}',src,png_exact=(sz,sz))
# maskable design: mark entirely inside safe circular central 80%, expanded foreground background
for sz in [192,512,1024]:
 body=f'<rect width="{sz}" height="{sz}" fill="{CHAR}"/>'+mark(sz*.24,sz*.325,sz*.52)
 src=svg(sz,sz,body,desc='Vendryx maskable app icon')
 emit(f'06_App_PWA/maskable-icon-{sz}',src,png_exact=(sz,sz))
save('06_App_PWA/manifest.webmanifest', json.dumps({'name':'Vendryx - Commerce Operating System','short_name':'Vendryx','description':'Commerce OS by Syed Systems','display':'standalone','background_color':CHAR,'theme_color':CHAR,'start_url':'/','icons':[{'src':'/icons/app-icon-dark-192.png','sizes':'192x192','type':'image/png','purpose':'any'},{'src':'/icons/app-icon-dark-512.png','sizes':'512x512','type':'image/png','purpose':'any'},{'src':'/icons/maskable-icon-192.png','sizes':'192x192','type':'image/png','purpose':'maskable'},{'src':'/icons/maskable-icon-512.png','sizes':'512x512','type':'image/png','purpose':'maskable'}]},indent=2))

# social designs: no image references, all reproducible SVG shapes + outlined Vendryx wordmarks

def social_lockup(x,y,s=100,mode='dark'):
 icon=tile(x,y,s,'dark')
 word=txt_svg('Vendryx',x+s+20,y+3,s*3.15,WHITE if mode=='dark' else CHAR)[0]
 tagline=txt_svg('B Y  S Y E D  S Y S T E M S',x+s+21,y+s*.71,s*1.8,'#C3CBD5' if mode=='dark' else SLATE,'Medium',spacing_units=50)[0]
 return icon+word+tagline

def display_text(lines,x,y,widths,cols,gap):
 shapes=[]; current=y
 for label,w,fill in zip(lines,widths,cols):
  path,height,_=txt_svg(label,x,current,w,fill)
  shapes.append(path)
  current+=height+gap
 return ''.join(shapes)

def cover(w,h,title_lines,desc,small=False,safe=False):
 # all key artwork left safe, abstract folded wing on right
 inner=f'<rect width="{w}" height="{h}" fill="{CHAR}"/>'
 inner+=f'<path d="M {w*.67} {-h*.15} L {w*.91} {-h*.15} L {w*.69} {h*1.08} L {w*.46} {h*1.08} Z" fill="url(#cover-glow)" opacity=".96"/>'
 inner+=f'<path d="M {w*.81} {-h*.12} L {w*1.06} {-h*.12} L {w*.84} {h*1.12} L {w*.66} {h*1.12} Z" fill="#ED5408" opacity=".48"/>'
 inner+=f'<path d="M {w*.89} {h*.33} L {w*1.09} {h*.33} L {w*.99} {h*.90} L {w*.81} {h*.90} Z" fill="#FFB12C" opacity=".28"/>'
 if h < 240:
  return svg(w,h,inner+social_lockup(w*.065,h*.24,h*.52),desc='Vendryx company social cover')
 if safe: # Youtube/very wide cover important material inside centered safe zone
  scale=max(min(w/2560,h/1440),.1)
  x=w*.28;y=h*.35; ico=h*.115
 else:
  x=w*.065;y=h*.14;ico=h*.19 if h>400 else h*.25
 inner+=social_lockup(x,y,ico)
 line_y=y+ico*1.31; gap=h*.022; max_title=w*.53
 for i,line in enumerate(title_lines):
  color=WHITE if i==0 else ORANGE
  width=min(max_title, w*.40*min(len(line)/18,1.06))
  word,height,_=txt_svg(line,x,line_y,width,color,weight='ExtraBold',spacing_units=-18)
  inner+=word;line_y+=height+gap
 if desc:
  inner+=txt_svg(desc,x,line_y+h*.035,min(w*.47,w*.36), '#D6DCE4',weight='Medium',spacing_units=-10)[0]
 return svg(w,h,inner,desc='Vendryx social banner')

socials=[
 ('linkedin-company-cover',1128,191,['Commerce.','Connected.'],'Commerce OS by Syed Systems',False),
 ('linkedin-personal-cover',1584,396,['Commerce.','Connected.'],'Vendryx by Syed Systems',False),
 ('facebook-cover',1640,624,['Build. Sell.','Scale.'],'Commerce OS by Syed Systems',False),
 ('x-twitter-header',1500,500,['Commerce','without limits.'],'Commerce OS by Syed Systems',False),
 ('youtube-channel-art',2560,1440,['Commerce.','Connected.'],'Commerce OS by Syed Systems',True),
 ('open-graph',1200,630,['Build. Sell.','Scale.'],'Vendryx | Commerce OS by Syed Systems',False),
 ('github-social-preview',1280,640,['Vendryx','Commerce OS'],'Built by Syed Systems',False),
]
for name,w,h,lines,desc,safe in socials:
 src=cover(w,h,lines,desc,safe=safe)
 emit(f'07_Social_Marketing/{name}',src,png_exact=(w,h))
for name in ['square','circular']:
 sz=1080
 if name=='circular':
  content=f'<circle cx="540" cy="540" r="540" fill="{CHAR}"/>'+mark(170,228,740)
 else:content=tile(0,0,sz,'dark')
 emit(f'07_Social_Marketing/profile-avatar-{name}',svg(sz,sz,content),png_exact=(sz,sz))

# Marketing-safe banners and background textures; scalable vector
for mode in ['dark','light']:
 w,h=1920,1080; bg=CHAR if mode=='dark' else WHITE
 body=f'<rect width="{w}" height="{h}" fill="{bg}"/>'
 body+=f'<path d="M 1260 -160 L 1770 -160 L 1150 1240 L 670 1240 Z" fill="url(#cover-glow)" opacity=".88"/>'
 body+=f'<path d="M 1650 -140 L 2120 -140 L 1450 1200 L 1170 1200 Z" fill="{DEEP}" opacity=".42"/>'
 emit(f'07_Social_Marketing/branded-background-{mode}',svg(w,h,body),png_exact=(w,h))

palette={'brand':{'orange':ORANGE,'orangeHover':'#D95700','orangeLight':AMBER,'orangeDeep':DEEP},'neutral':{'charcoal':CHAR,'pureBlack':PURE,'warmWhite':WHITE,'slate':SLATE,'lightGray':PALE,'darkSurface':DARKALT},'type':{'display':'Inter Display','body':'Inter'}}
save('09_Developer/tokens.json',json.dumps(palette,indent=2))
save('09_Developer/tokens.css',f'''/* Vendryx approved third-direction brand identity; v1.0 */
:root {{
  --vendryx-orange: {ORANGE};
  --vendryx-orange-hover: #D95700;
  --vendryx-orange-light: {AMBER};
  --vendryx-charcoal: {CHAR};
  --vendryx-warm-white: {WHITE};
  --vendryx-slate: {SLATE};
  --vendryx-border: {PALE};
  --vendryx-font-display: 'Inter Display', 'Inter', system-ui, sans-serif;
  --vendryx-font-body: 'Inter', system-ui, sans-serif;
}}
[data-theme='dark'] {{ --vendryx-surface: {CHAR}; --vendryx-text: {WHITE}; }}
[data-theme='light'] {{ --vendryx-surface: {WHITE}; --vendryx-text: {CHAR}; }}
''')
save('09_Developer/tailwind.config.example.ts', '''// Merge these tokens into the existing Tailwind config; do not replace other project settings.
export const vendryxTheme = {
  colors: {
    vendryx: {
      DEFAULT: '#FF7A00',
      hover: '#D95700',
      charcoal: '#0F1115',
      white: '#FFFCF8',
      slate: '#64748B',
      border: '#E5E7EB',
    },
  },
  fontFamily: {
    display: ['Inter Display', 'Inter', 'sans-serif'],
    sans: ['Inter', 'sans-serif'],
  },
};
''')
save('09_Developer/nextjs-icon-example.tsx', '''// Copy logo files from this kit to /public/branding/ and icon PNGs to /public/icons/.
// Next.js App Router metadata example (adjust URLs to your deployed domain).
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Vendryx | Commerce OS by Syed Systems',
  description: 'A modern commerce operating system for social and independent merchants.',
  icons: {
    icon: [{ url: '/branding/favicon.svg', type: 'image/svg+xml' }, { url: '/icons/favicon-32x32.png', sizes: '32x32' }],
    apple: [{ url: '/icons/app-icon-dark-180.png', sizes: '180x180' }],
  },
  openGraph: {
    title: 'Vendryx | Commerce OS',
    description: 'Build. Sell. Scale.',
    images: [{ url: '/branding/open-graph.png', width: 1200, height: 630 }],
  },
};
''')
save('09_Developer/IMPLEMENTATION_GUIDE.md', '''# Vendryx brand asset integration (for coding agents)

## Canonical choice
Use the **approved third revised direction**: the orange folded-wing/ribbon emblem in the charcoal app tile. This pack contains a **clean editable vector reconstruction** of the approved image concept. All production exports are derived from the same vector geometry. The provided original concept preview is not the vector source; see `11_References/`.

## Asset selection
- Site navbar on light UI: `01_Primary_Logos/vendryx_primary_light_with_by_syed_systems.svg`.
- Site navbar on dark UI: `01_Primary_Logos/vendryx_primary_dark_with_by_syed_systems.svg`.
- Narrow sidebars: `04_Symbol/vendryx_tile_dark.svg`.
- Footer / wordmark-only: `03_Wordmark/vendryx_wordmark_light.svg` or `_dark.svg`.
- Dedicated brand pages: `02_Secondary_Logos/vendryx_stacked_light_with_by_syed_systems.svg`.
- Favicon: `05_Favicons/favicon.ico`, `favicon.svg`, or size-specific PNGs.
- iOS home-screen: `06_App_PWA/app-icon-dark-180.png`.
- PWA: `06_App_PWA/manifest.webmanifest` (copy icons to paths in manifest or rewrite paths).
- Sharing cards: `07_Social_Marketing/open-graph.png`.

## Workflow
1. Copy the intended SVG and PNG assets to the repository's `public/branding` and `public/icons` folders, preserving filename clarity.
2. Wire favicon, Apple touch icon, and manifest in `app/layout.tsx` metadata or existing head implementation.
3. Use CSS custom properties from `tokens.css` and extend existing Tailwind theme with `vendryxTheme`; never replace your whole theme config blindly.
4. Select the light/dark wordmark programmatically or through CSS; **do not recolor the dark icon tile by CSS filters**.
5. Maintain natural logo aspect ratio; do not stretch. Avoid tagline in compact UI where unreadable.
6. Confirm against real UI on desktop/mobile and run a dark/light screenshot QA pass.
7. Ensure all high-level metadata and title strings spell **Vendryx**, never Vendrix or Vedrix.
8. Check responsive share preview crops and YouTube channel art safe area before publishing.

## Accessibility
- Treat images purely decorative only when adjacent visible text already names Vendryx. Otherwise use alt="Vendryx".
- `#FF7A00` is a brand accent but **not** the default fill for white-text CTA buttons; prefer darker `#B74600` for small white button text or charcoal text on the vivid orange.
- Preserve focus ring and other keyboard navigation styling.

## Source rebuild
See `12_Source/build_brand.py`. Export SVGs are fully self-contained and use glyph outlines; no font files are distributed. The rebuild script requires a licensed local Inter Display font installation.
''')

save('08_Guidelines/BRAND_GUIDELINES.md',f'''# Vendryx Brand Guidelines v1.0

**Product:** Vendryx | Commerce Operating System  
**Parent:** Syed Systems  
**Visual master:** Approved third refined orange folded-wing logo concept (vector-reconstructed)

## Brand concept
The mark has a compact orange two-wing input feeding a single rising/falling diagonal ribbon: a visual metaphor for commerce channels becoming one operating system. The shape is intentionally more abstract than a literal shopping cart; it is designed to be recognizable in an app tile. The dark square represents the stable commerce platform, while orange suggests movement, transaction flow and growth.

## Logo family
- **Primary:** tile + Vendryx outlined wordmark + `BY SYED SYSTEMS` signature. Use for marketing/navigation when space permits.
- **Secondary:** stacked tile over Vendryx wordmark + signature. Use for portrait-oriented layouts.
- **Wordmark:** word-only, no tile. Use where the product name must be foregrounded.
- **Lettermark:** standalone orange folded ribbon, transparent background. Use for product hero decoration or large brand moments; avoid as the only identification when audience is unfamiliar.
- **App icon:** orange folded ribbon inside a charcoal or warm white tile, preferred for avatars, app icons, favicons.
- **Monochrome:** black and white mark for limited color applications.

## Brand colors
| Name | Hex | Role |
|---|---|---|
| Vendryx Orange | `{ORANGE}` | accent, mark, highlights |
| Warm Amber | `{AMBER}` | gradient highlight |
| Deep Orange | `{DEEP}` | gradient lowlight |
| Charcoal | `{CHAR}` | surfaces, headlines |
| Warm White | `{WHITE}` | light surface |
| Slate Gray | `{SLATE}` | captions, secondary UI |
| Light Gray | `{PALE}` | borders |

## Typography
**Brand wordmark:** outlined from Inter Display ExtraBold, adjusted for this lockup; embedded as paths in every SVG. For UI headlines use Inter Display (or Plus Jakarta Sans as a stylistic alternative); for interface/body use Inter. Font files are intentionally not included. Respect each typeface's license.

## Clear space & sizes
- Leave free space around a complete logo at least **25% of the square tile width** on all four sides.
- Primary lockup recommended minimum **240 CSS px** wide with a tagline; below this size use no-tagline primary or symbol-only.
- Tile minimum **24 CSS px** for complex mark; at **16-32px** use the `favicon.svg` simplified artwork.
- Stacked mark recommended width **160 CSS px** or above, where text remains legible.

## Correct usage
- Match contrast: charcoal letters on light canvas, white letters on dark canvas.
- Preserve alpha channel for all transparent versions.
- Keep the mark vector-based when practical.
- Use orange consistently. Do not recolor into multiple unrelated product colors.
- Do not skew, outline, rotate, add drop shadows, or place on visually busy photos without a separate contrast field.
- Do not use the full signature line if it becomes illegible.

## Social media
Social banners are vector source plus exact-dimension PNGs. Preview how each platform crops photos/profile circles before publish. Banner copy is suggested marketing content, not product requirements.

## Status and legal notes
This pack includes a **vector reconstruction** of an image-generated approved logo direction, not the editable native file from the original image generator. The symbol and wordmark should be reviewed by the brand owner before trademark filing. No uniqueness/trademark clearance has been performed.
''')

# preserve approved concept comparison images for reference (non-production raster)
for inp,name in [('/mnt/data/vendryx_logo_lockup.png','approved-third-concept.png'),('/mnt/data/vendryx_brand_identity_guide.png','logo-family-concept-board.png')]:
 if Path(inp).exists(): save('11_References/'+name,Path(inp).read_bytes())

print('Interim created',len(files),'files')
