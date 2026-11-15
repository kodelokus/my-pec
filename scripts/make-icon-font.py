"""Builds src/assets/fonts/pec-icons.woff2, the fallback for Segoe Fluent Icons where Windows' icon font is missing.

The interface draws its icons as Segoe Fluent Icons code points (`--icons` in src/styles.css). This font puts a
look-alike from Fluent UI System Icons (MIT, Microsoft) at each code point the interface uses, so Windows keeps
Segoe and other systems show the same icons instead of blanks. A new icon in the interface needs a line in GLYPHS.

Usage: python3 scripts/make-icon-font.py <icons folder of @fluentui/svg-icons> <output .woff2>
Needs fonttools with WOFF2 support: pip install 'fonttools[woff]'
"""

import re
import sys
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.svgLib.path import parse_path

# Segoe Fluent Icons code point -> Fluent UI System Icons name (20px, regular).
GLYPHS = {
    0xE70B: "note",  # QuickNote: open notes
    0xEA8F: "alert",  # Ringer: open alarms
    0xE909: "globe",  # World: open the companion's world
    0xE708: "weather_moon",  # QuietHours: open the dream diary
    0xE82D: "book",  # Dictionary: open memory
    0xE713: "settings",  # Settings
    0xE711: "dismiss",  # Cancel: hide a note
    0xEA39: "error_circle",  # ErrorBadge: error alert
    0xE7BA: "warning",  # Warning: warning alert
    0xE946: "info",  # Info: info alert
    0xE930: "checkmark_circle",  # Completed: success alert
}

UPM = 2048
VIEWBOX = 20


def glyph(svg: Path):
    text = svg.read_text()
    if re.search(r"fill-rule|transform|<(?!svg|path|/)", text):
        sys.exit(f"{svg.name}: only plain <path> icons convert as they are")
    pen = TTGlyphPen(None)
    # SVG y runs down from the top; the font's runs up from the baseline, with the icon filling one em above it.
    scale = UPM / VIEWBOX
    flip = TransformPen(Cu2QuPen(pen, max_err=1.0), (scale, 0, 0, -scale, 0, UPM))
    for d in re.findall(r'<path[^>]*\sd="([^"]+)"', text):
        parse_path(d, flip)
    return pen.glyph()


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__.strip().splitlines()[-2])
    icons, out = Path(sys.argv[1]), Path(sys.argv[2])

    names = [".notdef"] + [f"uni{cp:04X}" for cp in GLYPHS]
    glyphs = {".notdef": TTGlyphPen(None).glyph()}
    for cp, icon in GLYPHS.items():
        glyphs[f"uni{cp:04X}"] = glyph(icons / f"{icon}_20_regular.svg")

    fb = FontBuilder(UPM, isTTF=True)
    fb.setupGlyphOrder(names)
    fb.setupCharacterMap({cp: f"uni{cp:04X}" for cp in GLYPHS})
    fb.setupGlyf(glyphs)
    glyf = fb.font["glyf"]
    fb.setupHorizontalMetrics({n: (UPM, getattr(glyf[n], "xMin", 0)) for n in names})
    fb.setupHorizontalHeader(ascent=UPM, descent=0)
    fb.setupNameTable(
        {
            "familyName": "Pec Icons",
            "styleName": "Regular",
            "copyright": "Icons from Fluent UI System Icons, copyright Microsoft Corporation, MIT License",
        }
    )
    fb.setupOS2(sTypoAscender=UPM, sTypoDescender=0, usWinAscent=UPM, usWinDescent=0)
    fb.setupPost()
    fb.font.flavor = "woff2"
    out.parent.mkdir(parents=True, exist_ok=True)
    fb.save(out)
    print(f"{out}: {len(GLYPHS)} icons")


if __name__ == "__main__":
    main()
