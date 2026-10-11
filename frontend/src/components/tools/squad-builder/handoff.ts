/**
 * "Open in DPS calculator": one seat plus up to three squadmates, as the DPS
 * calculator's `?import=` payload. A seat's buffs are the analysis hand-off's
 * counted sources whose giver is among the seats sent along, re-summed, so a
 * buff never arrives without the operator who gives it.
 */

import type { SeatHandoff } from "#/types/generated/SeatHandoff";
import { DPS_SQUADMATE_LIMIT } from "./constants";
import { type IDpsImport, type IDpsImportSeat, type IResolvedSeat, SUPPORT_SEAT } from "./model";

export { DPS_SQUADMATE_LIMIT } from "./constants";

export function seatBuffs(handoff: SeatHandoff | undefined, sentSeats: ReadonlySet<number>): IDpsImportSeat["buffs"] {
    const out = { atk: 0, flatAtk: 0, aspd: 0 };
    for (const src of handoff?.counted ?? []) {
        if (!sentSeats.has(src.from_seat)) continue;
        out.atk += src.atk;
        out.flatAtk += src.flat_atk;
        out.aspd += src.aspd;
    }
    // Floating sums of decimals (0.1 + 0.2) read badly in an input field.
    out.atk = Math.round(out.atk * 10000) / 10000;
    return out;
}

/**
 * `seat` first, then the picked squadmates in seat order. `seats` is indexed
 * by seat; the support, when sent, is `SUPPORT_SEAT`.
 */
export function buildDpsImport(seat: number, picked: readonly number[], seats: ReadonlyArray<IResolvedSeat | null>, support: IResolvedSeat | null, handoff: readonly SeatHandoff[]): IDpsImport {
    const at = (i: number): IResolvedSeat | null => (i === SUPPORT_SEAT ? support : (seats[i] ?? null));
    const order = [seat, ...[...new Set(picked)].filter((i) => i !== seat && at(i) !== null).sort((a, b) => a - b)].slice(0, DPS_SQUADMATE_LIMIT + 1);
    const sent = new Set(order);
    const out: IDpsImportSeat[] = [];
    for (const i of order) {
        const r = at(i);
        if (!r) continue;
        out.push({
            charId: r.member.char_id,
            phase: r.build.phase,
            level: r.build.level,
            potential: r.build.potential,
            skillIndex: r.member.skill_index,
            skillLevel: r.build.skill_level,
            mastery: r.build.mastery,
            moduleId: r.member.module_id,
            moduleLevel: r.member.module_id ? r.build.module_level : 0,
            buffs: seatBuffs(
                handoff.find((h) => h.seat === i),
                sent,
            ),
        });
    }
    return { seats: out };
}
