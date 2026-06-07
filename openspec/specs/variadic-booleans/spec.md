## ADDED Requirements

### Requirement: cut accepts multiple operand shapes
`cut` SHALL accept a tool shape followed by one or more operand shapes. Each operand SHALL be subtracted from the running result sequentially: `(cut tool b1 b2)` = `(cut (cut tool b1) b2)`. Keywords `:eager` and `:hide` SHALL apply to the final chained operation only.

If called with only a tool (zero operand shapes), the tool SHALL be returned unchanged.

#### Scenario: Cut multiple shapes from tool
- **WHEN** `(cut tool a b)` is called where tool overlaps with both `a` and `b`
- **THEN** the result is equivalent to `(cut (cut tool a) b)`

#### Scenario: Cut single shape (unchanged from current)
- **WHEN** `(cut tool a)` is called with one operand
- **THEN** the behavior is identical to the current single-arg `cut`

#### Scenario: Cut with zero operands returns tool
- **WHEN** `(cut tool)` is called with no operand shapes
- **THEN** the result is the tool shape itself (no-op)

#### Scenario: Cut with :eager keyword
- **WHEN** `(cut tool a b :eager)` is called
- **THEN** the final result is eagerly tessellated (intermediate results are not)

### Requirement: common accepts multiple shapes
`common` SHALL accept two or more shapes. The intersection SHALL be computed pairwise sequentially: `(common a b c)` = `(common (common a b) c)`. Keywords `:eager` and `:hide` SHALL apply to the final chained operation only.

If called with a single shape, it SHALL be returned unchanged.

#### Scenario: Common with three shapes
- **WHEN** `(common a b c)` is called where all three shapes share a common volume
- **THEN** the result is equivalent to `(common (common a b) c)`

#### Scenario: Common with zero operands returns the shape
- **WHEN** `(common a)` is called with one shape
- **THEN** the result is `a` unchanged

### Requirement: fuse accepts multiple shapes
`fuse` SHALL accept two or more shapes. The union SHALL be computed pairwise sequentially: `(fuse a b c)` = `(fuse (fuse a b) c)`. Keywords `:eager` and `:hide` SHALL apply to the final chained operation only.

If called with a single shape, it SHALL be returned unchanged.

#### Scenario: Fuse three shapes
- **WHEN** `(fuse a b c)` is called with three overlapping shapes
- **THEN** the result is equivalent to `(fuse (fuse a b) c)`

#### Scenario: Fuse single shape returns the shape
- **WHEN** `(fuse a)` is called with one shape
- **THEN** the result is `a` unchanged
