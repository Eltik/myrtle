import { Pause, Play } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { audioURL } from "#/components/operators/detail/impl/assets";
import type { IEnemyAudio, IEnemyAudioCategory } from "#/lib/api/enemies";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./SoundsTab.messages";

type SoundsT = TypedT<typeof messages>;
type SoundsMessageKey = keyof typeof messages & string;

const CATEGORY_ORDER: IEnemyAudioCategory[] = ["spawn", "attack", "skill", "projectile", "death", "other"];

const CATEGORY_LABEL_KEY: Record<IEnemyAudioCategory, SoundsMessageKey> = {
    spawn: "sounds.category.spawn",
    attack: "sounds.category.attack",
    skill: "sounds.category.skill",
    projectile: "sounds.category.projectile",
    death: "sounds.category.death",
    other: "sounds.category.other",
};

const EVENT_LABEL_KEY: Record<string, SoundsMessageKey> = {
    ON_ABILITY_START: "sounds.event.abilityStart",
    ON_ABILITY_ON: "sounds.event.abilityOn",
    ON_ABILITY_HIT: "sounds.event.abilityHit",
    ON_ABILITY_END: "sounds.event.abilityEnd",
    ON_ABILITY_ATTACK_FINISH: "sounds.event.abilityAttackFinish",
    ON_UNIT_BORN: "sounds.event.unitBorn",
    ON_BOSS_ENTER: "sounds.event.bossEnter",
    ON_UNIT_DEAD: "sounds.event.unitDead",
    ON_BUFF_START: "sounds.event.buffStart",
    ON_PROJECTILE_HIT: "sounds.event.projectileHit",
    ON_PROJECTILE_REACH: "sounds.event.projectileReach",
    ON_SPINE_EVENT_TRIGGER: "sounds.event.spineEventTrigger",
    ON_CUSTOM_TRIGGER: "sounds.event.customTrigger",
};

/** Battle order of an ability's events, so a skill reads wind-up -> release -> impact. */
const EVENT_ORDER = ["ON_BOSS_ENTER", "ON_UNIT_BORN", "ON_ABILITY_START", "ON_ABILITY_ON", "ON_ABILITY_HIT", "ON_ABILITY_ATTACK_FINISH", "ON_ABILITY_END", "ON_PROJECTILE_REACH", "ON_PROJECTILE_HIT"];

interface ISoundItem {
    id: string;
    category: IEnemyAudioCategory;
    label: string;
    /** Game token shown under the label: the skill, projectile or buff name. */
    detail: string | null;
    sortKey: string;
    urls: string[];
}

function eventLabel(event: string, t: SoundsT): string {
    const key = EVENT_LABEL_KEY[event];
    // An unmapped event is a raw game token with nothing in the catalog to look
    // it up by, so it is title-cased rather than translated.
    return key
        ? t(key)
        : event
              .replace(/^ON_/, "")
              .toLowerCase()
              .replace(/_/g, " ")
              .replace(/^\w/, (c) => c.toUpperCase());
}

function eventRank(event: string): number {
    const i = EVENT_ORDER.indexOf(event);
    return i === -1 ? EVENT_ORDER.length : i;
}

function buildItems(audio: IEnemyAudio[], server: string, t: SoundsT): ISoundItem[] {
    const assetServer = server === "cn" ? "cn" : undefined;
    const seen = new Set<string>();
    const items: ISoundItem[] = [];
    for (const entry of audio) {
        const urls = [...new Set(entry.sounds.flatMap((s) => s.urls))].map((u) => audioURL(u, assetServer));
        if (urls.length === 0) continue;
        // A basic attack's ability is just `combat`/`attack`; anything longer
        // (`attack.2`) is a real distinction worth showing.
        const ability = entry.ability ?? null;
        const plainAttack = entry.category === "attack" && (ability === "combat" || ability === "attack");
        const detail = entry.via ?? (plainAttack ? null : ability);
        const signature = `${entry.category}|${detail ?? ""}|${entry.event}|${urls.join(",")}`;
        if (seen.has(signature)) continue;
        seen.add(signature);
        items.push({
            id: entry.bankName,
            category: entry.category,
            label: eventLabel(entry.event, t),
            detail,
            sortKey: `${detail ?? ""}|${String(eventRank(entry.event)).padStart(2, "0")}|${entry.bankName}`,
            urls,
        });
    }
    items.sort((a, b) => a.sortKey.localeCompare(b.sortKey));
    return items;
}

function useSoundPlayer() {
    const audioRef = useRef<HTMLAudioElement | null>(null);
    const [playing, setPlaying] = useState<{ id: string; url: string } | null>(null);
    const [progress, setProgress] = useState(0);
    const [errored, setErrored] = useState<Set<string>>(() => new Set());

    const reset = useCallback(() => {
        setPlaying(null);
        setProgress(0);
    }, []);

    const markErrored = useCallback(
        (url: string) => {
            setErrored((prev) => (prev.has(url) ? prev : new Set(prev).add(url)));
            reset();
        },
        [reset],
    );

    const play = useCallback(
        (id: string, url: string) => {
            const a = audioRef.current;
            if (!a || errored.has(url)) return;
            if (playing?.id === id && playing.url === url) {
                a.pause();
                reset();
                return;
            }
            a.src = url;
            setProgress(0);
            setPlaying({ id, url });
            a.play().catch(() => markErrored(url));
        },
        [playing, errored, reset, markErrored],
    );

    return { audioRef, playing, progress, setProgress, errored, play, reset, markErrored };
}

export function SoundsTab({ enemyId, audio }: { enemyId: string; audio: IEnemyAudio[] }) {
    const t: SoundsT = useT("enemies");
    const server = useGamedataServer();
    const items = useMemo(() => buildItems(audio, server, t), [audio, server, t]);
    const player = useSoundPlayer();
    const { audioRef, reset } = player;

    // Moving to another enemy stops whatever was playing.
    // biome-ignore lint/correctness/useExhaustiveDependencies: intentionally reset on enemy change only
    useEffect(() => {
        const a = audioRef.current;
        if (a) {
            a.pause();
            a.removeAttribute("src");
        }
        reset();
    }, [enemyId]);

    const groups = CATEGORY_ORDER.map((category) => ({ category, items: items.filter((i) => i.category === category) })).filter((g) => g.items.length > 0);

    return (
        <div className="flex flex-col gap-5">
            {/* biome-ignore lint/a11y/useMediaCaption: game sound effects have no transcript track */}
            <audio
                ref={audioRef}
                preload="none"
                onTimeUpdate={(e) => {
                    const el = e.currentTarget;
                    if (el.duration > 0) player.setProgress((el.currentTime / el.duration) * 100);
                }}
                onEnded={player.reset}
                onError={() => {
                    if (player.playing) player.markErrored(player.playing.url);
                }}
            />
            <p className="m-0 font-sans text-[12px] text-muted-foreground leading-normal">{t("sounds.intro", { count: items.length })}</p>
            {groups.map((group) => (
                <section key={group.category} className="flex flex-col gap-2">
                    <h3 className="m-0 font-sans font-semibold text-[11px] text-muted-foreground uppercase tracking-wide">{t(CATEGORY_LABEL_KEY[group.category])}</h3>
                    {group.category === "projectile" && <p className="m-0 -mt-1 font-sans text-[11.5px] text-muted-foreground leading-normal">{t("sounds.projectileNote")}</p>}
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                        {group.items.map((item) => (
                            <SoundRow key={item.id} item={item} player={player} t={t} />
                        ))}
                    </div>
                </section>
            ))}
        </div>
    );
}

function SoundRow({ item, player, t }: { item: ISoundItem; player: ReturnType<typeof useSoundPlayer>; t: SoundsT }) {
    const [take, setTake] = useState(0);
    const url = item.urls[Math.min(take, item.urls.length - 1)];
    const isPlaying = player.playing?.id === item.id;
    const isUnavailable = item.urls.every((u) => player.errored.has(u));

    return (
        <div className={cn("relative overflow-hidden rounded-lg border transition-colors", isPlaying ? "border-primary bg-primary/10" : "border-border bg-card/30 hover:bg-secondary/30", isUnavailable && "opacity-60")}>
            <div className="flex items-center gap-3 p-2.5">
                <button
                    type="button"
                    onClick={() => player.play(item.id, url)}
                    disabled={isUnavailable && !isPlaying}
                    aria-label={isPlaying ? t("sounds.pause") : isUnavailable ? t("sounds.unavailable") : t("sounds.play")}
                    className={cn(
                        "relative flex h-8 w-8 shrink-0 items-center justify-center rounded-md transition-colors pointer-coarse:after:absolute pointer-coarse:after:-inset-1.5 disabled:cursor-not-allowed disabled:opacity-50",
                        isPlaying ? "bg-primary text-primary-foreground" : "bg-secondary text-foreground hover:bg-secondary/80",
                    )}
                >
                    {isPlaying ? <Pause className="h-3.5 w-3.5" /> : <Play className="h-3.5 w-3.5" />}
                </button>
                <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium font-sans text-[13px] text-foreground">{item.label}</span>
                        {isUnavailable && <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground uppercase tracking-wide">{t("sounds.unavailable")}</span>}
                    </div>
                    {item.detail && (
                        <p className="m-0 mt-0.5 truncate font-mono text-[11px] text-muted-foreground" title={item.detail}>
                            {item.detail}
                        </p>
                    )}
                </div>
                {item.urls.length > 1 && (
                    <div className="flex shrink-0 flex-wrap justify-end gap-1">
                        {item.urls.map((u, i) => {
                            const playingThis = player.playing?.id === item.id && player.playing.url === u;
                            const erroredThis = player.errored.has(u);
                            return (
                                <button
                                    key={u}
                                    type="button"
                                    disabled={erroredThis}
                                    aria-label={t("sounds.playVariant", { index: i + 1 })}
                                    onClick={() => {
                                        setTake(i);
                                        player.play(item.id, u);
                                    }}
                                    className={cn(
                                        "flex h-6 pointer-coarse:h-11 min-w-6 pointer-coarse:min-w-11 items-center justify-center rounded px-1.5 font-mono text-[11px] transition-colors",
                                        playingThis ? "bg-primary text-primary-foreground" : i === take ? "bg-secondary text-foreground" : "bg-secondary/40 text-muted-foreground hover:bg-secondary hover:text-foreground",
                                        erroredThis && "cursor-not-allowed line-through opacity-40",
                                    )}
                                >
                                    {i + 1}
                                </button>
                            );
                        })}
                    </div>
                )}
            </div>
            {isPlaying && (
                <div className="absolute inset-x-0 bottom-0 h-0.5 bg-primary/20">
                    <div className="h-full bg-primary" style={{ width: `${player.progress}%` }} />
                </div>
            )}
        </div>
    );
}
