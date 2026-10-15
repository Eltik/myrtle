import { ModifierSlab } from "frontend";

// Slot 03 of a randomizer roll: the challenge modifier. Its `kind` picks both the
// icon and the eyebrow label, so the three stories below sweep that axis with
// real entries from the challenge registry. Since the i18n migration a
// challenge carries message keys (`titleKey`/`descKey`), resolved under `tools`.

const noop = () => {};

export const Restriction = () => (
    <ModifierSlab
        challenge={{
            id: "ranged-only",
            type: "SQUAD_FILTER",
            kind: "restriction",
            titleKey: "randomizer.challenge.rangedOnly.title",
            descKey: "randomizer.challenge.rangedOnly.desc",
            filter: (op: { position: string }) => op.position === "RANGED",
        }}
        onReroll={noop}
    />
);

export const Modifier = () => (
    <ModifierSlab
        challenge={{
            id: "no-retreat",
            type: "PLAIN",
            kind: "modifier",
            titleKey: "randomizer.challenge.noRetreat.title",
            descKey: "randomizer.challenge.noRetreat.desc",
        }}
        onReroll={noop}
    />
);

export const Objective = () => (
    <ModifierSlab
        challenge={{
            id: "annihilation-one-operator",
            type: "STAGE",
            kind: "objective",
            titleKey: "randomizer.challenge.annihilationOneOperator.title",
            descKey: "randomizer.challenge.annihilationOneOperator.desc",
            match: (stage: { stageType: string }) => stage.stageType === "CAMPAIGN",
        }}
        onReroll={noop}
    />
);

// A long modifier body — the copy wraps to `max-w-prose` under the oversized title.
export const LongRule = () => (
    <ModifierSlab
        challenge={{
            id: "come-to-my-side",
            type: "PLAIN",
            kind: "modifier",
            titleKey: "randomizer.challenge.comeToMySide.title",
            descKey: "randomizer.challenge.comeToMySide.desc",
        }}
        onReroll={noop}
    />
);
