# Source reconstruction

This source script generates the complete vector and raster logo kit. It outlines glyph paths rather than shipping or linking external font files. It requires Python, fontTools, CairoSVG, Pillow, ReportLab, and a locally installed Inter Display font. Paths in `FONT_FILES` may need adaptation for your operating system. The generator intentionally recreates the approved image concept as vector geometry rather than extracting editable vector art from the original raster.

Run only after reviewing local `FONT_FILES` and `OUT` paths: the script clears the target directory before rebuilding. Do not blindly run on a live repository.
