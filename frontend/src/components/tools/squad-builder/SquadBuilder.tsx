import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Share2 } from "lucide-react";
import * as React from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { PageHeader } from "#/components/ui/page-header";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { deleteSquadFn, SQUADS_QUERY_KEY, saveSquadFn, squadAnalysisQueryOptions, squadRulesQueryOptions, squadsQueryOptions } from "#/lib/api/squads";
import { stageDetailQueryOptions } from "#/lib/api/stages";
import { userRosterQueryOptions } from "#/lib/api/user";
import { authActions } from "#/lib/auth/store";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { GameSquad } from "#/types/generated/GameSquad";
import type { RosterEntry } from "#/types/generated/RosterEntry";
import type { SquadMember } from "#/types/generated/SquadMember";
import type { IOperatorIndexEntry } from "#/types/operators";
import { SquadAnalysisPanels } from "./analysis/SquadAnalysisPanels";
import { useDebouncedValue } from "./analysis/useDebouncedValue";
import { REQUEST_DEBOUNCE_MS } from "./constants";
import { buildDpsImport } from "./handoff";
import { OperatorPicker } from "./impl/OperatorPicker";
import { type ISquadmateOption, SeatEditor } from "./impl/SeatEditor";
import { SeatGrid } from "./impl/SeatGrid";
import { ShareDialog } from "./impl/ShareDialog";
import { SlotBar } from "./impl/SlotBar";
import { SlotChooser } from "./impl/SlotChooser";
import { SquadToolbar } from "./impl/SquadToolbar";
import { StageCard } from "./impl/StageCard";
import { StagePicker } from "./impl/StagePicker";
import { advancedModules, decodeShare, emptyDraft, encodeDpsImport, filledSeats, type IRosterBuild, type IShareSearch, type ISquadDraft, memberFromRoster, normalizeMembers, resolveSeat, rosterBuildOf, SQUAD_SIZE, SUPPORT_SEAT, seatAt, toAnalyzeRequest } from "./model";
import { COMPACT_KEY, DRAFT_KEY, readHydration, START_DP_KEY, safeWrite } from "./persist";
import type { messages } from "./SquadBuilder.messages";
import { deriveStageContext, normalStageId } from "./stage";
import { draftFromSaved, INITIAL_STATE, isDirty, nextEmptySeat, squadReducer } from "./state";

type PageT = TypedT<typeof messages>;

/** A new seat's skill and module: the roster's choice when owned, else the top skill and first advanced module, the way a full build is usually run. */
function newMember(op: IOperatorIndexEntry | undefined, charId: string, roster: IRosterBuild | undefined): SquadMember {
    if (roster || !op) return memberFromRoster(charId, roster, op);
    return { char_id: charId, skill_index: Math.max(0, op.skillCount - 1), module_id: advancedModules(op)[0]?.uniEquipId ?? null, build: null };
}

export interface ISquadBuilderProps {
    /** Share-link params from the route; `null` when the URL carries none. */
    share: IShareSearch | null;
    /** Called once the share params are read, so the route can drop them. */
    onShareConsumed: () => void;
}

export function SquadBuilder({ share, onShareConsumed }: ISquadBuilderProps): React.ReactElement {
    const t: PageT = useT("tools");
    const server = useGamedataServer();
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const describeError = useErrorMessage();
    const { user } = useAuth();
    const uid = user?.uid ?? null;
    const signedIn = uid !== null;

    const [state, dispatch] = React.useReducer(squadReducer, INITIAL_STATE);
    const [ready, setReady] = React.useState(false);

    // Storage is read in an effect so the server render and the first client render agree.
    React.useEffect(() => {
        dispatch({ type: "HYDRATE", ...readHydration() });
        setReady(true);
    }, []);

    React.useEffect(() => {
        if (!ready || !share) return;
        const shared = decodeShare(share);
        if (shared) dispatch({ type: "OPEN_SHARE", draft: shared });
        onShareConsumed();
    }, [ready, share, onShareConsumed]);

    // A shared squad on screen is not the player's: keep the stashed working copy stored instead.
    const persisted = state.stash ?? state;
    React.useEffect(() => {
        if (ready) safeWrite(DRAFT_KEY, { draft: persisted.draft, slot: persisted.slot, savedBaseline: persisted.savedBaseline });
    }, [ready, persisted.draft, persisted.slot, persisted.savedBaseline]);
    React.useEffect(() => {
        if (ready) safeWrite(START_DP_KEY, { startDp: state.startDp, from: state.startDpFrom });
    }, [ready, state.startDp, state.startDpFrom]);
    React.useEffect(() => {
        if (ready) safeWrite(COMPACT_KEY, state.compact);
    }, [ready, state.compact]);

    const { data: operatorList } = useQuery(operatorsIndexQueryOptions(server));
    const operators = React.useMemo(() => new Map((operatorList ?? []).map((o) => [o.id, o])), [operatorList]);
    const { data: rosterRows } = useQuery({ ...userRosterQueryOptions(uid ?? ""), enabled: uid !== null });
    const roster = React.useMemo(() => new Map((rosterRows ?? []).map((r) => [r.operator_id, rosterBuildOf(r as unknown as RosterEntry)])), [rosterRows]);
    const { data: squads, isLoading: squadsLoading } = useQuery(squadsQueryOptions(uid));
    const saved = squads?.saved ?? [];

    const { draft, readOnlyShare } = state;
    const seats = React.useMemo(
        () =>
            draft.members.map((m) => {
                const op = m ? operators.get(m.char_id) : undefined;
                return m && op ? resolveSeat(m, op, roster.get(m.char_id), false) : null;
            }),
        [draft.members, operators, roster],
    );
    const supportOp = draft.support ? operators.get(draft.support.char_id) : undefined;
    const supportSeat = React.useMemo(() => (draft.support && supportOp ? resolveSeat(draft.support, supportOp, undefined, true) : null), [draft.support, supportOp]);

    const stageId = draft.stageId;
    const stageQuery = useQuery({ ...stageDetailQueryOptions(stageId ?? "", server), enabled: stageId !== null });
    const { data: rules } = useQuery(squadRulesQueryOptions(server));
    const thresholds = rules?.threats ?? null;
    const normalId = stageId ? normalStageId(stageId, rules?.normal_of) : null;
    const normalQuery = useQuery({ ...stageDetailQueryOptions(normalId ?? "", server), enabled: normalId !== null && normalId !== stageId });
    const normalDetail = normalId === stageId ? stageQuery.data : normalQuery.data;
    const challengeId = normalDetail?.stage.hardStagedId ?? null;
    const stageContext = React.useMemo(() => (stageId && stageQuery.data ? deriveStageContext(stageId, stageQuery.data, thresholds) : null), [stageId, stageQuery.data, thresholds]);
    const [stagePickerOpen, setStagePickerOpen] = React.useState(false);

    const applyStage = async (id: string) => {
        setStagePickerOpen(false);
        try {
            const detail = await queryClient.fetchQuery(stageDetailQueryOptions(id, server));
            dispatch({ type: "SET_STAGE", stageId: id });
            const ctx = detail ? deriveStageContext(id, detail, thresholds) : null;
            if (ctx?.startDp != null) dispatch({ type: "SET_START_DP", startDp: ctx.startDp, from: { kind: "stage", code: ctx.code } });
        } catch (err) {
            toastManager.add({ id: `squad-stage-${Date.now()}`, title: t("squad.toast.stageFailed"), description: describeError(err), type: "error" });
        }
    };
    const removeStage = () => {
        dispatch({ type: "SET_STAGE", stageId: null });
        dispatch({ type: "SET_START_DP", startDp: state.startDp, from: { kind: "page" } });
    };

    const filled = filledSeats(draft);
    const request = React.useMemo(() => (filled > 0 && operators.size > 0 ? toAnalyzeRequest(seats, supportSeat, { startDp: state.startDp, deployLimit: stageContext?.deployLimit ?? null, dub: null }) : null), [filled, operators.size, seats, supportSeat, state.startDp, stageContext?.deployLimit]);
    // Debounced once, here: the panels and the hand-off read this one query, so a dragged DP
    // line or a held stepper sends one request when it settles, not one per tick.
    const debouncedRequest = useDebouncedValue(request, REQUEST_DEBOUNCE_MS);
    const analysisQuery = useQuery(squadAnalysisQueryOptions(request === null ? null : debouncedRequest, server));
    const analysis = analysisQuery.data;
    const handoff = analysis?.handoff ?? [];
    const [hoverSeat, setHoverSeat] = React.useState<number | null>(null);

    const [focus, setFocus] = React.useState<number | null>(null);
    const [labOpen, setLabOpen] = React.useState(false);
    const panelRef = React.useRef<HTMLDivElement>(null);
    React.useEffect(() => {
        if (focus !== null) panelRef.current?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }, [focus]);

    const focusedMember = focus === null ? null : seatAt(draft.members, draft.support, focus);
    const focusedResolved = focus === null ? null : seatAt(seats, supportSeat, focus);
    const focusedOp = focusedMember ? operators.get(focusedMember.char_id) : undefined;

    const onFocus = (seat: number) => {
        if (readOnlyShare && !seatAt(draft.members, draft.support, seat)) return;
        setLabOpen(false);
        setFocus((f) => (f === seat ? null : seat));
    };

    /** Seat `member`, or empty the seat with `null`; the support at `SUPPORT_SEAT`. */
    const setSeatMember = (seat: number, member: SquadMember | null) => {
        if (seat === SUPPORT_SEAT) dispatch({ type: "SET_SUPPORT", member });
        else dispatch({ type: "SET_MEMBER", seat, member });
    };

    const onPick = (charId: string) => {
        if (focus === null) return;
        const op = operators.get(charId);
        if (focus === SUPPORT_SEAT) {
            dispatch({ type: "SET_SUPPORT", member: newMember(op, charId, undefined) });
            setFocus(null);
            return;
        }
        dispatch({ type: "SET_MEMBER", seat: focus, member: newMember(op, charId, roster.get(charId)) });
        const after = normalizeMembers(draft.members);
        after[focus] = { char_id: charId, skill_index: 0, module_id: null, build: null };
        setFocus(nextEmptySeat(after, focus));
    };

    const onSwap = (a: number, b: number) => {
        dispatch({ type: "SWAP", a, b });
        setFocus((f) => (f === a ? b : f === b ? a : f));
    };

    const onRemove = (seat: number) => {
        setSeatMember(seat, null);
        setFocus((f) => (f === seat ? null : f));
    };

    const onChangeMember = (member: SquadMember) => {
        if (focus !== null) setSeatMember(focus, member);
    };

    const seated = React.useMemo(() => {
        const out = new Map<string, number>();
        draft.members.forEach((m, i) => {
            if (m) out.set(m.char_id, i);
        });
        if (draft.support) out.set(draft.support.char_id, SUPPORT_SEAT);
        return out;
    }, [draft.members, draft.support]);

    const squadmates = React.useMemo((): ISquadmateOption[] => {
        if (focus === null) return [];
        const givers = new Set((handoff.find((h) => h.seat === focus)?.counted ?? []).map((c) => c.from_seat));
        const out: ISquadmateOption[] = [];
        const add = (seat: number, m: SquadMember | null) => {
            if (!m || seat === focus) return;
            const op = operators.get(m.char_id);
            if (op) out.push({ seat, charId: m.char_id, name: op.name, gives: givers.has(seat) });
        };
        draft.members.forEach((m, i) => {
            add(i, m);
        });
        add(SUPPORT_SEAT, draft.support);
        // Givers first, so the default pick is the squadmates whose buffs carry over.
        return out.sort((a, b) => Number(b.gives) - Number(a.gives) || a.seat - b.seat);
    }, [focus, handoff, draft.members, draft.support, operators]);

    const openLab = (picked: number[]) => {
        if (focus === null) return;
        const data = buildDpsImport(focus, picked, seats, supportSeat, handoff);
        if (data.seats.length === 0) return;
        void navigate({ to: "/tools/dps", search: { import: encodeDpsImport(data) } });
    };
    const openLabFor = (seat: number) => {
        setFocus(seat);
        setLabOpen(true);
    };

    const dirty = !readOnlyShare && isDirty(state);
    const [pendingLoad, setPendingLoad] = React.useState<(() => void) | null>(null);
    const guard = (load: () => void) => {
        if (dirty) setPendingLoad(() => load);
        else load();
    };

    const loadSlot = (slot: number) => {
        if (slot === state.slot) return;
        guard(() => {
            const squad = saved[slot - 1] ?? null;
            const next = squad ? draftFromSaved(squad) : emptyDraft();
            dispatch({ type: "LOAD", draft: next, slot, savedBaseline: squad ? next : null });
            setFocus(null);
        });
    };

    const loadGame = (squad: GameSquad) => {
        guard(() => {
            const next: ISquadDraft = emptyDraft(squad.name.trim() ? t("squad.importedName", { name: squad.name.trim() }) : "");
            next.members = normalizeMembers(squad.slots.map((s) => (s ? { char_id: s.char_id, skill_index: Math.max(0, s.skill_index), module_id: s.module_id, build: null } : null)));
            dispatch({ type: "LOAD", draft: next, slot: null, savedBaseline: null });
            setFocus(null);
        });
    };

    const [chooser, setChooser] = React.useState<"save" | "share" | null>(null);
    const [shareOpen, setShareOpen] = React.useState(false);

    const saveMutation = useMutation({
        mutationFn: ({ slot, squad }: { slot: number; squad: ISquadDraft }) => saveSquadFn({ data: { slot, server, input: { name: squad.name.trim() || t("squad.defaultName", { slot }), members: normalizeMembers(squad.members), support: squad.support, stage_id: squad.stageId } } }),
        onSuccess: (result, { slot }) => {
            dispatch({ type: "SAVED", slot, draft: draftFromSaved(result) });
            setChooser(null);
            void queryClient.invalidateQueries({ queryKey: SQUADS_QUERY_KEY });
            toastManager.add({ id: `squad-saved-${Date.now()}`, title: t("squad.toast.saved", { slot }), type: "success" });
        },
        onError: (err) => toastManager.add({ id: `squad-save-err-${Date.now()}`, title: t("squad.toast.saveFailed"), description: describeError(err), type: "error" }),
    });

    const deleteMutation = useMutation({
        mutationFn: (slot: number) => deleteSquadFn({ data: { slot } }),
        onSuccess: (_r, slot) => {
            dispatch({ type: "SLOT_DELETED" });
            void queryClient.invalidateQueries({ queryKey: SQUADS_QUERY_KEY });
            toastManager.add({ id: `squad-deleted-${Date.now()}`, title: t("squad.toast.deleted", { slot }), type: "success" });
        },
        onError: (err) => toastManager.add({ id: `squad-delete-err-${Date.now()}`, title: t("squad.toast.deleteFailed"), description: describeError(err), type: "error" }),
    });

    const onSave = () => {
        if (!signedIn) {
            authActions.openLoginDialog();
            return;
        }
        if (state.slot === null) setChooser("save");
        else saveMutation.mutate({ slot: state.slot, squad: draft });
    };

    return (
        <div className="page-shell [--page-max:1400px]">
            <PageHeader breadcrumbLabel={t("squad.breadcrumbLabel")} breadcrumb={[t("squad.breadcrumb.tools"), t("squad.title")]} title={t("squad.title")} description={t("squad.intro", { seats: SQUAD_SIZE })} />

            <div className="mt-6 flex flex-col gap-4">
                {readOnlyShare ? (
                    <SharedBanner
                        signedIn={signedIn}
                        onCopy={() => setChooser("share")}
                        onEditCopy={() => dispatch({ type: "KEEP_SHARE" })}
                        onClose={() => {
                            dispatch({ type: "CLOSE_SHARE" });
                            setFocus(null);
                        }}
                        t={t}
                    />
                ) : (
                    <SlotBar signedIn={signedIn} isLoading={squadsLoading} saved={saved} synced={squads?.synced ?? []} activeSlot={state.slot} dirty={dirty} onLoadSlot={loadSlot} onLoadGame={loadGame} onSignIn={() => authActions.openLoginDialog()} />
                )}

                <SquadToolbar
                    name={draft.name}
                    filled={filled}
                    hasSupport={draft.support !== null}
                    slot={state.slot}
                    slotSaved={state.slot !== null && saved[state.slot - 1] != null}
                    dirty={dirty}
                    compact={state.compact}
                    readOnly={readOnlyShare}
                    signedIn={signedIn}
                    saving={saveMutation.isPending}
                    onName={(name) => dispatch({ type: "SET_NAME", name })}
                    onToggleCompact={() => dispatch({ type: "SET_COMPACT", compact: !state.compact })}
                    onRevert={() => {
                        dispatch({ type: "REVERT" });
                        setFocus(null);
                    }}
                    onClear={() => {
                        dispatch({ type: "CLEAR" });
                        setFocus(null);
                    }}
                    onShare={() => setShareOpen(true)}
                    onSave={onSave}
                    onDelete={() => state.slot !== null && deleteMutation.mutate(state.slot)}
                />

                <StageCard
                    stageId={stageId}
                    detail={stageQuery.data}
                    context={stageContext}
                    isLoading={stageId !== null && stageQuery.isLoading}
                    challengeId={challengeId}
                    normalId={challengeId ? normalId : null}
                    readOnly={readOnlyShare}
                    onPick={() => setStagePickerOpen(true)}
                    onSwitch={(id) => void applyStage(id)}
                    onRemove={removeStage}
                />

                <SeatGrid members={draft.members} seats={seats} supportMember={draft.support} support={supportSeat} operators={operators} compact={state.compact} readOnly={readOnlyShare} focus={focus} highlight={hoverSeat} onFocus={onFocus} onSwap={onSwap} onRemove={onRemove} onOpenDps={openLabFor} />

                <div ref={panelRef} className="scroll-mt-24">
                    {focus !== null && focusedMember && focusedResolved && focusedOp ? (
                        <SeatEditor
                            key={`${focus}-${focusedMember.char_id}`}
                            seat={focus}
                            resolved={focusedResolved}
                            op={focusedOp}
                            roster={focus === SUPPORT_SEAT ? undefined : roster.get(focusedMember.char_id)}
                            readOnly={readOnlyShare}
                            squadmates={squadmates}
                            labOpen={labOpen}
                            onLabOpenChange={setLabOpen}
                            onOpenLab={openLab}
                            onChange={onChangeMember}
                            onRemove={() => onRemove(focus)}
                            onClose={() => setFocus(null)}
                        />
                    ) : focus !== null && !focusedMember && !readOnlyShare ? (
                        <OperatorPicker key={focus === SUPPORT_SEAT ? "support" : "seat"} seat={focus} operators={operatorList ?? []} roster={roster} seated={seated} onPick={onPick} onClose={() => setFocus(null)} />
                    ) : null}
                </div>

                <SquadAnalysisPanels
                    request={request}
                    analysis={analysisQuery}
                    seats={seats}
                    support={supportSeat}
                    operators={operators}
                    startDp={state.startDp}
                    onStartDpChange={(startDp) => dispatch({ type: "SET_START_DP", startDp, from: { kind: "page" } })}
                    startDpFrom={state.startDpFrom}
                    stage={stageContext}
                    onHoverSeat={setHoverSeat}
                    onOpenInDps={openLabFor}
                />
            </div>

            <ShareDialog open={shareOpen} onOpenChange={setShareOpen} draft={draft} seats={seats} support={supportSeat} operators={operators} classTotal={analysis?.themes.stats.class_total} />
            <StagePicker open={stagePickerOpen} onOpenChange={setStagePickerOpen} onPick={(id) => void applyStage(id)} />
            <SlotChooser
                open={chooser !== null}
                onOpenChange={(open) => !open && setChooser(null)}
                saved={saved}
                busy={saveMutation.isPending}
                onPick={(slot) => saveMutation.mutate({ slot, squad: draft })}
                onEditCopy={
                    chooser === "share"
                        ? () => {
                              dispatch({ type: "KEEP_SHARE" });
                              setChooser(null);
                          }
                        : undefined
                }
            />

            <LeaveDialog
                open={pendingLoad !== null}
                onCancel={() => setPendingLoad(null)}
                onDiscard={() => {
                    pendingLoad?.();
                    setPendingLoad(null);
                }}
                t={t}
            />
        </div>
    );
}

/** Over a shared squad, read only: keep a copy (saved or not) or go back to the working copy. */
function SharedBanner({ signedIn, onCopy, onEditCopy, onClose, t }: { signedIn: boolean; onCopy: () => void; onEditCopy: () => void; onClose: () => void; t: PageT }): React.ReactElement {
    return (
        <section className="flex flex-col gap-3 rounded-xl border border-primary/40 bg-primary/5 p-4 sm:flex-row sm:items-center">
            <Share2 className="size-5 shrink-0 text-primary" aria-hidden="true" />
            <div className="min-w-0 flex-1">
                <h2 className="font-semibold text-[14px]">{t("squad.shared.title")}</h2>
                <p className="text-[12.5px] text-muted-foreground">{t("squad.shared.desc")}</p>
            </div>
            <div className="flex flex-wrap gap-2">
                {signedIn ? (
                    <Button size="sm" onClick={onCopy}>
                        {t("squad.shared.copy")}
                    </Button>
                ) : (
                    <Button size="sm" onClick={onEditCopy}>
                        {t("squad.shared.editCopy")}
                    </Button>
                )}
                <Button size="sm" variant="outline" onClick={onClose}>
                    {t("squad.shared.close")}
                </Button>
            </div>
        </section>
    );
}

/** Asks before a load replaces unsaved changes. */
function LeaveDialog({ open, onCancel, onDiscard, t }: { open: boolean; onCancel: () => void; onDiscard: () => void; t: PageT }): React.ReactElement {
    return (
        <AlertDialog open={open} onOpenChange={(next) => !next && onCancel()}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>{t("squad.leave.title")}</AlertDialogTitle>
                    <AlertDialogDescription>{t("squad.leave.desc")}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={(p) => <Button {...p} variant="outline" />}>{t("squad.leave.keep")}</AlertDialogClose>
                    <AlertDialogClose render={(p) => <Button {...p} variant="destructive" onClick={onDiscard} />}>{t("squad.leave.discard")}</AlertDialogClose>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
