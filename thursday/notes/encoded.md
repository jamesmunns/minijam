# Encoded notes

The idea is to use variable-length encoding for each of three fields:

* Pitch
* Starting Beat
* Note Length

Rests are NOT encoded, only played notes.

This format is intended to be used both as an in-memory format for generating data into a buffer, though it may serve as (part of) a wire format as well.

To do this, let's start with a single variable length primitive. This is roughly based on [varints](https://postcard.jamesmunns.com/wire-format.html#varint-encoded-integers), which I use in postcard.

Note: This is a sketch, not a formal spec proposal.

## `KCint`s

Let's make a number format that encodes 8 OR 16-bit numbers.

> authors note: 816 is the area code of Kansas City. Since these are 8/16-bit numbers, it
> made sense to me. Not to be confused with [Kansas City Standard](https://en.wikipedia.org/wiki/Kansas_City_standard).

The shorter, 8-bit variants will look like this:

`[0b0xxx_xxxx]` - One flag bit (zero), and 7 data bits. 0..=127

`[0b1xxx_xxxx, 0byyyy_yyyy]` - One flag bit (one) and 15 data bits, little endian. 0..=32767.

UNLIKE varints, the "continuation" bit only exists in the first, least significant byte, which means we have seven bits of data in one byte, or fifteen bits of data in a two-byte pair.

## Pitch

### 7-bit mode

Here, I'm going to steal a page from [MIDI encoding](https://www.audiolabs-erlangen.de/resources/MIR/FMP/C1/C1S2_MIDI.html) of pitch. Basically, `0` maps to `C0`, which has a freqency of 16.35Hz, and each number basically "walks up the keyboard", or increments the frequency by one semitone. [This table](https://pages.mtu.edu/~suits/notefreqs.html) also shows how the frequencies nonlinearly increase by semitone.

This gives us a frequency range of 16.25Hz (`C0`), up to 25kHz or so (`G10`), which is pretty much beyond the full range of human hearing in both directions.

### 15-bit mode

BUT! You might not want *exactly* a note from the scale! You might be doing something atonal, or may want to use a non-western scale.

For that, the lower 7 bits retain the same meaning as a "base note", and the upper 8 bits encode a linear offset towards the next note.

Let's say we start with `C4`, or `60`, or `0x3C` as our base note:

* `[0x3C]` - `C4`, 261.63Hz
* `[0x3D]` - `C#4`, 277.18Hz
* `[0xBC, 0x00`] - `C4`, 00% (000 / 256) to `C#4`, still 261.63Hz
* `[0xBC, 0x80`] - `C4`, 50% (128 / 256) to `C#4`, 269.41Hz
* `[0xBC, 0xC0`] - `C4`, 75% (192 / 256) to `C#4`, 273.29Hz

I'm not sure linear interpolation is the best choice here, so consider that subject to change.

## Starting Beat

Here, I plan to break blocks (somehow) into 64-quarter-note blocks. This matches 16 bars at a 4:4 time signature.

### 7-bit mode

In 7 bit mode, the starting number represents the index of eighth notes from the start.

This means that `0x00` maps to bar one, eighth note one. `0x7F` maps to bar 16, eighth note eight.

### 15-bit mode

In 15 bit mode, I again steal a page from MIDI and other audio equipment, and use the concep of [Pulses Per Quarter Note](https://en.wikipedia.org/wiki/Pulses_per_quarter_note). This is basically an integer divider for the "resolution" available within a single quarter note.

I plan to use 192 PPQN, which means I can accurately reproduce 64th note offsets as well as 32nd note triplet offsets. That's good enough for me.

This mode also indexes from the start of the 16-bar segment, which means:

* `0x80, 0x00` - (0) is the down beat of the first note of the first bar
* `0xC0, 0x01` - (192) is the down beat of the second quarter note of the first bar
* `0xFF, 0x5F` - (12287) is the last 64th note of the 16th bar
* All values above 12287 are invalid for now (treated as a decode error)

## Length

This is the length the note is to be held.

### 7-bit mode

In 7-bit mode, we use "regular" common note values.

* 0x00 - 32nd note triplet
* 0x01 - 16th note triplet
* 0x02 - 8th note triplet
* 0x03 - quarter note triplet
* 0x04 - half note triplet
* 0x05 - 64th note
* 0x06 - 32nd note
* 0x07 - 16th note
* 0x08 - 8th note
* 0x09..=0x3F - RESERVED (decode error)
* 0x40..=0x7F - Sustain, quarter note increments (see below)

For values 0x40..=0x7F, the note will be the (number of quarter note beats - 0x3F).

So:

* 0x40 - quarter note
* 0x41 - half note
* 0x43 - whole note
* 0x45 - whole + half
* 0x47 - two whole
* 0x7F - 16x whole notes

### 15-bit mode

In 15-bit mode, we go back to 192 PPQN counts. Currently a length of zero is reserved (decode error).

This means:

* `0x80, 0x01` - (1) would be 1/3 of a 64th note
* `0xC0, 0x01` - (192) would be the same as a quarter note
* `0xA0, 0x02` - (288) would be a quarter + an eighth

Anything > 12288 (`192 * 16 * 4`) will be treated as a decode error.

## Framing

Currently, this doesn't tackle the concepts of framing, or how we stitch together 16-bar chunks, or how to do fancy things like repeated blocks, or anything like that. I imagine the format described above will be "wrapped" in some slightly more comprehensive format.
