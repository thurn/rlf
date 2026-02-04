# Appendix: RLF Standard Library

This appendix documents the standard transforms and metadata tags provided by RLF for the world's most widely-spoken languages.

## Overview

RLF provides three categories of transforms:

1. **Universal transforms**: Work on any text in any language
2. **Language-family transforms**: Shared across related languages
3. **Language-specific transforms**: Unique to individual languages

---

## Universal Transforms

These transforms work identically in all languages:

| Transform | Effect | Example |
|-----------|--------|---------|
| `@cap` | Capitalize first letter | "card" → "Card" |
| `@upper` | All uppercase | "card" → "CARD" |
| `@lower` | All lowercase | "Card" → "card" |

---

## Language Reference

### English

**Grammatical features**: No gender, no case, simple plural (one/other)

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:a` | Use "a" as indefinite article (required for `@a`) |
| `:an` | Use "an" as indefinite article (required for `@a`) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@a` | `@an` | `:a`, `:an` (required) | Prepend indefinite article; errors if tag missing |
| `@the` | - | - | Prepend "the" |

**Plural categories**: `one`, `other`

```rust
// strings.rlf.rs
rlf! {
    card = :a { one: "card", other: "cards" };
    event = :an { one: "event", other: "events" };
    ally = :an { one: "ally", other: "allies" };
    hour = :an "hour";       // silent h
    uniform = :a "uniform";  // /juː/ sound

    draw_one = "Draw {@a card}.";      // → "Draw a card."
    play_one = "Play {@a event}.";     // → "Play an event."
    the_card = "{@the card}";          // → "the card"
}
```

---

### Mandarin Chinese (简体中文)

**Grammatical features**: No plural, no gender, no case, measure words required

**Metadata tags**:
| Tag | Purpose | Measure word |
|-----|---------|--------------|
| `:zhang` | Flat objects (cards, paper) | 张 |
| `:ge` | General classifier | 个 |
| `:ming` | People (formal) | 名 |
| `:wei` | People (respectful) | 位 |
| `:tiao` | Long thin objects | 条 |
| `:ben` | Books, volumes | 本 |
| `:zhi` | Animals, hands | 只 |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@count` | - | measure word tags | Insert number + measure word |

**Plural categories**: `other` (no plural distinction)

```rust
// zh_cn.rlf
pai = :zhang "牌";
jue_se = :ge "角色";
wan_jia = :ming "玩家";

// @count:n uses n as context (the count), reads measure word tag from phrase
draw(n) = "抽{@count:n pai}";       // n=3 → "抽3张牌"
summon(n) = "召唤{@count:n jue_se}"; // n=2 → "召唤2个角色"
```

---

### Hindi (हिन्दी)

**Grammatical features**: Two genders (masc/fem), two numbers, three cases (direct/oblique/vocative)

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |

**Plural categories**: `one`, `other`

```rust
// hi.rlf
card = :masc {
    dir: "कार्ड",
    obl: "कार्ड",
    obl.other: "कार्डों",
};

event = :fem {
    dir: "घटना",
    obl: "घटना",
    obl.other: "घटनाओं",
};
```

---

### Spanish (Español)

**Grammatical features**: Two genders (masc/fem), definite and indefinite articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |

**Transforms**:
| Transform | Aliases | Reads | Context | Effect |
|-----------|---------|-------|---------|--------|
| `@el` | `@la` | `:masc`, `:fem` | `:one`/`:other` | Definite article (el/la/los/las) |
| `@un` | `@una` | `:masc`, `:fem` | `:one`/`:other` | Indefinite article (un/una/unos/unas) |

Use context selector for plural forms: `@el:other` → los/las, `@un:other` → unos/unas.

**Plural categories**: `one`, `other`

```rust
// es.rlf
card = :fem { one: "carta", other: "cartas" };
enemy = :masc { one: "enemigo", other: "enemigos" };

destroyed = { masc: "destruido", fem: "destruida" };

draw_one = "Roba {@un card}.";                // → "Roba una carta."
the_enemy = "{@el enemy}";                    // → "el enemigo"
return_all(t) = "devuelve {@el:other t}";     // → "devuelve las cartas"
destroy(x) = "{x} fue {destroyed:x}.";        // → "carta fue destruida."
```

---

### French (Français)

**Grammatical features**: Two genders, articles, contractions, elision

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |
| `:vowel` | Starts with vowel sound (for elision) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@le` | `@la` | `:masc`, `:fem`, `:vowel` | Definite article (le/la/l'/les) |
| `@un` | `@une` | `:masc`, `:fem` | Indefinite article (un/une) |
| `@de` | - | `:masc`, `:fem`, `:vowel` | "de" + article (du/de la/de l'/des) |
| `@au` | - | `:masc`, `:fem`, `:vowel` | "à" + article (au/à la/à l'/aux) |

**Plural categories**: `one`, `other`

```rust
// fr.rlf
card = :fem "carte";
enemy = :masc :vowel "ennemi";
friend = :masc :vowel "ami";
void = :masc "vide";
hand = :fem "main";

the_card = "{@le card}";      // → "la carte"
the_enemy = "{@le enemy}";    // → "l'ennemi" (elision)
from_void = "{@de void}";     // → "du vide"
to_hand = "{@au hand}";       // → "à la main"
```

---

### Arabic (العربية)

**Grammatical features**: Two genders, three numbers (singular/dual/plural), definite article, complex agreement

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |
| `:sun` | Sun letter (assimilates ال) |
| `:moon` | Moon letter (no assimilation) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@al` | - | `:sun`, `:moon` | Definite article with assimilation |

**Plural categories**: `zero`, `one`, `two`, `few`, `many`, `other`

```rust
// ar.rlf
card = :fem :moon {
    one: "بطاقة",
    two: "بطاقتان",
    few: "بطاقات",
    many: "بطاقة",
    other: "بطاقات",
};
```

---

### Bengali (বাংলা)

**Grammatical features**: No gender, classifiers for counting

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:ta` | General classifier (টা) |
| `:ti` | Formal classifier (টি) |
| `:khana` | For flat objects (খানা) |
| `:jon` | For people (জন) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@count` | - | classifier tags | Number + classifier |

**Plural categories**: `one`, `other`

---

### Portuguese (Português)

**Grammatical features**: Two genders, articles, contractions

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@o` | `@a` | `:masc`, `:fem` | Definite article (o/a/os/as) |
| `@um` | `@uma` | `:masc`, `:fem` | Indefinite article (um/uma) |
| `@de` | - | `:masc`, `:fem` | "de" + article (do/da/dos/das) |
| `@em` | - | `:masc`, `:fem` | "em" + article (no/na/nos/nas) |

**Plural categories**: `one`, `other`

```rust
// pt_br.rlf
card = :fem "carta";
enemy = :masc "inimigo";
void = :masc "vazio";
hand = :fem "mão";

the_card = "{@o card}";      // → "a carta"
from_void = "{@de void}";    // → "do vazio"
in_hand = "{@em hand}";      // → "na mão"
```

---

### Russian (Русский)

**Grammatical features**: Three genders, six cases, complex plural

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |
| `:neut` | Neuter gender |
| `:anim` | Animate (affects accusative) |
| `:inan` | Inanimate |

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `prep`

```rust
// ru.rlf
card = :fem :inan {
    nom: "карта",
    nom.many: "карт",
    acc: "карту",
    acc.many: "карт",
    gen: "карты",
    gen.many: "карт",
    ins.one: "картой",
    ins: "картами",
};

ally = :masc :anim {
    nom.one: "союзник",
    nom: "союзники",
    nom.many: "союзников",
    acc, gen: "союзника",
    acc.many, gen.many: "союзников",
    ins.one: "союзником",
    ins: "союзниками",
};
```

---

### Japanese (日本語)

**Grammatical features**: No plural, no gender, counters (similar to Chinese), particles

**Metadata tags**:
| Tag | Purpose | Counter |
|-----|---------|---------|
| `:mai` | Flat objects | 枚 |
| `:nin` | People | 人 |
| `:hiki` | Small animals | 匹 |
| `:hon` | Long objects | 本 |
| `:ko` | General small objects | 個 |
| `:satsu` | Books | 冊 |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@count` | - | counter tags | Number + counter |

**Plural categories**: `other` (no plural distinction)

```rust
// ja.rlf
card = :mai "カード";
character = :nin "キャラクター";

draw(n) = "{@count:n card}を引く";  // n=3 → "3枚カードを引く"
```

---

### German (Deutsch)

**Grammatical features**: Three genders, four cases, definite/indefinite articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine (der) |
| `:fem` | Feminine (die) |
| `:neut` | Neuter (das) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@der` | `@die`, `@das` | `:masc`, `:fem`, `:neut` + case | Definite article (der/die/das/den/dem/des) |
| `@ein` | `@eine` | `:masc`, `:fem`, `:neut` + case | Indefinite article (ein/eine/einen/einem/einer/eines) |

**Plural categories**: `one`, `other`

**Case variants**: `nom`, `acc`, `dat`, `gen`

```rust
// de.rlf
karte = :fem {
    nom, acc, dat, gen: "Karte",
    nom.other, acc.other, dat.other, gen.other: "Karten",
};

charakter = :masc {
    nom, acc, dat, gen: "Charakter",
    nom.other, acc.other, dat.other, gen.other: "Charaktere",
};

ereignis = :neut {
    nom, acc, dat, gen: "Ereignis",
    nom.other, acc.other, dat.other, gen.other: "Ereignisse",
};

the_card = "{@der:nom karte}";   // → "die Karte"
a_char = "{@ein:acc charakter}"; // → "einen Charakter"
```

---

### Korean (한국어)

**Grammatical features**: No gender, counters, particles, honorifics

**Metadata tags**:
| Tag | Purpose | Counter |
|-----|---------|---------|
| `:jang` | Flat objects | 장 |
| `:myeong` | People (formal) | 명 |
| `:mari` | Animals | 마리 |
| `:gae` | General objects | 개 |
| `:gwon` | Books | 권 |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@count` | - | counter tags | Number + counter (Korean or Sino-Korean) |
| `@particle` | - | final sound | Context-sensitive particle (가/이, 를/을, etc.) |

The `@particle` transform inspects the final sound of the preceding word. See
**Advanced Transforms** section for details.

**Plural categories**: `other` (no plural distinction)

```rust
// ko.rlf
card = :jang "카드";
character = :myeong "캐릭터";

// @count produces "카드 3장", @particle adds correct object particle based on final sound
draw(n) = "{@count:n card}{@particle:obj card} 뽑는다";  // n=3 → "카드 3장을 뽑는다"
thing_exists(thing) = "{thing}{@particle:subj thing} 있다";
```

---

### Vietnamese (Tiếng Việt)

**Grammatical features**: No inflection, classifiers

**Metadata tags**:
| Tag | Purpose | Classifier |
|-----|---------|------------|
| `:cai` | General objects | cái |
| `:con` | Animals, some objects | con |
| `:nguoi` | People | người |
| `:chiec` | Vehicles, single items | chiếc |
| `:to` | Flat paper items | tờ |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@count` | - | classifier tags | Number + classifier |

**Plural categories**: `other` (no plural distinction)

---

### Turkish (Türkçe)

**Grammatical features**: Vowel harmony, agglutinative, no gender, six cases

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:front` | Front vowels (e, i, ö, ü) |
| `:back` | Back vowels (a, ı, o, u) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@inflect` | - | `:front`, `:back` | Suffix chain with vowel harmony |

The `@inflect` transform handles agglutinative suffix chains. See **Advanced
Transforms** section for details.

**Plural categories**: `one`, `other`

```rust
// tr.rlf
ev = :back "ev";
göz = :front "göz";

to_house = "{@inflect:dat ev}";              // → "eve"
from_my_houses = "{@inflect:abl.poss1sg.pl ev}"; // → "evlerimden"
```

---

### Italian (Italiano)

**Grammatical features**: Two genders, articles, contractions, elision

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine |
| `:fem` | Feminine |
| `:vowel` | Starts with vowel |
| `:s_imp` | Starts with s+consonant, z, gn, ps, x |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@il` | `@lo`, `@la` | gender + sound tags | Definite article (il/lo/la/l'/i/gli/le) |
| `@un` | `@uno`, `@una` | gender + sound tags | Indefinite article (un/uno/una/un') |
| `@di` | - | gender + sound tags | "di" + article (del/dello/della/dell'/dei/degli/delle) |
| `@a` | - | gender + sound tags | "a" + article (al/allo/alla/all'/ai/agli/alle) |

**Plural categories**: `one`, `other`

```rust
// it.rlf
card = :fem "carta";
student = :masc :s_imp "studente";
friend = :masc :vowel "amico";

the_card = "{@il card}";       // → "la carta"
the_student = "{@il student}"; // → "lo studente"
the_friend = "{@il friend}";   // → "l'amico"
```

---

### Polish (Polski)

**Grammatical features**: Three genders, seven cases, complex plural, animate distinction

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc_anim` | Masculine animate |
| `:masc_inan` | Masculine inanimate |
| `:fem` | Feminine |
| `:neut` | Neuter |

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc`

```rust
// pl.rlf
card = :fem {
    nom: "karta",
    nom.many: "kart",
    acc: "kartę",
    acc.many: "kart",
    gen: "karty",
    gen.many: "kart",
};

enemy = :masc_anim {
    nom.one: "wróg",
    nom: "wrogowie",
    nom.many: "wrogów",
    acc, gen: "wroga",
    acc.many, gen.many: "wrogów",
};
```

---

### Ukrainian (Українська)

**Grammatical features**: Three genders, seven cases, complex plural (same as Russian/Polish family)

**Metadata tags**: Same as Russian (`:masc`, `:fem`, `:neut`, `:anim`, `:inan`)

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc`

---

### Dutch (Nederlands)

**Grammatical features**: Two genders (common/neuter for articles), definite/indefinite articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:de` | Common gender (de-words) |
| `:het` | Neuter gender (het-words) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@de` | `@het` | `:de`, `:het` | Definite article (de/het) |
| `@een` | - | - | Indefinite article (een) |

**Plural categories**: `one`, `other`

```rust
// nl.rlf
card = :de "kaart";
character = :het "karakter";

the_card = "{@de card}";        // → "de kaart"
the_char = "{@de character}";   // → "het karakter"
```

---

### Thai (ภาษาไทย)

**Grammatical features**: No inflection, classifiers

**Metadata tags**:
| Tag | Purpose | Classifier |
|-----|---------|------------|
| `:bai` | Flat objects, cards | ใบ |
| `:tua` | Animals, letters, characters | ตัว |
| `:khon` | People | คน |
| `:an` | General small objects | อัน |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@count` | - | classifier tags | Number + classifier |

**Plural categories**: `other` (no plural distinction)

---

### Indonesian (Bahasa Indonesia)

**Grammatical features**: No inflection, no gender, reduplication for plural

**Metadata tags**: None required

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@plural` | - | - | Reduplication (kartu → kartu-kartu) |

**Plural categories**: `other` (context-dependent)

```rust
// id.rlf
card = "kartu";

all_cards = "semua {@plural card}";  // → "semua kartu-kartu"
```

---

### Persian (فارسی)

**Grammatical features**: No gender, ezafe construction, simple plural

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:vowel` | Ends in vowel (affects ezafe) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@ezafe` | - | `:vowel` | Ezafe connector (-e/-ye) |

**Plural categories**: `one`, `other`

```rust
// fa.rlf
card = "کارت";
hand = :vowel "دست";

card_of_player = "{@ezafe card} بازیکن";  // → "کارت‌ِ بازیکن"
```

---

### Romanian (Română)

**Grammatical features**: Three genders, postposed definite article, two cases

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine |
| `:fem` | Feminine |
| `:neut` | Neuter (masc singular, fem plural) |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@def` | - | gender | Postposed definite article |

**Plural categories**: `one`, `few`, `other`

```rust
// ro.rlf
card = :fem "carte";

the_card = "{@def card}";  // → "cartea"
```

---

### Greek (Ελληνικά)

**Grammatical features**: Three genders, four cases, articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine |
| `:fem` | Feminine |
| `:neut` | Neuter |

**Transforms**:
| Transform | Aliases | Reads | Effect |
|-----------|---------|-------|--------|
| `@o` | `@i`, `@to` | gender + case | Definite article (ο/η/το/τον/την/του/της/οι/τα...) |
| `@enas` | `@mia`, `@ena` | gender + case | Indefinite article |

**Plural categories**: `one`, `other`

---

### Czech (Čeština)

**Grammatical features**: Three genders, seven cases, animate distinction

**Metadata tags**: Same as Polish

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc`

---

## Summary Table

| Language | Gender | Cases | Plural Forms | Key Transforms |
|----------|--------|-------|--------------|----------------|
| English | - | - | 2 | `@a`, `@the` |
| Chinese | - | - | 1 | `@count` |
| Hindi | 2 | 3 | 2 | - |
| Spanish | 2 | - | 2 | `@el`, `@un` |
| French | 2 | - | 2 | `@le`, `@un`, `@de`, `@a` |
| Arabic | 2 | 3 | 6 | `@al` |
| Bengali | - | - | 2 | `@count` |
| Portuguese | 2 | - | 2 | `@o`, `@um`, `@de`, `@em` |
| Russian | 3 | 6 | 4 | - |
| Japanese | - | - | 1 | `@count` |
| German | 3 | 4 | 2 | `@der`, `@ein` |
| Korean | - | - | 1 | `@count`, `@particle` |
| Vietnamese | - | - | 1 | `@count` |
| Turkish | - | 6 | 2 | `@inflect` |
| Italian | 2 | - | 2 | `@il`, `@un`, `@di`, `@a` |
| Polish | 3 | 7 | 4 | - |
| Ukrainian | 3 | 7 | 4 | - |
| Dutch | 2 | - | 2 | `@de`, `@een` |
| Thai | - | - | 1 | `@count` |
| Indonesian | - | - | 1 | `@plural` |
| Persian | - | - | 2 | `@ezafe` |
| Romanian | 3 | 2 | 3 | `@def` |
| Greek | 3 | 4 | 2 | `@o`, `@enas` |
| Czech | 3 | 7 | 4 | - |

---

## Advanced Transforms

These transforms handle morphological operations that selection alone cannot express—
they inspect phonology, apply harmony rules, or copy agreement features dynamically.

### `@particle` — Phonology-Based Particle Selection

**Languages:** Korean, Japanese

Korean particles change form based on the preceding sound. The transform inspects
the final Unicode grapheme cluster (not byte or code point) to determine vowel
vs. consonant ending. This handles composed Hangul syllables correctly.

```rust
// ko.rlf
apple = "사과";   // ends in vowel (과 = gwa)
book = "책";      // ends in consonant (책 = chaek)

thing_is(thing) = "{thing}{@particle:subj thing} 있다";

// apple → "사과가 있다" (vowel-final: 가)
// book → "책이 있다" (consonant-final: 이)
```

| Context | Vowel-final | Consonant-final |
|---------|-------------|-----------------|
| `:subj` | 가 (ga) | 이 (i) |
| `:obj` | 를 (reul) | 을 (eul) |
| `:topic` | 는 (neun) | 은 (eun) |

The transform cannot use tags because any phrase might end in any sound depending
on its variant form. It must inspect rendered text at runtime.

---

### `@inflect` — Agglutinative Suffix Chains

**Languages:** Turkish, Finnish, Hungarian, other Uralic/Turkic languages

Agglutinative languages build words by chaining suffixes, where each suffix's
form depends on vowel harmony with what precedes it. A single transform handles
the full chain.

```rust
// tr.rlf
ev = :back "ev";           // house (back vowel)
göz = :front "göz";        // eye (front vowel)

from_my_houses = "{@inflect:abl.poss1sg.pl ev}";   // → "evlerimden"
from_my_eyes = "{@inflect:abl.poss1sg.pl göz}";   // → "gözlerimden"
```

**Suffix chain for "evlerimden" (from my houses):**

| Suffix | Function | Back form | Front form |
|--------|----------|-----------|------------|
| plural | -ler/-lar | -lar | -ler |
| poss1sg | -im/-ım/-um/-üm | -ım | -im |
| ablative | -den/-dan/-ten/-tan | -dan | -den |

Result: ev + ler + im + den → "evlerimden"

The transform applies vowel harmony at each step. The `:back`/`:front` tag on
the root determines the initial harmony class, and subsequent suffixes follow
the "last vowel" rule.

**Available suffixes:**

| Context | Meaning |
|---------|---------|
| `pl` | Plural |
| `poss1sg`, `poss2sg`, `poss3sg` | Possessive (my/your/their) |
| `poss1pl`, `poss2pl`, `poss3pl` | Possessive plural |
| `nom`, `acc`, `dat`, `gen`, `loc`, `abl` | Cases |

Suffixes are applied left-to-right as specified in the context.

---

### `@liaison` — Prevocalic Form Selection

**Languages:** French

French has a small set of adjectives with special prevocalic forms. The `@liaison`
transform selects between standard and prevocalic forms based on the following
word's `:vowel` tag.

```rust
// fr.rlf
ami = :masc :vowel "ami";
livre = :masc "livre";

// Use variants for liaison forms
ce = { standard: "ce", vowel: "cet" };
beau = { standard: "beau", vowel: "bel" };

this_thing(thing) = "{@liaison ce thing} {thing}";

// ami (has :vowel) → "cet ami"
// livre (no :vowel) → "ce livre"
```

The transform reads the `:vowel` tag from its second argument and selects the
matching variant from the first argument. This handles the four common cases
(ce/cet, beau/bel, nouveau/nouvel, vieux/vieil) without phonological analysis.

---

## Design Notes

### Transform Names Are Language-Scoped

The same transform name can have different meanings in different languages. For
example:

- `@a` in **English**: Indefinite article ("a card" / "an event")
- `@a` in **Portuguese**: Alias for `@o`, the definite article ("a carta" = "the card")
- `@a` in **Italian**: Preposition+article contraction ("a" + article → "al/alla/...")

This is intentional—transforms are registered per-language, so there is no
conflict at runtime. However, when reading translation files, be aware that
familiar transform names may behave differently than in other languages.

### Required Metadata Tags

Metadata-driven transforms require their expected tags to be present. Using `@a`
on a phrase without `:a` or `:an` produces a runtime error, not a guess based on
phonetics. This prevents silent incorrect output (e.g., "a uniform" is correct
but heuristics would suggest "an uniform"; "an hour" is correct but heuristics
would suggest "a hour").

Similarly, `@the` in German requires `:masc`/`:fem`/`:neut`, `@count` in Chinese
requires a measure word tag like `:zhang`/`:ge`, etc. Always define phrases with
the tags required by the transforms that will be applied to them.

### Languages Without Special Transforms

Some languages (Russian, Polish, Ukrainian, Czech) have complex case systems but don't need special transforms—variant selection handles all the complexity. The Rust code selects the appropriate case+number variant.

### Classifier/Counter Languages

Chinese, Japanese, Korean, Vietnamese, Thai, and Bengali all use classifiers. The `@count` transform is shared across these languages but reads language-specific tags.

### Contraction Languages

French, Italian, and Portuguese all have preposition+article contractions. Each has its own transforms (`@de`, `@a`, `@em`, etc.) that handle the contraction rules.

### Vowel Harmony Languages

Turkish and other Turkic languages require vowel harmony in suffixes. Tags like `:front`/`:back` let transforms select the correct suffix form.
