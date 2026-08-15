import { AudioContent } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The Audio/SFX tab. Voice lines come from `/static/voices`, which no preview can
// reach, so that panel honestly renders its "no voice data" state; the Battle SFX
// panel is driven by `operator.audio` on the operator payload and renders in full.
//
// Fixture is Młynar's seven resolved FSB5 banks from `/api/operators/char_4064_mlynar`.
const MLYNAR_AUDIO = [
    {
        "bankName": "battle.ON_ABILITY_HIT.char_4064_mlynar.attack.1",
        "event": "ON_ABILITY_HIT",
        "category": "attack",
        "skillSlot": null,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Player/p_imp/p_imp_loyalsword_n",
                "urls": [
                    "/audio/sound_beta_2/player/p_imp_0/p_imp_loyalsword_n.ogg"
                ]
            }
        ]
    },
    {
        "bankName": "battle.ON_ABILITY_START.char_4064_mlynar.attack.2",
        "event": "ON_ABILITY_START",
        "category": "attack",
        "skillSlot": null,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Player/p_imp/p_imp_loyalsword_d",
                "urls": [
                    "/audio/sound_beta_2/player/p_imp_0/p_imp_loyalsword_d.ogg"
                ]
            }
        ]
    },
    {
        "bankName": "battle.ON_ABILITY_START.char_4064_mlynar.attack.3",
        "event": "ON_ABILITY_START",
        "category": "attack",
        "skillSlot": null,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Player/p_imp/p_imp_loyalsword_h",
                "urls": [
                    "/audio/sound_beta_2/player/p_imp_0/p_imp_loyalsword_h.ogg"
                ]
            }
        ]
    },
    {
        "bankName": "battle.ON_ABILITY_START.skchr_mlynar_1",
        "event": "ON_ABILITY_START",
        "category": "skill",
        "skillSlot": 1,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Battle/b_char/b_char_atkboost",
                "urls": [
                    "/audio/sound_beta_2/btl_snd_0/b_char_atkboost.ogg"
                ]
            }
        ]
    },
    {
        "bankName": "battle.ON_ABILITY_START.skchr_mlynar_2",
        "event": "ON_ABILITY_START",
        "category": "skill",
        "skillSlot": 2,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Battle/b_char/b_char_atkboost",
                "urls": [
                    "/audio/sound_beta_2/btl_snd_0/b_char_atkboost.ogg"
                ]
            }
        ]
    },
    {
        "bankName": "battle.ON_ABILITY_START.skchr_mlynar_3",
        "event": "ON_ABILITY_START",
        "category": "skill",
        "skillSlot": 3,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Battle/b_char/b_char_atkboost",
                "urls": [
                    "/audio/sound_beta_2/btl_snd_0/b_char_atkboost.ogg"
                ]
            }
        ]
    },
    {
        "bankName": "battle.ON_SKILL_FINISH.skchr_mlynar_2",
        "event": "ON_SKILL_FINISH",
        "category": "skill",
        "skillSlot": 2,
        "language": null,
        "sounds": [
            {
                "asset": "Audio/Sound_Beta_2/Battle/b_char/b_char_boostclose",
                "urls": [
                    "/audio/sound_beta_2/btl_snd_0/b_char_boostclose.ogg"
                ]
            }
        ]
    }
];

const operator = (audio: unknown) => ({ id: "char_4064_mlynar", name: "Młynar", server: "en", audio });

// The Battle SFX panel is behind an internal tab; click it on mount so the card
// shows the sound-effect rows rather than the default voice panel. Base UI wires
// the tab after the first paint, so the click waits two frames.
const OpenSfxTab = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                const tabs = ref.current?.querySelectorAll("button");
                for (const b of tabs ?? []) {
                    if (b.textContent?.includes("Battle SFX")) {
                        b.click();
                        return;
                    }
                }
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return <div ref={ref}>{children}</div>;
};

export const BattleSfx = () => (
    <OpenSfxTab>
        <AudioContent operator={operator(MLYNAR_AUDIO)} />
    </OpenSfxTab>
);

export const VoiceLinesUnavailable = () => <AudioContent operator={operator(MLYNAR_AUDIO)} />;
