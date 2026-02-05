# Phase 8: Greek, Romanian, and Middle Eastern Transforms - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Article transforms for Greek and Romanian, plus special transforms for Arabic (sun/moon letter assimilation) and Persian (ezafe connector). This phase covers four distinct language families: Greek (Hellenic), Romanian (Romance with postposed articles), Arabic (Semitic), and Persian (Indo-Iranian).

</domain>

<decisions>
## Implementation Decisions

### Greek Articles (@o, @enas)
- Three genders: `:masc`, `:fem`, `:neut` tags
- Four cases via context: nom, acc, gen, dat
- Definite article @o with @i (fem) and @to (neut) aliases
- Indefinite article @enas with @mia (fem) and @ena (neut) aliases
- Full declension tables required (gender x case x number combinations)
- Plural categories: `one`, `other`

### Romanian Postposed Articles (@def)
- Three genders: `:masc`, `:fem`, `:neut` tags
- Neuter behaves as masculine singular, feminine plural
- @def transform appends article suffix to word (not prepends)
- Two cases to consider (nominative/accusative vs genitive/dative)
- Plural categories: `one`, `few`, `other`

### Arabic Sun/Moon Letters (@al)
- Tags: `:sun` for sun letters, `:moon` for moon letters
- Sun letters: assimilation occurs (ال + شمس → الشَّمس, pronounced ash-shams)
- Moon letters: no assimilation (ال + قمر → القَمَر, pronounced al-qamar)
- Transform prepends definite article with appropriate assimilation marker
- Plural categories: `zero`, `one`, `two`, `few`, `many`, `other`

### Persian Ezafe (@ezafe)
- Single tag: `:vowel` (ends in vowel)
- Words ending in vowel: use -ye connector
- Words ending in consonant: use -e connector (kasra: ِ)
- No gender system in Persian
- Plural categories: `one`, `other`

### Claude's Discretion
- Greek article declension table organization (single struct vs nested)
- Romanian article suffix implementation details
- Arabic Unicode handling for shadda (ّ) marker with sun letters
- Persian zero-width non-joiner (ZWNJ) usage in ezafe

</decisions>

<specifics>
## Specific Ideas

From APPENDIX_STDLIB.md specification:
- Greek: `@o` produces ο/η/το/τον/την/του/της/οι/τα etc.
- Romanian: `@def card` produces "cartea" (carte + a suffix)
- Arabic: `@al` handles assimilation — documented sun vs moon letter distinction
- Persian: `@ezafe card` produces "کارت‌ِ" with kasra connector

Follow established transform patterns:
- Use existing RomanceGender for Romanian (like Spanish/Portuguese)
- Create new GreekGender enum for three-way distinction
- Static dispatch via TransformKind enum (no trait objects)
- Tags read from Value, context from EvalContext

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-greek-romanian-and-middle-eastern-transforms*
*Context gathered: 2026-02-04*
