import { addTierListPlacementFn, createTierFn, deleteTierFn, moveTierListPlacementFn, removeTierListPlacementFn, updateTierFn, updateTierListFn, updateTierListPlacementDescriptionFn } from "#/lib/api/tier-lists";
import { type IEditState, type IEditTier, isDraftId } from "./state";

function descFor(state: IEditState, operatorId: string): string {
    return (state.descriptionByOperatorId[operatorId] ?? "").trim();
}

export interface ISaveProgress {
    step: number;
    total: number;
    label: string;
}

type ProgressFn = (p: ISaveProgress) => void;

interface ISaveContext {
    slug: string;
    original: IEditState;
    current: IEditState;
    onProgress?: ProgressFn;
}

/**
 * Park range for `display_order` while reordering. Set well above any
 * realistic tier count so it never collides with a "real" order. The backend
 * enforces UNIQUE(tier_list_id, display_order), so reorders that swap two
 * existing tiers must briefly stash one at a non-colliding value before
 * settling.
 */
const PARK_BASE = 10000;

interface IExistingChange {
    tier: IEditTier;
    finalIndex: number;
    orderChanged: boolean;
}

/**
 * Applies the edit diff against the backend in a sequence that respects the
 * UNIQUE(tier_list_id, display_order) constraint:
 *   1. List metadata
 *   2. Park existing tiers whose `display_order` is changing (also flushes
 *      any rename/recolor/description edits in the same call)
 *   3. Delete removed tiers (frees up their original order slots)
 *   4. Create new tiers at their final `display_order`
 *   5. Settle parked tiers back into their final `display_order`
 *   6. Field-only updates for existing tiers whose order is unchanged
 *   7. Reconcile placements (remove → move → add). Draft tier ids are
 *      resolved as tiers are created so placement calls target real ids.
 */
export async function saveEdits({ slug, original, current, onProgress }: ISaveContext): Promise<void> {
    const operations: Array<{ label: string; run: () => Promise<void> }> = [];

    if (original.title !== current.title || original.description !== current.description) {
        operations.push({
            label: "Saving list details",
            run: async () => {
                await updateTierListFn({ data: { slug, name: current.title, description: current.description || null } });
            },
        });
    }

    const origTierById = new Map(original.tiers.map((t) => [t.id, t] as const));
    const currTierById = new Map(current.tiers.map((t) => [t.id, t] as const));
    const origTierIndex = new Map(original.tiers.map((t, i) => [t.id, i] as const));

    const draftIdToReal = new Map<string, string>();

    const existingChanges: IExistingChange[] = [];
    current.tiers.forEach((tier, finalIndex) => {
        if (isDraftId(tier.id)) return;
        const orig = origTierById.get(tier.id);
        if (!orig) return;
        const origOrder = origTierIndex.get(tier.id) ?? -1;
        const orderChanged = origOrder !== finalIndex;
        const fieldsChanged = orig.name !== tier.name || orig.color !== tier.color || orig.description !== tier.description;
        if (!orderChanged && !fieldsChanged) return;
        existingChanges.push({ tier, finalIndex, orderChanged });
    });

    const reorderingExistingChanges = existingChanges.filter((c) => c.orderChanged);

    for (const { tier, finalIndex } of reorderingExistingChanges) {
        operations.push({
            label: `Reordering "${tier.name}"`,
            run: async () => {
                await updateTierFn({
                    data: {
                        slug,
                        tierId: tier.id,
                        name: tier.name,
                        displayOrder: PARK_BASE + finalIndex,
                        color: tier.color || null,
                        description: tier.description || null,
                    },
                });
            },
        });
    }

    for (const t of original.tiers) {
        if (currTierById.has(t.id)) continue;
        operations.push({
            label: `Deleting tier "${t.name}"`,
            run: async () => {
                await deleteTierFn({ data: { slug, tierId: t.id } });
            },
        });
    }

    current.tiers.forEach((t, i) => {
        if (!isDraftId(t.id)) return;
        operations.push({
            label: `Creating tier "${t.name}"`,
            run: async () => {
                const created = await createTierFn({
                    data: {
                        slug,
                        name: t.name,
                        displayOrder: i,
                        color: t.color || null,
                        description: t.description || null,
                    },
                });
                draftIdToReal.set(t.id, created.id);
            },
        });
    });

    for (const { tier, finalIndex } of reorderingExistingChanges) {
        operations.push({
            label: `Settling "${tier.name}"`,
            run: async () => {
                await updateTierFn({
                    data: {
                        slug,
                        tierId: tier.id,
                        name: tier.name,
                        displayOrder: finalIndex,
                        color: tier.color || null,
                        description: tier.description || null,
                    },
                });
            },
        });
    }

    for (const { tier, finalIndex, orderChanged } of existingChanges) {
        if (orderChanged) continue;
        operations.push({
            label: `Updating "${tier.name}"`,
            run: async () => {
                await updateTierFn({
                    data: {
                        slug,
                        tierId: tier.id,
                        name: tier.name,
                        displayOrder: finalIndex,
                        color: tier.color || null,
                        description: tier.description || null,
                    },
                });
            },
        });
    }

    const origPlacement = new Map<string, { tierId: string; subOrder: number }>();
    for (const t of original.tiers) {
        for (const [i, opId] of t.operatorIds.entries()) origPlacement.set(opId, { tierId: t.id, subOrder: i });
    }

    const currPlacement = new Map<string, { tier: IEditTier; subOrder: number }>();
    for (const t of current.tiers) {
        for (const [i, opId] of t.operatorIds.entries()) currPlacement.set(opId, { tier: t, subOrder: i });
    }

    for (const [opId, then] of origPlacement) {
        const now = currPlacement.get(opId);
        if (!now) {
            operations.push({
                label: `Removing operator`,
                run: async () => {
                    await removeTierListPlacementFn({ data: { slug, operatorId: opId } });
                },
            });
            continue;
        }
        if (!isDraftId(now.tier.id) && now.tier.id === then.tierId && now.subOrder === then.subOrder) continue;
        operations.push({
            label: `Moving operator`,
            run: async () => {
                const resolvedTierId = isDraftId(now.tier.id) ? draftIdToReal.get(now.tier.id) : now.tier.id;
                if (!resolvedTierId) throw new Error(`Tier "${now.tier.name}" wasn't created`);
                await moveTierListPlacementFn({ data: { slug, operatorId: opId, newTierId: resolvedTierId, subOrder: now.subOrder } });
            },
        });
    }

    for (const [opId, now] of currPlacement) {
        if (origPlacement.has(opId)) continue;
        operations.push({
            label: `Placing operator`,
            run: async () => {
                const resolvedTierId = isDraftId(now.tier.id) ? draftIdToReal.get(now.tier.id) : now.tier.id;
                if (!resolvedTierId) throw new Error(`Tier "${now.tier.name}" wasn't created`);
                await addTierListPlacementFn({ data: { slug, tierId: resolvedTierId, operatorId: opId, subOrder: now.subOrder, description: descFor(current, opId) || null } });
            },
        });
    }

    for (const [opId] of currPlacement) {
        if (!origPlacement.has(opId)) continue;
        const next = descFor(current, opId);
        if (next === descFor(original, opId)) continue;
        operations.push({
            label: `Updating description`,
            run: async () => {
                await updateTierListPlacementDescriptionFn({ data: { slug, operatorId: opId, description: next || null } });
            },
        });
    }

    const total = operations.length;
    for (let i = 0; i < operations.length; i++) {
        const op = operations[i];
        if (!op) continue;
        onProgress?.({ step: i + 1, total, label: op.label });
        await op.run();
    }
}
