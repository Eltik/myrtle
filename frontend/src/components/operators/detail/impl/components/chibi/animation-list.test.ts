import { describe, expect, it } from "vitest";
import { animationGroup, clipInfo, compareAnimationNames, groupAnimations, nextClipName } from "./animation-list";

describe("animationGroup", () => {
    it("reads the group from the clip name", () => {
        expect(animationGroup("Idle")).toBe("idle");
        expect(animationGroup("Default")).toBe("idle");
        expect(animationGroup("Relax")).toBe("idle");
        expect(animationGroup("Attack_Begin")).toBe("attack");
        expect(animationGroup("Combat")).toBe("attack");
        expect(animationGroup("Skill_2_Loop")).toBe("skill");
        expect(animationGroup("Move_Loop")).toBe("move");
        expect(animationGroup("Start")).toBe("move");
        expect(animationGroup("Die")).toBe("other");
        expect(animationGroup("Stun")).toBe("other");
    });
});

describe("compareAnimationNames", () => {
    it("orders skill phases in play order and skills by number", () => {
        const names = ["Skill_2_End", "Skill_10_Begin", "Skill_2_Begin", "Skill_2_Loop", "Skill_1_Idle", "Skill_2_Idle", "Skill_1"];
        expect([...names].sort(compareAnimationNames)).toEqual(["Skill_1", "Skill_1_Idle", "Skill_2_Begin", "Skill_2_Loop", "Skill_2_Idle", "Skill_2_End", "Skill_10_Begin"]);
    });

    it("puts an unnumbered skill's phases before the numbered skills", () => {
        expect(["Skill_2", "Skill_End", "Skill_2_Begin", "Skill", "Skill_Begin"].sort(compareAnimationNames)).toEqual(["Skill_Begin", "Skill", "Skill_End", "Skill_2_Begin", "Skill_2"]);
    });

    it("plays a bare stem between its begin and end", () => {
        expect(["Attack_End", "Attack", "Attack_Begin"].sort(compareAnimationNames)).toEqual(["Attack_Begin", "Attack", "Attack_End"]);
    });
});

describe("clipInfo", () => {
    it("counts frames at the stepper's rate and lists the event timeline's events in time order", () => {
        const info = clipInfo(
            {
                name: "Attack",
                duration: 1.2,
                timelines: [
                    { curves: [] },
                    {
                        events: [
                            { data: { name: "OnAttack" }, time: 0.4 },
                            { data: { name: "OnFx" }, time: 0.1 },
                        ],
                    },
                ],
            },
            30,
        );
        expect(info.frames).toBe(36);
        expect(info.group).toBe("attack");
        expect(info.events).toEqual([
            { name: "OnFx", time: 0.1 },
            { name: "OnAttack", time: 0.4 },
        ]);
    });

    it("never reports zero frames", () => {
        expect(clipInfo({ name: "Default", duration: 0, timelines: [] }, 30).frames).toBe(1);
    });
});

describe("groupAnimations / nextClipName", () => {
    const clips = ["Die", "Skill_1_End", "Idle", "Attack", "Skill_1_Begin", "Skill_1_Loop", "Move"].map((name) => clipInfo({ name, duration: 1, timelines: [] }, 30));
    const groups = groupAnimations(clips);

    it("drops empty groups and keeps the group order", () => {
        expect(groups.map((group) => group.id)).toEqual(["idle", "move", "attack", "skill", "other"]);
        expect(groups.find((group) => group.id === "skill")?.clips.map((clip) => clip.name)).toEqual(["Skill_1_Begin", "Skill_1_Loop", "Skill_1_End"]);
    });

    it("chains through the list in order and wraps", () => {
        expect(nextClipName(groups, "Skill_1_Begin")).toBe("Skill_1_Loop");
        expect(nextClipName(groups, "Skill_1_End")).toBe("Die");
        expect(nextClipName(groups, "Die")).toBe("Idle");
    });
});
