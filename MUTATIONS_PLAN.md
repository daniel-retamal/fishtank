# MUTATIONS EPIC — The God Plan

> This file is the **single source of truth** for the multi-session "big mutations" effort.
> The ORIGINAL PROMPT (Section 1) is law. Nothing in it may be dropped, softened, or
> forgotten across sessions. Every session reads this file first and updates Section 7
> (Progress) + Section 8 (Next-Session Prompt) before ending.
>
> **HARD RULE:** Every session MUST end by writing the exact paste-ready prompt for the
> next session into Section 8, reflecting real progress. No exceptions.

---

## 0. Locked decisions (from the kickoff Q&A — do not relitigate)

- **Scope of the current arc:** Tier 1 (cosmetic/state) + Tier 2 (appendages). Tier 3
  (fusion: backwardstelophase, engulfment, endocytosis, telophase ability-stacking) is a
  **dedicated later arc** — but its spec still lives here verbatim and must not be lost.
- **Backwardstelophase gating & interconversion (clarified 2026-06-12 by user):** Gating is
  CAPABILITY-only — if an entity is telophase-CAPABLE it is backwardstelophase-capable (NO requirement
  to be currently telophased). Forward telophase and backwardstelophase INTERCONVERT: a single entity
  can become either; a forward-double can backwardstelophase (flip heads→tails); a backward-double can
  telophase (flip tails→heads) — "and viceversa". Model as `backwards: bool` on the double state
  (`Telophase` ⟹ `is_double=true, backwards=false`; `BackwardsTelophase` ⟹ `is_double=true,
  backwards=true`). `supports_now`: `Telophase` available unless already forward-double
  (`!is_double || backwards`); `BackwardsTelophase` available unless already backward-double
  (`!is_double || !backwards`). Cytokinesis AND endocytosis work on EITHER form.
- **Telophase ability-stacking applies to EVERY telophase (CORRECTION 2026-06-12 — supersedes the
  Section 1 "changes to the telophase" wording):** The original prompt restricted double-ability
  stacking to telophases born from *engulfment*. That was a design mistake. **There is no
  mechanical difference between a telophase from fusion and a telophase from normal duplication
  when it comes to ability stacking.** EVERY telophase entity (normal telophase,
  backwardstelophase, engulfment fusion, endocytosis) has both halves' abilities ON, stacking when
  repeated. A normally-telophased goldenfish yields 2× zoomie money; a normally-telophased
  mutantfish auto-mutates at 2× frequency; a normal telophase cow has 2 milk stacks. Engulfment /
  endocytosis matter ONLY for combining *different* species / identities — never for *whether*
  abilities stack. Architecturally: the `fused` companion (or its ability-count accounting) must be
  populated for a plain telophase too — a normal telophase = 2 components of the same species.
- **Fusion data model (Tier 3, when we get there):** *Lightweight companion struct.* Keep a
  single `FishSpecies`; add a `fused` companion (a `Vec` of components carrying extra ability
  tags w/ counts, names, stat deltas). Rendering, ability ticks, and cytokinesis read from it.
  Do **not** turn `species` into a multiset. (User chose "Lightweight companion struct".)
- **Cadence:** Fully autonomous. **One commit per mutation.** Do not wait for user input.
  No `Co-Authored-By` trailers (user preference). Build + test + clippy must pass before each
  commit. If a mutation can't cleanly apply to an entity, follow the original prompt's per-entity
  instruction; if the prompt is silent, simply do not add it to that entity and note it here.
- **No agents.** These mutations are sequential and dependency-linked and edit the same files;
  parallel agents = git conflicts, sequential agents = re-deriving context = token burn. Work
  directly; rely on harness context summarization; rely on THIS file for continuity.

---

## 1. ORIGINAL PROMPT (verbatim — THE GOD)

> I want to add more mutations. I feel we have not that many, so this will be a big sesh for
> creating the mutations. Assume this works for every mutable entity (fishes, all unfishes, cows),
> try to make it work on every entity, if not ask for directions or simply do not add that
> mutation to that entity.

### backwardstelophase
Telophase, but no heads on each extreme, but TAILS. So two butts instead of two heads. Should
follow the same rules of the telophase, but with the rear and not the head. If an entity did not
have the telophase then no backwardstelophase. But if it did, it should do this. So how cows will
now have two "w" or two udders, duplicate the milk stacks if the cow is a backwardstelophase cow.
Of course, same mechanics, backwardstelophase can be separated via cytokinesis.

example of backwardstelophase fish
```
><))))><
```

a backwardstelophase cow looks like
```
     ________
 /\/(        )\/\
    | w----w |
    ||      ||
```

I noticed that the original telophase cow has two udders "w", change those to "-", only
backwardstelophase cow has two udders.

### hydra
Hydra creates a new "head" inside the entity. It's LIKE the increaseyes mutation, but apart from
the eyes of the head, basically have eyes inside the body. For example, a backwardstelophase with
a hydra mutation could look like:
```
><))ºº))><
```
A normal fish with 2 hydra mutations could look like
```
<º))ºº)º)><
```
So we have rules for the eyes, should be 1-2 and be the same amount (initially) at the right and
left directions of the fish. It should be at least 1 cell separated from OTHER eyes, be it from
the eyes of the head, or other hydra eyes. This is really for "line-like" entities, fishes,
unfishes with fish body, and the worm. The other entities like the ball and skull have no "head"
so no hydra mutation. THE COW can have this mutation. The "HEAD" sprite is
```
^__^
(oo)
(__)
```
that can be spawned in inside the body. for example:
```
^__^   ^__^
(oo)\__(oo)__
(__)\  (__)  )\/\
    ||-----w |
    ||      ||
```
the hydra head should be at the same height of the other original heads. The new hydra head should
remain inside the boundaries of the torso, this is the lower line of "-", the head can only appear
if every cell of the new head is above those "-" and counting the udder "w", that is the torso. In
both cow and line-like fishes, if the head cannot fit inside the body, the mutation should not
occur, it should not be valid.

### feet
This mutation adds evenly spread characters below the body of the entities. The chars could either
be the `"`, or the pair `^^`, an example in fishes:
```
<º)))))><
  ^^ ^^
```
```
<º)))))><
  " " "
```
No reason to have it on the cow. For other entities, follow the same rule for the lowest row,
evenly spread it below the lowest row.

If the entity has feet, nofeet mutation should unlock on the entity and that will remove them. Make
the feet be aware of the color above (it will initially copy it) and then can be subject to other
color mutations. Consider the feet for the glistening effects, it IS a part of the entity now.

### bodyextension
This mutation will add three variants of "body extensions". This body extensions cannot be applied
if feet are on the entity, and no feet can appear if this mutation is on. I will tell how each
variant works, and how it looks on the entities. A bodyextension applied to an entity will randomly
choose one of the variants, and each new bodyextension application will increase the length of the
extension by 1. A new mutation decreaseextension will reduce the extension length, only available
to entities with the bodyextension on. The extensions disappear at the length of 0.

**Tentacle extensions:** the tentacle extensions have a gap of 1 between them, and as all variants
spread around the body. In the case of the fish, line-like entities like the unfish with fish body
and the worm, the tentacles appear below the body, an example:
```
<º)))))><
  | | |
  | | |
```
For the skull and ball, the tentacles appear both at the top and at the bottom, max 2 per side and
centered. Only on the highest or lowest rows chars. Similar to the feet spawning in, but accounting
for the top too.

In the case of the cow, the extensions will spawn in at the top of the torso (this is all cells of
chars of the cow that appear on top of the "-"s and the "W"s)
```
^__^  | | |
(oo)\_|_|_|__
(__)\        )\/\
    ||-----w |
    ||      ||
```
What makes the tentacles particular, the line of "|" should act like the seaweed plants or
tentacles from the alientank. This means sway movement, just like those plants. The starting length
is 2 as you've noticed.

**Spike extensions:** no gap between the extensions, starting length of 1, no movement. Should
spawn in JUST LIKE THE TENTACLES; but in the case of line-like entities, the extensions appear on
top and the bottom of the body. For example:
```
  ¦¦¦¦¦
<º)))))><
  ¦¦¦¦¦
```
The rest of the entities just like the tentacles, but again no gap between the spikes. But the same
spawnpoints remain. For the skull and ball of course no max extensions, but only on the designated
places.

**Wings:** just like the others, this is akin to the spike extensions. Exactly the same, but
considering the new char "/" or "\". It should mirror depending on the direction, the following
example is looking at the left.
```
  /////
<º)))))><
  \\\\\
```

### ear
This mutation adds an ear to the entities. This ear in the line-like entities appear behind the
eyes. The char for looking at the right is "3", for looking at the left it should be Ɛ. Example of
a fish looking at the left:
```
<º3)))))><
```
This mutation can be stacked, more stacks are more ears next to each other. Having ears unlocks the
"eardecrease" that reduces the ear count.

For the cow, there is no ear mutation. For the skull and balls, on the left and right edges can
ears spawn in, not making a stack to the left-right, but spawning in eyes AROUND the left-right
borders of the body. So if no space is available now left-right around the skull and ball, then no
ear can spawn in there more.

example of a full on ball full of ears
```
    Ɛ,-----.3
  Ɛ,'      ,`.3
 Ɛ/  `        \3
Ɛ; `  `      , :3
Ɛ|   '     '   |3
Ɛ:  ,        \ ;3
 Ɛ\           /3
  Ɛ`. ,   ' ,'3
    Ɛ'-----'3
```
same for the skull
```
    Ɛ.---.3
   Ɛ/     \3
 }\Ɛ       3/{
 }(Ɛ       3){
 }/Ɛ\  ^  /3\{
    Ɛ\___/3
```
notice where the skull actually overrides part of itself, so that it does not spawn in ears in the
wings. Make the ears initially have pink color.

Add new mutations for the color changing of the feet, ear, and extensions, of course only if they
are available. Similar to the eye color mutations.

### bubble color
Changes the bubble color (same palette of the color patch colors).

### nightowl
Permanent eyes closed on the entity, does not react to the coffee. Sometimes gets "sleepy" which
means slow movement and very downwards movement.

### helpedbygod
Permanent eye open. React more to the coffee. More zoomies.

nightowl and helpedbygod cancel each other, and cannot be stacked.

### heterochromia
Makes each eye color independent, changes them all to a random color, and now color of the eye
mutations apply to only one of the eyes.

### engulfment (TIER 3)
Only available on those that can do telophase. If an entity receives this mutation, in the next 10
seconds, if another of the same type of entity appears near them, they will "fuse" onto a telophase.
The "original" body will be the receiver of the engulfment mutation, and the "now head on the butt"
will be the engulfed entity. The engulfed entity will remain with their own characteristics, colors,
eyes, hydras, extensions, etc. It will be a 50/50 telophased entity with each half being the half of
their bodies. During this time, the name of the entity will be "Name of the Engulfed Mutant /
Trapped Entity", both names separated with a "/". The stats will be an average of each, the species
will be the same if they are the same, if not also do the "Species 1 / Species 2". THE ENTITY then
CAN be separated via cytokinesis, each taking away their pondered part of the stats they garnered
during their fused-together-time. Each fish will remain with its identity, name and species.

### changes to the telophase (TIER 3)
If the origin of the telophasing is the engulfment, then the newly telophase entity will have both
of their abilities ON. Meaning, a mutantfish and a candyfish telophase entity WILL have the
automutating ability of the mutantfish, AND the weight-increasing touch of the candyfish. For cows,
this means that one cow could have more than one stacking of milk. IF the ability is repeated, then
they stack. So a doublemutantfish has double frequency of auto-mutating.

> **AMENDMENT (2026-06-12, user — NOT part of the original prompt; corrects a design mistake):** The
> "if the origin of the telophasing is the engulfment" qualifier above is WRONG. Ability stacking is
> NOT exclusive to engulfment-born telophases. EVERY telophase stacks its halves' abilities — fusing
> and normal duplicating are identical for this mechanic. A goldenfish that telophases normally gets
> 2× zoomie money exactly as a goldenfish/goldenfish engulfment fusion would. Engulfment /
> endocytosis only differ in that the two halves may be *different* species/identities. See Section 0
> (locked decision) and Section 6 C4.

### endocytosis (TIER 3)
The telophased entity can resort to internal cannibalism. One entity will devour the other, and be
1 again. This new entity will be a double species, and remain with its original name. This entity
WILL BE a fusion of both entities, but only one identity. This means that through many engulfments
and endocytosis, a fish could have multiple multiple stacks of many different abilities. Or a cow
could have many many stacks of different milks. Only works in telophased entities (from engulfment
or not, and even backwardstelophase).

### Closing instruction (verbatim)
> read CLAUDE.md, clean code. we just made a refactor of mutations, so DONT fuck it up and make it
> a mess again. always clean code, clear contracts, clear interfaces, no magic numbers, you love
> design patterns. no mistakes, and less with the ASCII art.

---

## 2. Entity taxonomy (how the spec's entity classes map to code)

- **Line-like (single-row body via `Fish::segments() -> Vec<Cell>`):**
  - Standard/Alternating fish (`BodyTemplate::Standard|Alternating`) — `MutantState`-backed.
  - Fish-bodied unfish: `Reversed`, `Doppleganger`, `Phantom`, `Blinker` (slime style, render via segments path).
  - `Worm` (own builder `build_worm`, worm mutation style).
- **Multi-row blobs (no "head"):** `Ball` (9 rows), `Skull` (6 rows) — `is_multi_row()==true`,
  rendered as grids in `tank_view`, eyes are `FloatingEye`s in `UnfishState`.
- **Cow:** 5-row grid (+1 antenna row if alienated), own `cow_sprite()` in `entities/cow.rs`,
  `MutantState`-backed via `MutantBacked`.
- **Fixed-body fish (jelly/multichar):** `BodyTemplate::Fixed` — `MutantState`-backed, limited caps.

Per-class applicability of each new mutation is in Section 4.

---

## 3. Architecture map (so we never re-explore)

- **Mutation vocabulary:** `enum Mutation` in `src/fishes/mutations.rs`. Add variant → add to
  `Mutation::ALL` → add `token()` arm → `parse()` is automatic → `auto_selectable()` if it should
  be excluded from random rolls.
- **Capability sets (which entity gets which mutation):** consts in `mutations.rs`
  (`MUTANT_FULL_CAPS = Mutation::ALL`, `FIXED_MULTICHAR_CAPS`, `FIXED_JELLY_CAPS`, `SLIME_CAPS`,
  `WORM_CAPS`) and `COW_CAPS` in `entities/cow.rs`. `Fish::capabilities()` picks the right one.
- **Dynamic gating (e.g. "only if feet present"):** `Mutatable::supports_now(mutation)` in
  `mutations.rs` — the ONLY place dynamic guards live (telophase needs !double, cytokinesis needs
  double, glistenenable/disable toggle on has_glisten). Add guards for `nofeet`/`eardecrease`/
  `decreaseextension`/color-of-appendage here.
- **Effect application:** `apply_mutant_mutation::<T: MutantBacked>()` (regular/fixed fish + cow)
  and `apply_unfish_mutation()` (slime/worm) in `mutations.rs`. `Fish::apply_one` branches once.
- **Recording:** `apply_mutation()` (generic) is the ONLY place count/history is recorded.
- **Single entry:** `Tank::apply_named_mutation(name, token)` (`""`=random) in
  `src/tank/mutations.rs`. `/mutate`, void wish, auto/rad mutation all route here. Split geometry
  (`split_standard_fish`/`split_fixed_fish`/`split_worm`/`split_cow`) owned by Tank.
- **State storage:** `MutantState` (`src/fishes/mutant.rs`) for fish/cow; `UnfishState`
  (`src/fishes/unfish.rs`) for unfish (`slime_*`, `worm_*`, `floating_eyes`). New per-entity
  appendage state goes on these.
- **Eyes:** `EyeState` (`mutant.rs`) holds a `BlinkTimer` (+ will hold per-eye `Option<Color>` for
  heterochromia). `FloatingEye` (`unfish.rs`) for ball/skull.
- **Rendering:**
  - Line fish: `Fish::segments()` + `segments_mutant()`/`segments_unfish()` → a single
    `Vec<(char,Color)>` placed by `tank_view`. **Feet/extensions force this to become a
    multi-row grid** — see Section 6 (B0 scaffold).
  - Cow: `cow_sprite()` returns `Vec<Vec<Cell>>`.
  - Ball/Skull: grids built in `tank_view` from `BALL_BASE`/`SKULL_OPEN` + `floating_eyes`.
  - Shared sprite helpers: `src/sprite/mod.rs` (`opaque_line`, `mirror_char`, `mirror_grid`,
    `apply_glisten`, `TRANSPARENT='\0'`). NEVER hand-roll these.
  - Seaweed/alien-plant sway (for tentacle extension): see `src/tank/background.rs` /
    alientank plant sway — reuse the same phase-driven horizontal offset.
- **Autocomplete/hints:** `App::entity_mutations()` → `available_mutations()`; tokens via
  `commands.rs` `entity_mutation_tokens`. Free once capability + token exist.
- **Colors:** `src/colors.rs` — generic hue names ONLY (never feature-named). `random_rgb()` in
  `mutant.rs` is the color-patch palette (reused for eye/bubble/appendage colors).
- **Coffee / zoomies (for nightowl/helpedbygod):** TODO confirm location — search `coffee`,
  `zoomie`. Eye open/closed permanence interacts with `BlinkTimer`/`EyeState`.

---

## 4. Mutation → entity applicability matrix

Legend: ✅ applies · ❌ excluded (with reason) · ⚠️ special rules.

| Mutation | Line fish | Worm | Ball/Skull | Cow | Fixed/Jelly |
|---|---|---|---|---|---|
| backwardstelophase (T3) | ✅ if telophase-capable | ✅ | ❌ (no telophase) | ✅ (two udders, milk dup) | ⚠️ as telophase allows |
| hydra | ✅ | ✅ | ❌ no "head" | ✅ (head must fit above torso line) | ❌ (no head room) |
| feet | ✅ | ✅ | ✅ (lowest row, evenly spread) | ❌ ("no reason on the cow") | ✅ |
| nofeet | unlocked iff feet present | same | same | — | same |
| bodyextension (tentacle/spike/wings) | ✅ | ✅ | ✅ (top+bottom, ≤2/side centered) | ✅ tentacle/spike/wings at top of torso | ✅ |
| decreaseextension | unlocked iff extension present | same | same | same | same |
| ear | ✅ (behind eyes, 3 / Ɛ, stackable) | ✅ | ✅ (around L/R borders, until no space) | ❌ "no ear mutation" for cow | ✅? (treat as line) |
| eardecrease | unlocked iff ear present | same | same | — | same |
| feetcolor | iff feet | iff feet | iff feet | — | iff feet |
| earcolor | iff ear | iff ear | iff ear | — | iff ear |
| extensioncolor | iff extension | iff extension | iff extension | iff extension | iff extension |
| bubblecolor | ✅ (see open Q) | ✅ | ✅ | ✅ | ✅ |
| nightowl / helpedbygod | ✅ (mutually exclusive, unstackable) | ✅ | ✅ | ✅ | ✅ |
| heterochromia | ✅ (≥1 eye) | ✅ | ✅ (floating eyes) | ✅ | ⚠️ jelly may have 0 eyes → ❌ |
| engulfment / endocytosis (T3) | telophase-capable only | ✅ | ❌ | ✅ | as telophase allows |

> Note: Ear default color = **pink**. Feet default color = **copies the cell above**. Ears on
> skull must NOT spawn into the `}{ ` wing chars (skull overrides itself there — see art).

---

## 5. Open questions to resolve at implementation time (don't block; pick the documented default)

1. **bubblecolor target.** Per-entity rising bubbles don't clearly exist (bubbles look tank-wide);
   only the cow owns a speech bubble. DEFAULT until proven otherwise: store a per-entity
   `bubble_color: Option<Color>` on `MutantState`/`UnfishState`; apply it to (a) the cow speech
   bubble border, and (b) any bubble the entity emits if such emission exists. Confirm by grepping
   `bubble` in `simulation.rs`/`tank`. If genuinely no per-entity bubble, still store the field and
   tint the cow speech bubble + leave a documented hook for fish.
2. **nightowl "very downwards movement" / helpedbygod "more zoomies" magnitudes.** Use named
   constants in the entity/behavior file (no Settings). Pick conservative defaults; document here.
3. **heterochromia on jelly (0 eyes):** exclude via `supports_now` (needs ≥1 eye).
4. **ear on Fixed/Jelly fish:** treat like line (behind eye) only if it has an eye; else exclude.
5. **EAR GLYPH CONTRADICTION (resolved).** The prompt's sentence says "right → `3`, left → `Ɛ`",
   but its fish example `<º3)))))><` (labelled "looking at the left") shows `3`. The ball/skull
   art is unambiguous and agrees with the SENTENCE: left edge = `Ɛ`, right edge = `3`. DECISION:
   trust the sentence — **facing-Left ear = `Ɛ`, facing-Right ear = `3`**; the fish example's `3`
   is a typo. Add `'3' ↔ 'Ɛ'` to `sprite::mirror_char` so facing-flip mirrors correctly. Ear sits
   immediately behind the eye(s) (between eyes and body on the head side); stacking adds more
   adjacent ears. Default color PINK; `earcolor` recolors later.
6. **WIDTH GOTCHA for in-row appendages (ear, hydra).** Ears/hydra-eyes inserted into the body row
   widen the entity → `display_width` must account for them EVERYWHERE: `MutantState::display_width`,
   the fixed-body width math, `Fish::recompute_display_width`, `cow_display_width`, and the worm/
   width helpers. Miss one and the fish clips or leaves gaps. Store counts on `MutantState`/
   `UnfishState`; fold into every width computation in the same commit.
7. **Ball/skull ears are a distinct render job** (per-row glyphs around the L/R border, must NOT
   overwrite the skull `}{` wing chars — see art). Recommend splitting B1 into B1a (line/worm ears)
   and B1b (ball/skull border ears) as two commits to keep each correct and reviewable.

---

## 6. Implementation order (dependency-sound) & per-step checklist

Each step = its own commit. Per step: read every file you'll touch → implement → `cargo build`
→ `cargo test` → `cargo clippy && cargo fmt` → commit (`feat(mut): <name> …`, no Co-Authored-By).
ZERO comments in `.rs`. Named constants for every literal. Update Section 7 after each.

### Phase A — Tier 1 (state/color, no new geometry)
- [x] **A1 bubblecolor** — `Mutation::BubbleColor`; `bubble_color: Option<Color>` on `MutantState`
      + `UnfishState`; apply arms; added to all cap lists incl. `COW_CAPS`; fish emitted-bubble
      override in `simulation.rs`; cow speech-bubble tint in `tank_view.rs`. Commit after this line.
- [x] **A2 nightowl / helpedbygod** — `enum Circadian {Neutral,NightOwl,HelpedByGod}` on `MutantState`
      + `UnfishState`; `Mutatable::circadian()` accessor + `supports_now` guards (each available only
      if not already that state → cancel/unstack). Eyes forced via `MutantState::tick_eyes` +
      `UnfishState::tick` (`EyeState::set_open`, `BlinkTimer.is_open`). Behavior in `Fish::tick`:
      `coffee_response` (nightowl→0, helpedbygod→×2), `circadian_zoomie_mult` (nightowl→0,
      helpedbygod→×3), nightowl slow (×0.4) + downward sink (+1.5 dy). +2 tests.
      DOCS: "sometimes sleepy" simplified to a constant calm slow+sink (no new state machine).
- [x] **A3 heterochromia** — `EyeState.color: Option<Color>` + `MutantState.heterochromia` +
      `FloatingEye.color` + `UnfishState.heterochromia`. `Heterochromia` randomizes every eye and sets
      the flag; `EyeColor` recolors ONE eye when heterochromatic (else global); `BodyColor`
      re-randomizes per-eye when heterochromatic. Render: `apply_patches_and_eyes` now takes
      `&[Option<Color>]` per eye-range (4 call sites: live + static, fixed + standard); cow rows +
      double-cow per-eye via `MutantState::eye_render_color`; ball/skull floating eyes via `eye.color`.
      Caps: in FULL/FIXED_MULTICHAR/SLIME/COW; EXCLUDED from FIXED_JELLY (no eyes) + WORM (count-based
      eyes). +4 tests. Open Q3 resolved (jelly excluded). Worm/slime-fish documented as degenerate.

### Phase B — Tier 2 (appendages)
- [x] **B1a ear (+ eardecrease + earcolor) for line-like fish + worm** — `Ɛ` (facing-Left) / `3`
      (facing-Right) spliced immediately behind the eye(s) on standard/alternating mutant fish, the
      slime-style fish-bodied unfish (reversed/doppleganger/phantom/blinker), and the worm. Stackable
      to `MAX_EARS=4`; eardecrease/earcolor gated in `supports_now` on `ear_count() > 0`. Default pink
      via `ear_color.unwrap_or(PINK)`. `ear_count`/`ear_color` on `MutantState` + `UnfishState`; folded
      into `MutantState::display_width`, `worm_display_width`/`worm_eye_cols`/`build_worm` (now take
      `ears`), the slime recompute in `apply_unfish_mutation`, and both cytokinesis split sites. Helpers
      `insert_ears`/`insert_line_ears` in `fish.rs`; `'3' ↔ 'Ɛ'` added to `sprite::mirror_char` (no
      sprite art contains a literal `3`, verified). +4 tests incl. rendered-width invariant.
      ALSO (pre-req fix): **worm heterochromia** — A3 wrongly excluded the worm; the matrix lists it ✅.
      Added `worm_eye_colors: Vec<Option<Color>>` (per-eye, indexed by `worm_eye_cols` order),
      `worm_eye_count`/`resync_worm_eye_colors`/`worm_eye_render_color`; `Heterochromia` added to
      `WORM_CAPS`; resync runs on every worm count change. +2 tests.
- [x] **B1b ball/skull border ears** — DONE. One ear per decorated border row, filled top-to-bottom:
      `Ɛ` left, `3` right. `tank_view::ear_border_cols` traces each row — ball follows the contour
      (ears overhang ±1, clipped at walls, fixed BALL_WIDTH unchanged); skull places ears at the inner
      body wall (`SKULL_WING_WIDTH=2` from each edge) so the `}{` wings are never overwritten. Per-entity
      cap via new `Mutatable::max_ears()` (BALL_HEIGHT / SKULL_HEIGHT rows; `MAX_EARS` for line). Rendered
      once in `render_multi_row_unfish_at` → live + show/index overlays all covered. +4 tests.
      **→ B1 (ear) fully complete across all eligible entities; cow excluded by design.**
- [x] **B3 hydra** — DONE for line fish + slime unfish + worm + cow. Extra in-body eyes woven into the
      body run (≥1 cell from every other eye, symmetric across facing); cow spawns a whole `^__^/(oo)/(__)`
      head inside the torso with a fits-above-the-line validity/capacity check. Ball/skull/fixed-jelly
      excluded. Reused `EyeState` (`hydra_eyes` on `MutantState`, `hydra_count` on `UnfishState`); folded
      into every display-width path + cytokinesis/size truncation. Split across 4 commits (line / slime /
      worm / cow), each build+test+clippy+fmt green.
- [x] **B0 multi-row line-fish scaffold** — DONE. `LineSprite { rows: Vec<Vec<Cell>>, body_row }` +
      `Fish::line_sprite()` (today a one-row grid wrapping `segments()`; the seam B2/B4 extend with
      above/below rows). `render_fish`'s line case now routes through one grid drawer
      `render_line_sprite` (anchors `body_row` at `position.y`, skips `sprite::TRANSPARENT` gaps, clips
      top+bottom+left+right) — no second renderer forked. No-op for single-row fish, locked by a
      byte-identical test (plain Merluza/Salmon/Koi + Mutantfish + slime Reversed, both facings). NOTE:
      `sprite_y_margins`/bounce untouched (still `(0,0)` for line fish — no appendages yet); B2/B4 must
      add a lightweight `appendage_extent()` (reading counts, NOT building the sprite) and fold it into
      `sprite_y_margins` so above/below rows stay inside the tank. Static overlays still use
      `static_left_segments()` — B2 adds a `static_left_sprite()` symmetric path when feet must show there.
- [x] **B2 feet (+ nofeet + feetcolor)** — DONE. `"` or `^^` evenly below lowest row; copies color above;
      glisten-aware. Cow excluded. Split across 4 commits: B2a (infra + mutant/fixed line via `line_sprite`),
      B2b (slime unfish + ball/skull multi-row), B2c (worm portal), B2d (`render_fish_sprite` shared helper
      → feet in `/show` + catch overlays). Feet placement is single-sourced in `sprite::feet_row`
      (painted-span inset 1; quote=`"` every-other; caret=`^^` pairs gap-1; reproduces the spec art for a
      5-wide body). `appendage_extent` folds the below-row into `sprite_y_margins`. Width never changes.
      Index table + shop preview intentionally excluded (compact table / shows only unmutated display fish).
      **STILL OWED to B4:** the feet↔bodyextension mutual-exclusion guard in `supports_now` (both
      directions) — bodyextension didn't exist during B2, so the guard is currently feet-only (`!has_feet`).
- [x] **B4 bodyextension (tentacle/spike/wings) (+ decreaseextension + extensioncolor)** — DONE across all
      eligible entities. Variant chosen at first application; length grows per application; tentacle sways
      (reuses `sway_x_offset` seaweed math), spike/wings static; per-class spawn rules (line/worm: tentacle
      below / spike+wings top&bottom; ball/skull: top+bottom ≤2/side centered; cow: top of torso). Mutually
      exclusive with feet (both directions, in `supports_now`). Width never changes. Split B4a (infra + line/
      fixed) → B4b (slime + ball/skull) → B4c (worm) → B4d (cow), each build+test+clippy+fmt green.
      **→ Phase B (Tier 2 appendages) COMPLETE.**

### Phase C — Tier 3 (fusion) — SEPARATE ARC (spec preserved in Section 1)
- [x] **C1 telophase udder fix** — DONE. `double_cow_sprite` row4 now draws `-` (plain belly) instead
      of the two `w` udders; single cow keeps its one `w` udder. Glyphs named (`COW_UDDER='w'`,
      `COW_BELLY='-'`). C2 will re-introduce the two `w` udders + milk duplication ONLY for
      backwardstelophase. +2 cow-sprite tests (telophase row has no `w`; single cow has exactly one).
- [x] **C2 backwardstelophase** — DONE across all telophase-capable entities (C2a standard fish, C2b fixed
      jelly+multichar, C2c worm, C2d cow). Tails on both extremes; interconverts with telophase; cytokinesis
      splits either form. Cow: two `w` udders + DOUBLED milk yield. **→ C2 COMPLETE.**
- [x] **C4 telophase ability-stacking (ALL telophases)** — DONE (C4a+C4b). Lightweight `fused`
      companion (`Vec<FusedComponent>` in `src/fishes/fused.rs`, on `MutantState` + `UnfishState`).
      `seed_double` seeds it with 2 same-species components on every telophase/backwardstelophase (fish,
      worm, cow); seeded only when EMPTY, preserved across telophase-flips and endocytosis — cytokinesis
      is the ONLY clearer. Cow `milk_yield` = `fused.len().max(1)` (subsumes the C2 backwards case). Fish
      `ability_components/ability_stacks/auto_mutate_stacks` scale goldenfish cash, candyfish touch,
      mutantfish auto-mutate frequency + rad weight by component count (capability-driven via each
      component's `auto_mutate` flag). `MutantBacked::self_component()` hook.
- [x] **C5 endocytosis** — DONE. `Mutation::Endocytosis` (telophase-capable caps; `supports_now` needs
      `is_double`). Collapses double→single WITHOUT clearing `fused`, so abilities stay stacked
      (single-bodied 2× cash, endocytosed cow keeps 2 milk). Standard/fixed fish, worm, cow.
- [x] **C3 engulfment** — DONE for standard fish + cows. `Mutation::Engulfment` arms a 10s `engulf_timer`
      (on `Fish`/`Cow`). `Tank::tick_engulfment` scans for a same-type entity within reach; on contact the
      receiver telophases, `fused` holds BOTH identities, name → "A / B", weights averaged, engulfed body
      absorbed. Heterogeneous cytokinesis (`try_split_engulfment_fish/_cow`) restores each original
      name/species/weight. Candyfish touch made ability-stack-driven so a heterogeneous fusion keeps both
      passives. DOC: fusion is mechanically faithful (identity/stats/abilities/re-split); the fused sprite
      renders as the receiver's telophase, not a per-half blend (documented simplification). Worm/fixed
      engulfment deferred (only standard fish + cows are engulf-capable).
      **→ Phase C (Tier 3 fusion) COMPLETE. The whole MUTATIONS epic is DONE.**

---

## 7. PROGRESS LOG (update every commit)

- 2026-06-09: Read full mutation architecture. Found dirty working tree (mutation-refactor WIP,
  builds clean, 84 tests green). Committed it as checkpoint `8961a35`. Wrote this plan.
- 2026-06-09: **A1 bubblecolor** done. Open Q1 resolved: per-fish bubbles DO exist (emitted on
  Zoomie in `simulation.rs`), cows have speech bubbles. `bubble_color` overrides emitted fish
  bubble color and tints cow speech bubble. +2 tests (104 total). Builds/clippy/fmt clean.
- 2026-06-09: **A2 nightowl / helpedbygod** done. `Circadian` enum, mutual cancel + unstack via
  `supports_now`, forced eyes, coffee/zoomie/movement effects in `Fish::tick`. Open Q2 resolved:
  nightowl = constant slow + sink (documented simplification). +2 tests (108 total).
- 2026-06-09: **A3 heterochromia** done. Per-eye color model (`EyeState.color`, `FloatingEye.color`),
  flag on `MutantState`/`UnfishState`, all 4 `apply_patches_and_eyes` sites converted to per-eye
  `&[Option<Color>]`, cow + ball/skull renders per-eye. Excluded from jelly/worm. +4 tests (112 total).
  **Phase A (Tier 1) COMPLETE.**
- 2026-06-09: **worm heterochromia** fix (own commit). A3 had excluded the worm though the matrix
  lists it ✅. Added per-eye `worm_eye_colors` indexed by `worm_eye_cols` order, resynced on every
  count change; `Heterochromia` added to `WORM_CAPS`; render falls back to slime_eye_color→default.
  Replaced the `worm_rejects_heterochromia` test with two acceptance tests. +2 tests.
- 2026-06-09: **B1a ear (+ eardecrease + earcolor)** done for line-like fish + worm (own commit).
  Glyph `Ɛ`/`3` behind the eye(s); slime variants insert by eye-index, mutant path splices head-first
  pre-reverse, worm bakes ears into the sprite. `ear_count` folded into ALL width paths + both split
  sites; `'3'↔'Ɛ'` in `sprite::mirror_char`. Cow + fixed/jelly excluded. +4 tests incl. a
  rendered-width invariant (confirms `Ɛ` is display-width 1 — landmine #6 closed for line ears).
- 2026-06-09: **fix** — B1a's slime width recompute was clobbering ball/skull's fixed width on every
  mutation; guarded to `!is_multi_row`. +1 test.
- 2026-06-09: **B1b ball/skull border ears** done (own commit). `ear_border_cols` (contour for ball,
  inner-wall for skull so `}{` wings survive), per-entity `Mutatable::max_ears()`, rendered in
  `render_multi_row_unfish_at`. +4 tests. **B1 (ear) now complete for every eligible entity.**
- 2026-06-09: **B3a-line hydra** done (own commit). `Mutation::Hydra` + `hydra_eyes: Vec<EyeState>`
  on `MutantState`; standard/alternating fish only (via `ALL`/`MUTANT_FULL_CAPS`; NOT yet in
  slime/worm/cow caps). 1–2 eyes per application woven into the body run via `insert_line_hydra` +
  `even_indices` (interior slots → guaranteed ≥1 cell from every other eye, both facings, both
  live + static render paths). Folded into `MutantState::display_width`, `all_eyes_mut`,
  `randomize_one_eye_color`. Capacity = `body_size - 1`; gated in `supports_now` via new
  `Mutatable::hydra_count/hydra_max`; truncated on `sizedecrease` and in `split_standard_fish` so
  display_width always equals rendered width. +4 tests (110 total).
- 2026-06-09: **B3a-slime hydra** done (own commit). `hydra_count: usize` on `UnfishState`;
  `Hydra` added to `SLIME_CAPS`. New `insert_line_appendages` helper places ears + hydra eyes in
  one descending-sorted pass (no eye-index drift) for the reversed/doppleganger/phantom/blinker
  paths (live + static). Hydra eye = `º` @ `UNFISH_EYE_COLOR` (matches the slime head eye). Folded
  into the slime width path + `Fish::hydra_count/hydra_max`; ball/skull excluded. +3 tests.
- 2026-06-09: **B3a-worm hydra** done (own commit). `Hydra` in `WORM_CAPS`; threaded a `hydra`
  param through `build_worm`/`worm_eye_cols`/`worm_display_width`. `0`-glyph eyes spliced between
  body segments via shared `worm_hydra_boundaries` (sprite + col math single-sourced; col_j =
  body_start + 3·b_j + j). `worm_eye_count` now counts hydra so heterochromia resyncs them.
  Capacity = `segments-1`; truncated on `sizedecrease` + per-half in `split_worm`. `even_indices`
  moved to `util`. +2 tests. **B3a (line + slime + worm hydra) COMPLETE.**
- 2026-06-09: **B3b-cow hydra** done (own commit). Cow reuses `mutant.hydra_eyes` (each `EyeState`
  drives one head's `oo` blink) and stamps a `^__^/(oo)/(__)` head into the torso above the `-w`
  line via `overlay_hydra_heads`. Capacity = `(torso+1)/(HEAD_W+GAP)` (0 while doubled / too small =
  INVALID per spec). Heads overlay existing torso cells → cow width unchanged. New
  `MutantBacked::hydra_capacity` hook unifies per-type fill/truncation (line=`body_size-1`,
  cow=head-fit). +4 tests. **B3 (hydra) COMPLETE for every eligible entity; ball/skull/jelly excluded.**
- 2026-06-12: **B0 multi-row line-fish scaffold** done (own commit `fb690b1`). `LineSprite { rows,
  body_row }` + `Fish::line_sprite()` (one-row grid wrapping `segments()` for now). `render_fish` line
  case routed through single `render_line_sprite` grid drawer (anchors body row at `position.y`, skips
  `sprite::TRANSPARENT`, clips all four edges). No-op for single-row fish; byte-identical test across
  plain/mutant/slime line fish in both facings. +1 test (275 total). `sprite_y_margins`/bounce left at
  `(0,0)` for line fish (no appendages yet — B2/B4 fold extents in).
- 2026-06-12: **B2a feet infra + mutant/fixed line fish** done (own commit). `Mutation::Feet/NoFeet/
  FeetColor`; `enum FeetStyle {Quote,Caret}` + `struct Feet {style,color}` + `sprite::feet_row(body, feet)`
  (single-sourced placement: painted-span inset 1, quote=`"` every-other via `even_indices`, caret=`^^`
  pairs with gap-1, both reproduce the spec example exactly for a 5-wide body). `feet: Option<Feet>` on
  `MutantState` + `UnfishState` (latter unused until B2b). Caps: FULL (via ALL) + FIXED_MULTICHAR +
  FIXED_JELLY; cow EXCLUDED. `roll_feet` picks the variant once at first application. `Mutatable::has_feet`
  + `supports_now` gates (Feet iff !has_feet; NoFeet/FeetColor iff has_feet). `Fish::feet()` accessor;
  `line_sprite()` appends one feet row below the body (color copied per-column from the cell above unless
  `feetcolor` overrides → glisten-aware for free). `appendage_extent()` seam folded into `sprite_y_margins`
  so the feet row stays in-tank at the bottom. Width unaffected (feet sit below). +3 tests (110 total).
  DOC: footable span = painted-span inset 1 (documented default; feet may sit under eye/tail-tip by one
  cell vs the idealized art — clean & general across all entities). **B4 must add the feet↔bodyextension
  mutual-exclusion guard in `supports_now` (bodyextension doesn't exist yet).**
- 2026-06-12: **B2b feet for slime unfish + ball/skull** done (own commit). `feet` wired on
  `UnfishState`; `Feet/NoFeet/FeetColor` added to `SLIME_CAPS` (shared by slime-line AND ball/skull) +
  `apply_unfish_mutation` arm (uses the same `roll_feet`). Slime-line unfish (reversed/doppleganger/
  phantom/blinker) get feet FOR FREE via `line_sprite()` + the `feet()` accessor. Ball/skull render a feet
  row below their lowest grid line via new `tank_view::render_feet_row` (builds the lowest line's cells,
  copies per-column color from glisten/patch/body, then `sprite::feet_row`); `appendage_extent` already
  reserves the below-row in `sprite_y_margins` for unfish too. +3 tests (slime line_sprite feet, ball
  feet width-stable + nofeet, ball feet render on the row below). Width unchanged everywhere.
- 2026-06-12: **B2c feet for worm** done (own commit). `Feet/NoFeet/FeetColor` added to `WORM_CAPS`
  (the apply arm in `apply_unfish_mutation` is style-agnostic, so worm reuses it). `render_worm_portal`
  now stamps a `sprite::feet_row` one row below the worm body, wrapped with the same `rem_euclid` portal
  math as the body. Width untouched (`worm_display_width` never sees feet). +2 tests (worm width-stable +
  feetcolor; worm render: carets one row below, never on the body row). **B2 feet now renders LIVE on
  every eligible entity (line fish, slime unfish, ball/skull, worm); cow excluded by design.** Remaining:
  B2d static overlays (show/index/shop/catch still render the body row only — feet absent there).
- 2026-06-12: **B2d static-overlay feet** done (own commit). New shared `ui::render_fish_sprite(LineSprite)`
  draws the body row at the anchor and any appendage rows relative to `body_row` (skips `TRANSPARENT`).
  `/show` detail card and the fishing catch panel now render `fish.line_sprite()` instead of `segments()`,
  so feet appear in those previews (ball/skull already showed feet via `render_multi_row_unfish_at`). Index
  table + shop preview deliberately left on `render_fish_segs`/single-row (index is a compact one-line-per-
  fish table; shop shows only unmutated `new_for_display` fish that can never have feet) — so `static_left_
  sprite()` was NOT needed and is not added. +1 helper test. **B2 feet COMPLETE across every eligible
  entity + every surface that showcases a fish.**
- 2026-06-12: **B4a bodyextension infra + line/fixed fish** done (own commit). `enum ExtensionVariant
  {Tentacle,Spike,Wing}` (+ `start_length`: 2/1/1, `sways`) and `struct BodyExtension {variant,length,color}`
  in `src/sprite/`. ONE shared `sprite::extension_row(body, ext, depth, top, facing_left, phase)` places
  all three: tentacle = `|` at `even_indices` spread (gap-1 / stride-2) swayed via `sway_x_offset` (reused
  seaweed math, depth-based ratio so the tip swings most); spike = contiguous `¦`; wing = contiguous `/`↔`\`
  (top vs bottom mirror via `mirror_char`, also flips by facing). Painted-span inset 1 single-sourced in new
  `sprite::painted_span` (feet_row now uses it too). `Mutation::BodyExtension/DecreaseExtension/ExtensionColor`
  + tokens + ALL; `roll_extension` picks the variant once; `grow_extension`/`shrink_extension` (vanishes at
  len 0, clamps to `EXTENSION_MAX_LENGTH=8`). `body_extension: Option<BodyExtension>` on `MutantState` +
  `UnfishState`; apply arms for both mutant + unfish paths. Caps in FULL(=ALL)+FIXED_MULTICHAR+FIXED_JELLY
  (slime/worm/cow deferred to B4b–d). **OWED debt PAID:** `Mutatable::has_bodyextension()` added; `supports_now`
  now enforces feet↔extension mutual exclusion BOTH ways (`Feet => !has_feet && !has_bodyextension`,
  `BodyExtension => !has_feet`). `Fish::body_extension()` accessor; `line_sprite()` grows top/below rows via
  `line_sprite_with_extension` (tentacle below-only; spike+wing top&bottom; `body_row` bumped by the top band so
  the body still anchors at `position.y`). `appendage_extent` folds the bands into `sprite_y_margins`. Width
  UNCHANGED everywhere (bands are vertical). `/show` + catch panel render it free via `render_fish_sprite`. +6
  tests (90 total). DOC: extension span = same painted-span inset 1 as feet (documented generalization vs the
  idealized art, per B2's precedent).
- 2026-06-12: **B4b bodyextension for slime + ball/skull** done (own commit). `BodyExtension/Decrease/
  ExtensionColor` added to `SLIME_CAPS` (shared by slime-line + ball/skull). Slime-line unfish (reversed/
  doppleganger/phantom/blinker) get the bands FOR FREE via `line_sprite()` + the `body_extension()` accessor
  (apply arm already wired in B4a). Ball/skull: `extension_row` gained a `max_tentacles: Option<usize>` knob
  (line passes `None`; ball/skull pass `Some(2)` = ≤2 per band, centered via `even_indices`); new
  `tank_view::render_extension_bands` stamps top (above row 0) + bottom (below the lowest line) bands for ALL
  three variants — spike/wing fill the edge line's painted span, tentacles cap at 2 and sway. Factored
  `resolve_line_cells` + `draw_appendage_row` (render_feet_row now reuses them). `appendage_extent` returns
  `(len,len)` for multi-row unfish so both bands stay in-tank. `Fish::facing_left()` accessor added. Width
  unchanged everywhere. +4 tests (2 ball render: spike both edges + tentacle ≤2/band; 2 state: slime grows
  rows / ball excludes feet).
- 2026-06-12: **B4c bodyextension for worm** done (own commit). `BodyExtension/Decrease/ExtensionColor`
  added to `WORM_CAPS` (apply arm already style-agnostic from B4a). `render_worm_portal` now stamps extension
  bands with the same `rem_euclid` portal wrap as the body: tentacle hangs BELOW only, spike/wing top&bottom
  (worm is line-like → reuses the now-`pub fn line_extension_bands`). New `draw_portal_row` helper (shared by
  feet + every band) wraps one cell-row both axes. Tentacles sway via `fish.sway.phase`. Width untouched
  (`worm_display_width` never sees extensions). +2 tests (worm render: tentacles below only, never on body;
  worm state: recorded + extensioncolor, width stable).
- 2026-06-12: **B4d bodyextension for cow** done (own commit). `BodyExtension/Decrease/ExtensionColor` added
  to `COW_CAPS` (apply arm already shared via `apply_mutant_mutation`); `Cow` overrides `has_bodyextension`
  so the follow-ups gate correctly. Cow gets TOP-of-torso bands only (all three variants), rendered in
  `tank_view::render_cow_extension`: builds a torso mask from the `_` cells of the torso-top sprite row
  (index `sprite_top_offset+1`, glisten/patch colors intact) and feeds it to the shared `extension_row`, then
  stamps each band one row higher (`base_y + torso_row_idx - 1 - depth`). Tentacles sway via `cow.sway.phase`,
  full `even_indices` spread (no per-band cap). DOC: tentacles rise ABOVE the torso top rather than piercing
  the `_` line (clean simplification, same spirit as line tentacles hanging below the body); cows are
  stationary so no bounce-margin work. Width never changes. +2 tests (cow state gating + spike-above-torso
  render).

- 2026-06-12: **SPEC CORRECTION (no code)** — user fixed a design mistake: telophase ability-stacking is
  NOT exclusive to engulfment-born telophases. EVERY telophase (normal/duplicate, backwardstelophase,
  engulfment, endocytosis) stacks its halves' abilities; fusing vs. duplicating is identical for this
  mechanic. A normally-telophased goldenfish = 2× zoomie money, like a goldenfish/goldenfish fusion would.
  Engulfment/endocytosis differ only in combining DIFFERENT species/identities. Recorded as a Section 0
  locked decision; Section 1 amended (verbatim prompt preserved); C4 (Section 6 + Section 8) rewritten:
  the `fused` companion must be seeded for plain telophase too, not just fusion.

- 2026-06-12: **C1 telophase udder fix** done (own commit). `double_cow_sprite` row4 belly is now all
  `-` (named `COW_BELLY`); the two `w` udders are gone from the plain telophase cow. Single cow keeps its
  one `w` udder (named `COW_UDDER`). C2 (backwardstelophase) will re-add the two `w` udders + milk
  duplication behind a `backwards` flag. +2 tests (116 total). build+test+clippy+fmt green.

- 2026-06-12: **C2a backwardstelophase for standard/mutant fish** done (own commit). New
  `Mutation::BackwardsTelophase` (token `backwardstelophase`, in `ALL` → standard fish via FULL caps;
  fixed/worm/cow caps deferred to C2b–d). `backwards: bool` on `MutantState` (alongside `is_double`).
  Render: both `segments_mutant` (live) and `static_left_segments_mutant` (static) standard branches gain
  a backward layout = `non_double_tail` on BOTH extremes + body fill + NO head eyes (matches spec
  `><))))><`; hydra still woven into the body centre → `><))ºº))><`). `MutantState::display_width` backward
  branch = `2·tail_w + body_size + max_eyes + EXTRA_BODY_FOR_DOUBLE + double_eyes (+ ear + hydra)`; render
  width matches (invariant test). Gating via new `Mutatable::backwards()`: `Telophase` avail unless already
  forward-double, `BackwardsTelophase` avail unless already backward-double → the two INTERCONVERT (flip
  either way); `Telophase` apply sets `backwards=false`. `split_standard_fish` clears `backwards` on the
  retained half. +5 tests (121 total). build+test+clippy+fmt green.

- 2026-06-12: **C2b backwardstelophase for fixed jelly + multichar fish** done (own commit).
  `BackwardsTelophase` added to `FIXED_MULTICHAR_CAPS` + `FIXED_JELLY_CAPS`. Both render paths' Fixed
  multichar branch gain a backward layout = `tail_ch` + body (hidden head-eyes replaced by body, same
  length as forward) + `mirror_body` + `invert_mouth(tail_ch)`; identical char count to the forward double
  so the existing fixed width formula (`2*(1+n_eyes+body_size)`, independent of `backwards`) applies
  unchanged — NO `recompute_display_width` edit. Jelly (single-char) needs nothing: any double renders
  `n=2`. `split_fixed_fish` clears `backwards`. Key fact: fixed multichar fish have 0 EyeState eyes by
  default (`ensure_fish_mutant` clears them). +3 tests (124 total). build+test+clippy+fmt green.

- 2026-06-12: **C2c backwardstelophase for the worm** done (own commit). `worm_backwards: bool` on
  `UnfishState` alongside `worm_is_double`. Backward worm = `,`-tail caps (head-width each, faceless) +
  body + `,` + tail cap — no `(0)` heads; hydra eyes still woven into the body. Same char count as the
  forward double → `worm_display_width` untouched. `worm_eye_count` returns 0 head-eyes when backward
  (hydra only); `worm_eye_cols` backward branch emits only hydra cols (heterochromia resync stays aligned).
  `BackwardsTelophase` added to `WORM_CAPS` + the unfish apply arm (resets/sets `worm_backwards`);
  `Fish::backwards()` now reads the worm; `split_worm` clears it. **Clean-up:** `build_worm` + `worm_eye_cols`
  shared 8 identical params → extracted a `WormShape` param object (fixes `too_many_arguments`, dedupes the
  call sites). +3 tests (127 total). build+test+clippy+fmt green.

- 2026-06-12: **C2d backwardstelophase for the cow** done (own commit) — **C2 now COMPLETE**.
  `BackwardsTelophase` in `COW_CAPS`; `Cow::backwards()` reads `mutant.backwards` (the cow shares the mutant
  apply arm, so the flag was already set). New `backward_cow_sprite`: 5 rows = blank top, torso-top `_`,
  `/\/(`…`)\/\` tails on both ends (no `(oo)` faces / `^__^` horns), `| w---w |` belly with TWO `w` udders,
  legs — matches the Section 1 art. `cow_display_width` backward branch = `2*COW_TAIL_W + torso`.
  **Milk duplication:** `Cow::milk_yield()` returns 2 for a backwards double; `tank_cow_counts` multiplies
  each cow by its yield (forward-telophase milk stacking deferred to C4 per the Section 0 correction).
  `split_cow` clears `backwards`. +5 tests (cow sprite/width/milk + cow routing). build+test+clippy+fmt green.

- 2026-06-13: **C4a fused companion + per-component cow milk** done (own commit). `src/fishes/fused.rs`
  (`FusedComponent {species: Option<FishSpecies>, name, weight_g}` — `None` species = cow). `fused: Vec`
  on `MutantState` + `UnfishState`; `seed_double` + worm seed it = 2 same-species components; all 4
  cytokinesis split sites clear it. Cow `milk_yield` now counts components (forward telophase = 2, like
  backward). `MutantBacked::self_component()`; `Fish::fused_components/ability_components/ability_stacks`.
  +3 tests.
- 2026-06-13: **C4b passive ability-stacking** done (own commit). Goldenfish zoomie cash, candyfish
  weight-touch, mutantfish auto-mutate frequency + rad weight gain all multiply by stack count via
  `ability_stacks`/`auto_mutate_stacks` (capability-driven). +2 tests. **C4 COMPLETE.**
- 2026-06-13: **C5 endocytosis** done (own commit). `Mutation::Endocytosis` collapses a double back to a
  single body while KEEPING `fused`, so abilities stay stacked. Seed guard changed to `fused.is_empty()`
  so the ledger is durable across flips/endocytosis and only cytokinesis clears it. +3 tests.
- 2026-06-13: **C3 engulfment** done (own commit). `Mutation::Engulfment` (standard fish + cow caps,
  `!is_double`) arms a 10s `engulf_timer`; `Tank::tick_engulfment` fuses a nearby same-type entity into a
  telophase carrying both identities (name "A / B", averaged weight); heterogeneous cytokinesis restores
  each original identity. Candyfish touch made ability-stack-driven. +4 tests. **C3 COMPLETE.**

- 2026-06-13: **C3 cross-species/variant fusion fix** (own commit). The first C3 cut wrongly required
  the SAME species (`same_fish_type`) — but Section 1 says "species the same if they are the same, if not
  also do Species 1 / Species 2" and the canonical example is a mutantfish+candyfish fusion. Fixed:
  `FusedComponent` now carries a `Lineage` enum (`Fish(species)` / `Unfish(kind)` / `Cow(variant)`);
  fusion gate relaxed to `fish_engulf_compatible` (any line-bodied fish — cross-species + slime unfish
  partners; ball/skull/worm excluded). Abilities stack across DIFFERENT natures: goldenfish cash now
  fires for any fish with a goldenfish component (not just goldenfish-host), auto-mutate selection
  (`apply_random_mutation`/rad) is `auto_mutate_stacks`-driven, cow milk is per-component-variant
  (`Cow::milk_components`, `tank_cow_counts`). Heterogeneous cytokinesis restores each component by
  `Lineage` (a phantom comes back via `new_unfish`, a yellow cow comes back yellow). +6 tests.
  DEFERRED: the live unfish KIND behaviour (phantom teleport, blinker invisibility, doppleganger clone)
  does not actively run while the unfish is fused into a non-unfish host — the identity round-trips and
  species/variant passives stack, but those sim-loop behaviours are tied to `unfish_state` internals.

- 2026-06-14: **fused unfish personas (live behaviour through fusion)** done. The deferred C3 gap is closed:
  `FusedComponent` now carries an optional `persona: Box<UnfishState>` (seeded ONLY for engulfed slime
  unfish whose `UnfishKind::has_fused_behavior()` — phantom/blinker/doppleganger; reversed & same-species
  seeds stay `None`). `Fish::personas_mut()` yields the unfish behaviours a fish embodies (own `unfish_state`,
  else its mutant passengers); `Fish::is_invisible()` reads it (host vanishes while a blinker passenger is
  invisible); `Fish::tick` ticks passenger `UnfishState::tick` so the blinker phase machine runs; `tick_phantoms`
  drives every persona's `phantom_timer` and teleports the HOST; `tick_dopplegangers` splits into standalone +
  fused — a fused un-cloned doppleganger impersonates a victim by renaming its component (`Vic?`) so it emerges
  impersonating on re-split. Personas are behaviour-only: split still respawns the unfish fresh (consistent with
  the existing render simplification). +3 tests (phantom/blinker/doppleganger through fusion).
- 2026-06-14: **worm engulfment** done (the Section 8 follow-up). `Engulfment` added to `WORM_CAPS`;
  `apply_unfish_mutation` arms `engulf_timer`. Compatibility is now class-matched via `engulf_compatible(receiver,
  candidate)` — a worm receiver fuses only with another worm, a non-worm receiver only with non-worm line/slime
  partners (ball/skull never). `fuse_fish` writes the ledger through `Fish::set_fused` (works for both the
  mutant-backed and worm-`unfish_state` receivers). New `try_split_engulfment_worm` restores both worm identities
  on cytokinesis, mirroring `try_split_engulfment_fish`. +2 tests (worm fuse+resplit; worm refuses a standard fish).

- 2026-06-14: **appendage polish + fusion equilibrium** (post-epic bug-fix pass, from live play feedback):
  - **Feet/extensions now sit ONLY over true body cells.** Replaced the `painted_span` inset-by-1 heuristic
    (which leaked feet onto tails and extensions onto the eye column, and jittered with the koi swaying tail).
    `Fish::line_cells()` now returns the rendered cells **plus a structural body-column span**, computed at
    construction: `segments_mutant` (anchored head-side: `1 + eye_count + ear_count` head / tail width on the
    far side; backwards-double = tail-width head; double-forward excludes the trailing head-eyes) via
    `mutant_body_span`, and `insert_line_appendages` now RETURNS the slime body span. `feet_row`/`extension_row`
    take the span explicitly; multi-row/worm/cow callers pass `painted_span` (now `pub`) of their body row.
  - **Tentacles render like seaweed top→down.** Each tentacle is its own plant: independent per-strand phase
    (`TENTACLE_PHASE_STEP`), `|`→`(`/`)` lean glyphs by sway sign (reuses plant amount 2.0 / spread 0.5), root
    (depth 0) anchored. Gap widened (stride 2→3).
  - **Wings stack diagonally** (`+depth` cols facing-left, `-depth` facing-right).
  - **Ragged rows:** every strand's first row always shows; each deeper row needs its (vertical/diagonal)
    predecessor AND a 50/50 — deterministic per `BodyExtension.seed` (new field) via `strand_length`.
  - **Removed `extensioncolor`** (Mutation + the `BodyExtension.color` field): extensions always copy body color.
  - **Fusion equilibrium:** `Engulfment` is now `auto_selectable` (was excluded) so auto-mutation actually fuses
    — previously only `Cytokinesis` (split, +1 fish) ran automatically while engulfment (−1) never did, so the
    population only ever grew. Engulf reach 1→5 cells, fish y-tolerance 1→2.
  All build+test (133)+clippy+fmt green.

- 2026-06-14: **engulfment per-half visual merge** (closes the documented "renders as the receiver's telophase"
  simplification, per user direction "full per-half merge / heavier wins"). `FusedComponent` now carries an
  `Option<Box<Fish>>` **snapshot** of each fused half (seeded in `fuse_fish` from a clone of each fish BEFORE
  telophase). A fish renders per-half when `fused_render_halves()` (is_double + 2 components, both with snapshots)
  is `Some`: `Fish::fused_line_sprite` reconstructs each half (left facing-Left, right facing-Right, host's sway
  phase), takes each one's head+body via the structural body span (drops the tail), and concatenates the two
  `LineSprite`s row-aligned by `body_row` — so each half keeps its OWN color/glisten/eyes/body-extension/feet for
  free (reuses `line_sprite`). `recompute_display_width`/`fused_render_width` set the fused width to the summed
  half widths (all kept cells are width-1 since tails are dropped). **Cytokinesis** restores each half from its
  snapshot (`restore_from_snapshot`, exact visual state), honoring the component name (so a fused doppleganger's
  `Vic?` impersonation still emerges). **Mutations while merged hit both halves** (`propagate_mutation_to_halves`
  re-applies the same non-structural mutation to each snapshot, gated by each snapshot's `supports_now`).
  **Endocytosis** (`collapse_endocytosis`) collapses to the HEAVIER component's body, keeps the 2-component ledger
  (snapshots dropped) for ability stacking, and `ensure_fish_mutant`s a plain heavier-fish so the ledger has a home.
  Deferred (documented): worm/fixed/cow fused bodies still render as the uniform telophase (only standard-line
  receivers get the per-half body); per-half appendage bounce-margins aren't reserved (cosmetic clip at tank edges).
  +4 tests. build+test(137 lib)+clippy+fmt green.

- 2026-06-14: **engulfment per-half for worm + cow; cow/worm split now carries modifications** (closes the
  documented "worm/cow render as the receiver's telophase" + "split respawns fresh" gaps, per user direction).
  `FusedComponent.snapshot` is now `Option<Snapshot>` where `enum Snapshot { Fish(Box<Fish>), Cow(Box<Cow>) }`
  (`fish_snapshot`/`cow_snapshot` (+`_mut`) accessors); `fuse_cow` seeds cow snapshots like `fuse_fish` does.
  **Worm per-half:** `Fish::fused_worm_cells()` concatenates the left half + the mirrored (`sprite::mirror_char`)
  right half (each rebuilt facing-Left, tail comma popped) → two heads outside; `render_worm_portal` +
  `fused_render_width` read it. **Cow per-half:** `cow_sprite`'s double path → `per_half_double_cow` builds the
  two-headed `double_cow_sprite` twice (skin-driven, one `&Cow` per side via `skinned_double_cow`) and splices
  the two same-width grids at the torso seam — width unchanged. **Carryover:** the engulfment-split path is
  unified into one backing-agnostic `try_split_engulfment_fish` (fish + worm, snapshot-only; the old fresh-respawn
  fallback + `try_split_engulfment_worm` deleted); `try_split_engulfment_cow` restores via `restore_cow_from_snapshot`.
  Mutations while merged now propagate to worm snapshots too (`propagate_mutation_to_halves` reads
  `fused_components_mut`, no longer mutant-only) and to cow snapshots (new `propagate_cow_mutation_to_halves`).
  So a fused half that turned green / grew tentacles / got red eyes splits back out exactly that way. +4 tests
  (worm + cow per-half color render; worm + cow mutation carried through fuse→cytokinesis). build+test(141)+clippy+fmt green.

**Done so far:** A1, A2, A3 (Tier 1); worm-heterochromia fix; B1 ear; B3 hydra; B0 scaffold; B2 feet;
B4 bodyextension; **C1 udder fix; C2 backwardstelophase; C4 ability-stacking; C5 endocytosis; C3
engulfment (incl. cross-species/variant fusion); fused unfish personas; worm engulfment.**
**Status: THE WHOLE MUTATIONS EPIC IS COMPLETE.** Every mutation in the original prompt (Section 1) has
landed across every entity it cleanly applies to. Documented simplifications: engulfment is mechanically
faithful but renders as the receiver's telophase (not a per-half visual blend); fixed-fish engulfment is
still deferred (fixed fish remain telophase/endocytosis-capable). Worm engulfment and the live unfish
personas (phantom/blinker/doppleganger behaving through fusion) are now wired.

---

## 8. NEXT-SESSION PROMPT (paste-ready — KEEP CURRENT)

> **THE MUTATIONS EPIC IS COMPLETE — there is no next mutation to implement.** Phase A (Tier 1), Phase B
> (Tier 2 appendages), and Phase C (Tier 3 fusion: C1 udder fix, C2 backwardstelophase, C3 engulfment, C4
> ability-stacking, C5 endocytosis) have ALL landed, each its own commit, build+test+clippy+fmt green. Every
> mutation in the original prompt (Section 1) is implemented across every entity it cleanly applies to.
>
> If you are asked to extend the mutation system further, the architecture (all GREEN) is: `enum Mutation`
> (token/parse/ALL) in `src/fishes/mutations.rs` is the ONLY name list; cap consts (`MUTANT_FULL_CAPS=ALL`,
> `FIXED_*`, `SLIME_CAPS`, `WORM_CAPS`, `COW_CAPS`) + `Mutatable::supports_now` are the ONLY gates;
> `apply_mutant_mutation`/`apply_unfish_mutation` are the only effect sites; `apply_mutation` is the only
> recorder; `Tank::apply_named_mutation` the only entry. Fusion lives in the `fused` companion
> (`src/fishes/fused.rs`, `Vec<FusedComponent>` on `MutantState`/`UnfishState`), seeded by `seed_double`,
> read by `Fish::ability_stacks`/`auto_mutate_stacks` + `Cow::milk_yield`, cleared only by cytokinesis,
> populated with both identities by `Tank::tick_engulfment` and restored by `try_split_engulfment_fish/_cow/_worm`.
> Live unfish behaviour through fusion rides in `FusedComponent::persona` (an optional `Box<UnfishState>`
> seeded for engulfed phantom/blinker/doppleganger), driven by `Fish::personas_mut`/`is_invisible`/`tick_passengers`.
>
> **Known follow-ups (OPTIONAL polish, not part of the epic):**
> - Engulfment is wired for standard fish + cows + worms, and ALL THREE now render per-half (each half its own
>   color/eyes/etc) AND carry their merged-time modifications back out on cytokinesis (snapshot restore).
>   Only fixed-fish engulfment is still deferred (they remain telophase/endocytosis-capable, and still render as
>   the uniform telophase when doubled). Extending it = add `Engulfment` to `FIXED_*_CAPS`, seed a fish snapshot in
>   the fuse path, and add a fixed per-half render branch (the split already works via `try_split_engulfment_fish`).
> - Per-half APPENDAGES (feet/tentacles) on the fused worm + cow are still host-driven, not per-half — only the
>   body/eyes/ears/hydra/color split per side. Purely cosmetic; the split-out entities restore their exact
>   appendages from the snapshot regardless.
