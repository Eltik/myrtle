import { eliteIcon, potentialIcon } from "#/components/operators/detail/impl/assets";
import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { Dialog, DialogTrigger } from "#/components/ui/dialog";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./CompactCard.messages";
import { isMaxed, MAX_LEVEL_BY_RARITY, moduleBadgeLetter, moduleIconURL, ownedAvatar, parseOperatorName, specializedIcon } from "./helpers.card";
import type { messages as cardMessages } from "./helpers.card.messages";
import { OperatorDialog } from "./OperatorDialog";
import type { IOwnedEntry } from "./types";

/** The shared card vocabulary is declared in `helpers.card.messages.ts`. */
type CompactT = TypedT<typeof messages & typeof cardMessages>;

interface ICompactCardProps {
    entry: IOwnedEntry;
    lastRef?: ((node: HTMLElement | null) => void) | null;
}

export function CompactCard({ entry, lastRef }: ICompactCardProps) {
    const t: CompactT = useT("user");
    const op = entry.static;
    const star = entry.rarity;
    const rarityColor = RARITY_COLORS[star] ?? "#ffffff";

    const { displayName, subtitle } = parseOperatorName(entry.name);
    const nameIsLong = displayName.split(" ").length > 1 && displayName.length >= 16;

    const maxed = isMaxed(entry);
    const maxLevelForPhase = MAX_LEVEL_BY_RARITY[star]?.[entry.elite] ?? 90;
    const isAtMaxLevel = entry.level === maxLevelForPhase;

    const skillCount = Math.min(entry.elite + 1, op?.skills?.length ?? entry.elite + 1);

    const unlockedModules = op
        ? op.modules.filter((m) => {
              if (m.typeName1 === "ORIGINAL") return false;
              const rosterMod = entry.modules.find((rm) => rm.id === m.uniEquipId);
              return rosterMod !== undefined && !rosterMod.locked && rosterMod.level > 0;
          })
        : [];

    return (
        <div ref={lastRef ?? undefined}>
            <Dialog>
                <DialogTrigger render={<button type="button" className="block text-left" />}>
                    <div
                        className="fade-in slide-in-from-bottom-2 relative flex h-min w-min animate-in cursor-pointer flex-col rounded bg-card transition-transform hover:scale-102"
                        style={{
                            padding: "4px 8px 4px 6px",
                            margin: "2px 4px 4px 10px",
                            boxShadow: maxed ? `0 0 10px ${rarityColor}, 0 0 20px ${rarityColor}80` : "0 1px 3px 0 rgb(0 0 0 / 0.1)",
                        }}
                    >
                        <div className="ml-px flex h-4.25 flex-col justify-center text-left sm:h-5">
                            {subtitle && <span className="z-10 text-[0.4375rem] text-foreground leading-normal sm:text-[0.5625rem] sm:leading-loose">{subtitle}</span>}
                            <span
                                className="z-10 text-foreground"
                                style={{
                                    fontSize: nameIsLong ? "9px" : "12px",
                                    lineHeight: nameIsLong ? "9px" : "17px",
                                }}
                            >
                                {displayName}
                            </span>
                        </div>

                        <div
                            className="relative box-content aspect-square h-20 sm:h-30"
                            style={{
                                borderBottom: `4px solid ${rarityColor}`,
                                filter: maxed ? "drop-shadow(0px 0px 8px rgba(255,255,255,0.3))" : undefined,
                            }}
                        >
                            <img alt={entry.name} className="h-full w-full object-contain" decoding="async" height={120} loading="lazy" src={ownedAvatar(entry.operator_id, entry.skin_id)} width={120} />

                            {maxed && (
                                <div className="absolute -bottom-1 left-0 rounded-t px-1 font-medium text-xs sm:text-sm" style={{ backgroundColor: rarityColor, color: "#121212" }}>
                                    {t("profile.roster.card.maxed")}
                                </div>
                            )}
                        </div>

                        {!maxed && (
                            <div className="absolute -bottom-2 -left-3 z-10 flex flex-col gap-0.5">
                                {entry.potential > 0 && (
                                    <div className="relative mb-0.5 ml-1 h-4 w-3 sm:mb-2 sm:h-6 sm:w-5">
                                        <img alt={t("profile.roster.card.potentialAlt", { rank: entry.potential + 1 })} className="h-full w-full object-contain" decoding="async" height={24} loading="lazy" src={potentialIcon(entry.potential)} width={20} />
                                    </div>
                                )}

                                {entry.elite > 0 && (
                                    <div className="mb-0 h-5 w-5 sm:mb-1 sm:h-8 sm:w-8">
                                        <img alt={t("profile.roster.card.eliteAlt", { elite: entry.elite })} className="icon-theme-aware h-full w-full object-contain" decoding="async" height={32} loading="lazy" src={eliteIcon(entry.elite)} width={32} />
                                    </div>
                                )}

                                {(entry.elite > 0 || entry.level > 1) && (
                                    <div
                                        className="flex aspect-square h-8 flex-col items-center justify-center rounded-full border-2 bg-secondary text-lg leading-none sm:h-12 sm:text-2xl"
                                        style={{
                                            borderColor: isAtMaxLevel ? "hsl(var(--primary))" : "hsl(var(--muted-foreground))",
                                        }}
                                    >
                                        <abbr className="hidden text-[0.5625rem] leading-none no-underline sm:flex" title={t("profile.roster.card.level")}>
                                            {t("profile.roster.compact.levelAbbr")}
                                        </abbr>
                                        {entry.level}
                                    </div>
                                )}
                            </div>
                        )}

                        {!maxed && entry.skill_level > 1 && skillCount > 0 && (
                            <div className="absolute top-8 right-0 z-10 flex flex-col gap-0.5 sm:-right-3">
                                {Array.from({ length: skillCount }).map((_, idx) => {
                                    const mastery = entry.masteries.find((m) => m.index === idx)?.mastery ?? 0;
                                    const hasM = mastery > 0;
                                    return (
                                        <div
                                            className="relative flex h-4 w-4 items-center justify-center sm:h-6 sm:w-6"
                                            // biome-ignore lint/suspicious/noArrayIndexKey: skill slots are positional and stable
                                            key={idx}
                                            style={{ marginLeft: `${idx * 4}px` }}
                                            title={hasM ? t("profile.roster.compact.skillMastery", { n: idx + 1, mastery }) : t("profile.roster.compact.skillLevel", { n: idx + 1, level: entry.skill_level })}
                                        >
                                            {hasM ? (
                                                <img alt={t("profile.roster.card.masteryAlt", { mastery })} className="h-full w-full object-contain" decoding="async" height={24} loading="lazy" src={specializedIcon(mastery)} width={24} />
                                            ) : (
                                                <div className="flex h-full w-full items-center justify-center rounded bg-secondary font-bold text-[0.625rem] text-secondary-foreground sm:text-xs">{entry.skill_level}</div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        )}

                        {!maxed && unlockedModules.length > 0 && (
                            <div className="absolute -right-2 -bottom-3 z-10 flex flex-row-reverse gap-1">
                                {unlockedModules.map((module) => {
                                    const rosterMod = entry.modules.find((m) => m.id === module.uniEquipId);
                                    const moduleLevel = rosterMod?.level ?? 0;
                                    const typeLetter = moduleBadgeLetter(module);
                                    const titleLabel = module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : (module.typeName1 ?? t("profile.roster.card.moduleAlt"));
                                    return (
                                        <div className="relative aspect-square h-6 overflow-hidden rounded bg-secondary sm:h-8" key={module.uniEquipId} title={t("profile.roster.compact.moduleStage", { module: titleLabel, level: moduleLevel })}>
                                            <img alt={titleLabel} className="h-full w-full object-contain" decoding="async" height={32} loading="lazy" src={moduleIconURL(module)} width={32} />
                                            <span className="absolute right-0 bottom-0 rounded-tl bg-secondary/90 px-0.5 font-medium text-[0.5625rem] leading-none sm:text-[0.625rem]">
                                                {typeLetter}
                                                {moduleLevel}
                                            </span>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                </DialogTrigger>
                <OperatorDialog entry={entry} />
            </Dialog>
        </div>
    );
}
