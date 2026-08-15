import { ModifierSlab } from "frontend";

// Slot 03 of a randomizer roll: the challenge modifier. Its `kind` picks both the
// icon and the eyebrow label, so the three stories below sweep that axis with
// real entries from the challenge registry.

const noop = () => {};

export const Restriction = () => (
    <ModifierSlab
        challenge={{
            id: "ranged-only",
            type: "SQUAD_FILTER",
            kind: "restriction",
            title: "Ranged only",
            description: "Only ranged operators allowed.",
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
            title: "No retreating",
            description: "Once deployed, no operator may be retreated.",
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
            title: "Annihilation: 1P Relay",
            description: "You must do a 1 operator relay (eg. only one operator deployed at a time).",
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
            title: "Come to My Side",
            description: "Operators may only be deployed within the arrow-rain range (eg. Texas skill2 range) of other operators (the first operator is exempt).",
        }}
        onReroll={noop}
    />
);
