/**
 * Synthetic scripts for the book tests: a command list in the wire shape,
 * with assets the tests name. Not a test file itself, so vitest does not run it.
 */
import type { StoryAssets } from "#/types/generated/StoryAssets";
import type { StoryScript } from "#/types/generated/StoryScript";

export type Cmd = [kind: string, args?: Record<string, string>, text?: string];

export function scriptOf(id: string, commands: Cmd[], assets: Partial<StoryAssets> = {}, synopsis?: string): StoryScript {
    return {
        id,
        name: id,
        groupId: "g",
        wordCount: 0,
        synopsis,
        commands: commands.map(([kind, args = {}, text], i) => ({ kind, args, text, line: i + 1 })),
        assets: { backgrounds: {}, images: {}, characters: {}, music: {}, sounds: {}, avatars: {}, imageSizes: {}, videos: {}, ...assets },
    };
}

/** `[name=X] text`. */
export const say = (speaker: string, text: string): Cmd => ["name", { name: speaker }, text];
export const decide = (options: string[], values: string[]): Cmd => ["decision", { options: options.join(";"), values: values.join(";") }];
export const gate = (...refs: string[]): Cmd => ["predicate", refs.length === 0 ? {} : { references: refs.join(";") }];
