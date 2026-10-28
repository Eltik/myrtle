/**
 * How a player can get an operator without headhunting, read from game data only.
 *
 * Two facts, both derived on the server from the tables and so true of an operator
 * added after this file was written:
 *
 * - `obtainChannel`, the language-neutral category of `character_table.itemObtainApproach`
 *   (`ObtainChannel::classify`). Anything but `headhunting` is welfare: event rewards
 *   (including the Contingency Contract shop and main story awards), voucher exchange
 *   (the certificate shops), Integrated Strategies, the credit store, gift packs and
 *   the story ("Main Theme Story", Amiya).
 * - The recruitment pool, the operators `gacha_table.recruitDetail` names (the
 *   `/operators/recruitment` endpoint). Their channel reads `headhunting` because the
 *   game's one value is "Recruitment & Headhunting", yet recruitment permits are free:
 *   Texas, Specter, Silence and every recruitment-only 1-4★ are in this set.
 *
 * "Gachaless" is the union, the pool a no-headhunting account draws from. Measured
 * 2026-10-28 against the community gachaless account sheet (EN tree): of its 64 free
 * operators 61 classify welfare, 2 are not in any tree yet, and Snow Hunter reads
 * "Recruitment & Headhunting" on EN and CN; of the 63 operators that account owns, 62
 * classify gachaless and Botani reads "Recruitment & Headhunting" on EN (an event
 * reward on CN).
 */
import type { ObtainChannel } from "#/types/generated/ObtainChannel";

export type ObtainSource = "headhunting" | "welfare";

/**
 * `null` is an operator with no approach: the backend stores the table's missing value
 * as a `null` channel for the `isNotObtainable` rows (Reserve Operators, IS-only
 * temporaries). Neither side claims those.
 */
export function sourceOfChannel(channel: ObtainChannel | null): ObtainSource | null {
    if (channel === null) return null;
    return channel === "headhunting" ? "headhunting" : "welfare";
}

/** The acquisition filter a picker offers, broadest first. */
export const ACQUISITION_FILTERS = ["any", "gachaless", "welfare", "recruitment"] as const;
export type AcquisitionFilter = (typeof ACQUISITION_FILTERS)[number];

/** Whether the filter reads the recruitment pool, so a caller loads it only then. */
export function needsRecruitPool(filter: AcquisitionFilter): boolean {
    return filter === "gachaless" || filter === "recruitment";
}

export interface IAcquirableOperator {
    id: string;
    /** Absent in a cached response from before the field existed; reads as unknown. */
    obtainChannel?: ObtainChannel | null;
}

/**
 * `recruitable` is the set of operator ids the recruitment pool names. An operator
 * whose channel is unknown matches only by being in that set.
 */
export function matchesAcquisition(op: IAcquirableOperator, filter: AcquisitionFilter, recruitable: ReadonlySet<string>): boolean {
    if (filter === "any") return true;
    const welfare = op.obtainChannel != null && sourceOfChannel(op.obtainChannel) === "welfare";
    if (filter === "welfare") return welfare;
    const recruit = recruitable.has(op.id);
    if (filter === "recruitment") return recruit;
    return welfare || recruit;
}
