## ADDED Requirements

### Requirement: Create 2D text face from font file

The system SHALL create a 2D text shape (Face) from a string, a TTF/OTF font file path, and a font size in model units. Holes in glyphs (e.g., 'A', 'B', 'O', '8') SHALL be produced via boolean subtract of inner contours. The resulting shape SHALL be a Face suitable for further operations (extrude, rotate, fuse, etc.).

#### Scenario: Basic 2D text
- **WHEN** `(text "Hello" "DejaVuSans.ttf" 10)` is evaluated
- **THEN** the system returns a rojcad/shape of type FACE containing the rendered text "Hello"

#### Scenario: Text with hole-containing glyphs
- **WHEN** `(text "AO" "DejaVuSans.ttf" 10)` is evaluated
- **THEN** the system returns a FACE where glyphs 'A' and 'O' have proper holes (inner contours removed)

#### Scenario: Empty string
- **WHEN** `(text "" "DejaVuSans.ttf" 10)` is evaluated
- **THEN** the system signals an error indicating empty text

#### Scenario: Missing font file
- **WHEN** `(text "Hi" "/nonexistent/font.ttf" 10)` is evaluated
- **THEN** the system signals an error indicating font file not found

#### Scenario: Invalid font file
- **WHEN** `(text "Hi" "/etc/hosts" 10)` is evaluated
- **THEN** the system signals an error indicating invalid font data

### Requirement: Text on workplane with position

The system SHALL support workplane selection via the `:plane` keyword (same as `rect`, `circle`, `polygon`) and position via `:at` keyword.

#### Scenario: Text on XZ plane
- **WHEN** `(text "Hi" "font.ttf" 10 :plane "xz")` is evaluated
- **THEN** the system returns a FACE oriented on the XZ plane

#### Scenario: Text at specific position
- **WHEN** `(text "Hi" "font.ttf" 10 :at [5 10 0])` is evaluated
- **THEN** the system returns a FACE centered at world position (5, 10, 0)

### Requirement: Create 3D extruded text

The system SHALL create a 3D text shape (Solid) by extruding a 2D text face. The `:depth` keyword on the `text` function and the `text3d` convenience function SHALL both produce extruded text.

#### Scenario: Extruded text via :depth keyword
- **WHEN** `(text "Hi" "font.ttf" 10 :depth 5)` is evaluated
- **THEN** the system returns a rojcad/shape of type SOLID with extrusion depth 5

#### Scenario: Extruded text via text3d
- **WHEN** `(text3d "Hi" "font.ttf" 10 5)` is evaluated
- **THEN** the system returns a rojcad/shape of type SOLID with extrusion depth 5

#### Scenario: Bidirectional extrusion
- **WHEN** `(text "Hi" "font.ttf" 10 :depth 5 :both)` is evaluated
- **THEN** the system returns a SOLID extruded equally in both directions from the text plane

### Requirement: List system fonts

The system SHALL provide a function to discover and list available system fonts by scanning standard OS font directories. Each entry SHALL include the human-readable font name, the file path, and the font aspect (regular, bold, italic, bold-italic).

#### Scenario: List available fonts
- **WHEN** `(list-fonts)` is evaluated
- **THEN** the system returns an array of `[name path aspect]` tuples for all discovered TTF/OTF fonts

#### Scenario: Font entry structure
- **WHEN** the first entry of `(list-fonts)` is inspected
- **THEN** it has the structure `("Font Name" "/path/to/font.ttf" :regular)` where aspect is one of `:regular`, `:bold`, `:italic`, `:bold-italic`

### Requirement: Eager tessellation support

The system SHALL support the `:eager` keyword to trigger immediate tessellation after text shape creation, consistent with other primitives.

#### Scenario: Eager tessellation on text
- **WHEN** `(text "Hi" "font.ttf" 10 :eager)` is evaluated
- **THEN** the returned shape has tessellation data computed immediately

### Requirement: Font path resolution with system font fallback

The system SHALL resolve the font argument by trying three strategies in order: (1) direct file path, (2) filename lookup in known system font directories, (3) font name lookup in known system font directories. This allows users to pass either a full path, a bare filename like `"DejaVuSans.ttf"`, or a font name like `"Arial"`.

#### Scenario: Bare filename resolves to system font
- **WHEN** `(text "Hi" "DejaVuSans.ttf" 10)` is evaluated and a file named `DejaVuSans.ttf` exists in a standard system font directory
- **THEN** the system resolves it to the full path and returns a valid FACE

#### Scenario: Font name resolves to system font
- **WHEN** `(text "Hi" "Arial" 10)` is evaluated and a font with family name "Arial" exists in a standard system font directory
- **THEN** the system resolves it to the full path and returns a valid FACE

#### Scenario: Still fails when nothing matches
- **WHEN** `(text "Hi" "QuxZotFakeFont.xyz" 10)` is evaluated and no file or font by that name exists
- **THEN** the system signals an error indicating font not found

### Requirement: Font loading from byte buffer

The system SHALL support loading font data from a byte buffer in addition to file paths, for cases where fonts are embedded or loaded from non-filesystem sources. The Rust internal API SHALL accept both `&str` paths and `&[u8]` data.

#### Scenario: Font from bytes (Rust API)
- **WHEN** `text::load_font_from_bytes(font_data)` is called with valid TTF bytes
- **THEN** the system returns a FontData usable for text rendering

### Requirement: Overhang allowed

The system SHALL NOT clip or constrain glyph geometry that extends outside the nominal bounding box. Display fonts, swashes, and decorative glyphs that render outside their advance width or ascent/descent SHALL be rendered as-is.

#### Scenario: Display font with overhang
- **WHEN** `(text "S" "swash-font.ttf" 10)` is evaluated with a font where the 'S' glyph extends outside its bounding box
- **THEN** the returned shape includes all geometry including overhanging parts
