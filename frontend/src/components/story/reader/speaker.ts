/**
 * WHICH SPRITE IS SPEAKING, and what colour that makes the speaker plate.
 *
 * The user asked for "the main colour scheme for each character". The source
 * is NOT the skin table's colour list: that list is per OUTFIT, so it has one
 * answer for every story sprite of a skin and no answer at all for the sprites
 * a character wears in a story where she is not in that costume. The source is
 * the STORY SPRITE ON SCREEN, sampled from the body PNG the stage has already
 * fetched (`#/lib/story/palette`, `spritePalette`).
 *
 * The join from the halt's `speaker` to a sprite is the SCENE, because nothing
 * on the wire connects the two: a `[name="Amiya"]` line carries a display name
 * and a `[charslot]` carries a sprite key, and the corpus writes the name in
 * whatever form the writer felt like. What the client does instead is LIGHT the
 * speaking slot, which is `[Character(focus=N)]` and the `lit` flag the engine
 * already tracks, so the lit slot IS the speaker and the rule below is only
 * about what to do when the lighting is ambiguous.
 *
 * Everything here but {@link useSpeakerTint} is pure, so the rule is tested
 * against hand-built slot states with no DOM and no image decode.
 */

import { useEffect, useRef, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { cachedSpritePalette, spritePalette } from "#/lib/story/palette";
import type { Slot, SlotState } from "#/lib/story/scene";
import type { SpeakerTint, StorySettings } from "#/lib/story/settings";
import { contrastRatio, DARK_BOX_WORST, fitToBox, LIGHT_BOX_WORST, TEXT_CONTRAST_TARGET } from "./colors";

/**
 * The sprite a line belongs to.
 *
 * Exactly one lit slot is the whole answer and it is the common case. Two or
 * more lit is `[Character]` with no focus, or a `[charslot]` scene that lights
 * everyone, and there the tie-break is RECENCY: `swap` bumps every time a slot
 * takes a different sprite, so the highest `swap` is the slot this step just
 * changed, which is the one the line is about. A tie on `swap` falls back to
 * the engine's own frame-state order, which is insertion order over `l`, `m`
 * and `r`, so the first lit slot wins and the choice is deterministic.
 *
 * Nothing lit and ONE sprite on stage is still that sprite, and this case is
 * not hypothetical: `[charslot(focus="n")]` dims every slot, and the corpus
 * writes it 2,214 times as `n` plus 1,506 as `none`, 4.40% of the 84,586 focus
 * values in the EN scripts. `main_15_level_main_15-08_end` uses it before
 * nearly every line, so a rule that needed a LIT slot would have fallen back to
 * the hash for that whole story with Amiya alone on the stage. With one sprite
 * up there is nothing to disambiguate, so the lighting does not have to say.
 *
 * Nothing lit and TWO OR MORE on stage is genuinely ambiguous, and so is
 * nothing on stage at all (narration, an off-screen voice, a black screen):
 * both hand the caller a null and the hash hue stands.
 */
export function litSprite(slots: Partial<Record<Slot, SlotState>>): SlotState | null {
    const lit: SlotState[] = [];
    const all: SlotState[] = [];
    for (const key of Object.keys(slots) as Slot[]) {
        const s = slots[key];
        if (!s) continue;
        all.push(s);
        if (s.lit) lit.push(s);
    }
    if (lit.length === 0) return all.length === 1 ? (all[0] as SlotState) : null;
    if (lit.length === 1) return lit[0] as SlotState;
    let best = lit[0] as SlotState;
    for (const s of lit) if (s.swap > best.swap) best = s;
    return best;
}

/**
 * The remembered name-to-ink pairing for one story session.
 *
 * A character speaks from off stage all the time: she walks out of the scene
 * and keeps talking, or a line lands on a black screen between two shots. With
 * no memory her plate would flip from her own ink to the hash hue and back
 * mid-conversation. FIRST SEEN WINS, so the colour she is introduced in is the
 * one she keeps, and the map is cleared when the story changes because the same
 * name in another story is another scene.
 */
export type SpeakerInks = Map<string, string>;

/**
 * The RAW ink for a speaker: the lit sprite's dominant colour, else what this
 * name was already paired with, else null for "fall back to the hash hue".
 *
 * `paletteFor` is the lookup, never a fetch, so this stays synchronous and
 * pure apart from the one documented write into `remembered`.
 */
export function speakerInk(opts: { speaker: string | undefined; slots: Partial<Record<Slot, SlotState>>; paletteFor: (bodyUrl: string) => { c1: string } | undefined; remembered: SpeakerInks }): string | null {
    const { speaker, slots, paletteFor, remembered } = opts;
    if (!speaker) return null;
    const sprite = litSprite(slots);
    const sampled = sprite ? paletteFor(sprite.sprite.bodyUrl)?.c1 : undefined;
    if (sampled) {
        if (!remembered.has(speaker)) remembered.set(speaker, sampled);
        return remembered.get(speaker) ?? sampled;
    }
    return remembered.get(speaker) ?? null;
}

/**
 * Which surfaces the speaker's colour reaches, per mode.
 *
 * The LINE never takes a colour the plate does not: "Text and name" is one
 * voice printed twice, and a line inked differently from the name over it
 * reads as two speakers. So `line` implies `plate` and the third combination,
 * a tinted line under a hashed plate, is deliberately not offered.
 */
export function speakerTintTargets(mode: SpeakerTint): { plate: boolean; line: boolean } {
    return { plate: mode !== "off", line: mode === "text" };
}

/**
 * A speaker colour made safe to print a whole LINE in.
 *
 * A sampled ink arrives from {@link fitToBox} already clearing 4.5:1 on this
 * surface's worst composite, and is handed back BYTE-IDENTICAL, because a
 * round trip through HSL can move a channel by one and the line must match the
 * plate exactly. The hashed hue is the case that needs the walk, and it needs
 * it every time: MEASURED against #4d4d4d, the seven dark plate hues run 2.951
 * (#e07a8c) to 4.447 (#6fcf97) and ALL SEVEN fail 4.5:1, because they were
 * curated against the plate's own near-black chip (#0a0a0a) and not against
 * the box. The seven light hues pass on #e8e5de at 5.830 to 7.614, so on the
 * light surface the walk returns on step zero and nothing moves.
 */
export function readableLineTint(color: string, light: boolean): string {
    const surface = light ? LIGHT_BOX_WORST : DARK_BOX_WORST;
    return contrastRatio(color, surface) >= TEXT_CONTRAST_TARGET ? color : fitToBox(color, light);
}

/**
 * The colour the DIALOGUE LINE is printed in, or undefined for "the box keeps
 * its own".
 *
 * NARRATION keeps the reader's text colour in every mode, because it has no
 * speaker: tinting it prints a voice on a line nobody is saying. A spoken line
 * takes the plate's colour on `text` alone, and the plate is the sampled ink
 * when a sprite answered for this name and the hashed hue when none did.
 */
export function lineColorFor(opts: { mode: SpeakerTint; isNarration: boolean; plateColor: string; textColor: string | undefined; light: boolean }): string | undefined {
    const { mode, isNarration, plateColor, textColor, light } = opts;
    if (!speakerTintTargets(mode).line || isNarration || plateColor === "") return textColor;
    return readableLineTint(plateColor, light);
}

/**
 * The speaker's tint for the current line, already conditioned for the surface
 * the box is on, or undefined for "the reader's own hash hue stands".
 *
 * Undefined is the DEFAULT and the kill switch: with `speakerTint` on "off"
 * the hook never samples, never stores and never returns a colour, so the
 * plate is byte-identical to what the reader shipped.
 */
export function useSpeakerTint(opts: { speaker: string | undefined; slots: Partial<Record<Slot, SlotState>>; settings: StorySettings; storyId: string }): string | undefined {
    const { speaker, slots, settings, storyId } = opts;
    const on = speakerTintTargets(settings.speakerTint).plate;
    const remembered = useRef<SpeakerInks>(new Map());
    // A counter rather than the palette itself: the sample resolves into the
    // module cache, and the render reads it from there, so one number is all
    // the component needs to re-read every sprite at once.
    const [sampled, setSampled] = useState(0);

    // A name keeps its ink for the STORY, so a new story starts over. Done in
    // render rather than in an effect because the ink below is read in the same
    // pass; an effect would print the previous story's colour for one frame.
    const storyRef = useRef(storyId);
    if (storyRef.current !== storyId) {
        storyRef.current = storyId;
        remembered.current = new Map();
    }

    const sprite = on ? litSprite(slots) : null;
    const bodyUrl = sprite ? asset(sprite.sprite.bodyUrl) : null;
    useEffect(() => {
        if (!bodyUrl) return;
        let live = true;
        void spritePalette(bodyUrl).then(() => {
            if (live) setSampled((n) => n + 1);
        });
        return () => {
            live = false;
        };
    }, [bodyUrl]);

    if (!on) return undefined;
    // `sampled` is read so the memo below re-runs when a sample lands.
    void sampled;
    const ink = speakerInk({ speaker, slots, paletteFor: (url) => cachedSpritePalette(asset(url)), remembered: remembered.current });
    return ink ? fitToBox(ink, settings.lightBox) : undefined;
}
