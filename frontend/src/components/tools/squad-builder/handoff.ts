/**
 * "Open in DPS calculator": one seat plus up to three squadmates, as the DPS
 * calculator's `?import=` payload. A seat's buffs are the analysis hand-off's
 * counted sources whose giver is among the seats sent along, re-summed, so a
 * buff never arrives without the operator who gives it.
 */

import type { SeatHandoff } from "#/types/generated/SeatHandoff";
import { DPS_SQUADMATE_LIMIT } from "./constants";
import { type IDpsImport, type IDpsImportSeat, type IResolvedSeat, seatAt, toMemberBuild } from "./model";

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
    const at = (i: number): IResolvedSeat | null => seatAt(seats, support, i);
    const order = [seat, ...[...new Set(picked)].filter((i) => i !== seat && at(i) !== null).sort((a, b) => a - b)].slice(0, DPS_SQUADMATE_LIMIT + 1);
    const sent = new Set(order);
    const out: IDpsImportSeat[] = [];
    for (const i of order) {
        const r = at(i);
        if (!r) continue;
        // The analysis body's build, renamed field by field; this order is the `?import=` JSON's.
        const b = toMemberBuild(r);
        out.push({
            charId: b.char_id,
            phase: b.phase,
            level: b.level,
            potential: b.potential,
            skillIndex: b.skill_index,
            skillLevel: b.skill_level,
            mastery: b.mastery,
            moduleId: b.module_id,
            moduleLevel: b.module_level,
            buffs: seatBuffs(
                handoff.find((h) => h.seat === i),
                sent,
            ),
        });
    }
    return { seats: out };
}
