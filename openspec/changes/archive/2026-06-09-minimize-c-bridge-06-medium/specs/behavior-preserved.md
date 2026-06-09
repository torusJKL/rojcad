## ADDED Requirements

### Requirement: Medium-complexity functions preserve behavior

#### Scenario: Primitives with keywords work
- **WHEN** `(sphere 10 :c [1 2 3] :a 90)` is called
- **THEN** it returns a hemisphere shape centered at [1,2,3]

#### Scenario: Booleans chain correctly
- **WHEN** `(cut box-a box-b box-c)` is called
- **THEN** box-c is subtracted from (box-a minus box-b)

#### Scenario: Transforms preserve original
- **WHEN** `(translate shape 5 0 0)` is called
- **THEN** it returns a new shape at the translated position, the original is unchanged

#### Scenario: Text creation works
- **WHEN** `(text "Hello" "font.ttf" 10 :depth 5)` is called
- **THEN** it returns an extruded 3D text shape
