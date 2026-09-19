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
        /* The badges hang off the card's edges by design. They used to hang into
           an asymmetric MARGIN on a `w-min` card, which left the card floating at
           the left of a `1fr` track with the rest of the track empty: at a 700px
           roster that was four cards where five fit. The overhang is reserved here,
           on the grid item, so the card itself can be `w-full` and the track sets
           the card size rather than the card ignoring the track. */
        <div ref={lastRef ?? undefined} className="pt-0.5 pr-2.5 pb-2.5 pl-2.5">
            <Dialog>
                {/* `w-full`: a `display: block` button still sizes to its content, so the
                    card's own `w-full` was resolving against a shrink-to-fit box and the
                    portrait came out 65px in a 132px track. */}
                <DialogTrigger render={<button type="button" className="block w-full text-left" />}>
                    <div
                        className="fade-in slide-in-from-bottom-2 relative flex h-min w-full animate-in cursor-pointer flex-col rounded bg-card transition-transform hover:scale-102"
                        style={{
                            padding: "4px 6px",
                            boxShadow: maxed ? `0 0 10px ${rarityColor}, 0 0 20px ${rarityColor}80` : "0 1px 3px 0 rgb(0 0 0 / 0.1)",
                        }}
                    >
                        {/* The box reserved 17px (20px at `sm`) for two lines that ask for
                            25px and 27px: `leading-loose` alone gives the subtitle 18px at
                            9px type. With `overflow-hidden` and `justify-center` that ate
                            the top of the alternate name on every operator that has one,
                            which is what "not enough vertical space" was. The height is
                            the sum of the two line boxes now, and stays fixed so cards in
                            a row keep their portraits on one line whether or not the
                            operator has an alternate name. */}
                        <div className="ml-px flex h-6.25 min-w-0 flex-col justify-center overflow-hidden text-left sm:h-6.75">
                            {subtitle && <span className="z-10 truncate text-[0.4375rem] text-foreground leading-[10px] sm:text-[0.5625rem] sm:leading-[12px]">{subtitle}</span>}
                            <span
                                className="z-10 truncate text-foreground"
                                style={{
                                    fontSize: nameIsLong ? "9px" : "12px",
                                    lineHeight: nameIsLong ? "11px" : "15px",
                                }}
                            >
                                {displayName}
                            </span>
                        </div>

                        <div
                            /* The portrait follows the TRACK, which follows the grid. A fixed
                               80px (then 120px at `sm`) portrait set the card's width and left
                               every track partly empty, and needed a viewport calc to fit three
                               on a phone. The grid decides the density now and the card fills
                               whatever it is given. */
                            className="relative box-content aspect-square w-full"
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
                            <div className="absolute -bottom-2 -left-2.5 z-10 flex flex-col gap-0.5">
                                {entry.potential > 0 && (
                                    <div className="relative mb-0.5 ml-1 h-5 w-4 sm:mb-2 sm:h-6 sm:w-5">
                                        <img alt={t("profile.roster.card.potentialAlt", { rank: entry.potential + 1 })} className="icon-theme-aware h-full w-full object-contain" decoding="async" height={24} loading="lazy" src={potentialIcon(entry.potential)} width={20} />
                                    </div>
                                )}

                                {entry.elite > 0 && (
                                    <div className="mb-0 h-6.5 w-6.5 sm:mb-1 sm:h-8 sm:w-8">
                                        <img alt={t("profile.roster.card.eliteAlt", { elite: entry.elite })} className="icon-theme-aware h-full w-full object-contain" decoding="async" height={32} loading="lazy" src={eliteIcon(entry.elite)} width={32} />
                                    </div>
                                )}

                                {(entry.elite > 0 || entry.level > 1) && (
                                    <div
                                        className="flex aspect-square h-8 flex-col items-center justify-center rounded-full border-2 bg-secondary text-lg leading-none sm:h-12 sm:text-2xl"
                                        style={{
                                            borderColor: isAtMaxLevel ? "var(--primary)" : "var(--muted-foreground)",
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

                        {/* `top-8` put the bottom of a three-skill column into the module
                            row in the corner below it. The portrait's top right is empty,
                            so the column starts there and clears the modules at every
                            card size the grid produces. */}
                        {!maxed && entry.skill_level > 1 && skillCount > 0 && (
                            <div className="absolute top-1 right-0 z-10 flex flex-col gap-0.5 sm:-right-2.5">
                                {Array.from({ length: skillCount }).map((_, idx) => {
                                    const mastery = entry.masteries.find((m) => m.index === idx)?.mastery ?? 0;
                                    const hasM = mastery > 0;
                                    return (
                                        <div
                                            className="relative flex h-5 w-5 items-center justify-center sm:h-6 sm:w-6"
                                            // biome-ignore lint/suspicious/noArrayIndexKey: skill slots are positional and stable
                                            key={idx}
                                            style={{ marginLeft: `${idx * 4}px` }}
                                            title={hasM ? t("profile.roster.compact.skillMastery", { n: idx + 1, mastery }) : t("profile.roster.compact.skillLevel", { n: idx + 1, level: entry.skill_level })}
                                        >
                                            {hasM ? (
                                                /* NOT `icon-theme-aware`. That filter is `invert(1)
                                                   hue-rotate(180deg)`, which is right for the white line
                                                   art of the potential and promotion sprites and wrong
                                                   here: the mastery sprite's filled triangles are gold,
                                                   and inverting gold gives the black-with-pink-corners
                                                   that got reported. The sprite sits on the same
                                                   `bg-secondary` chip its own skill-level sibling and the
                                                   module badges use, which is what makes it readable on
                                                   the light theme without touching its hue. */
                                                <div className="flex h-full w-full items-center justify-center rounded bg-secondary">
                                                    <img alt={t("profile.roster.card.masteryAlt", { mastery })} className="h-[86%] w-[86%] object-contain" decoding="async" height={24} loading="lazy" src={specializedIcon(mastery)} width={24} />
                                                </div>
                                            ) : (
                                                <div className="flex h-full w-full items-center justify-center rounded bg-secondary font-bold text-[0.6875rem] text-secondary-foreground sm:text-xs">{entry.skill_level}</div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        )}

                        {!maxed && unlockedModules.length > 0 && (
                            <div className="absolute -right-2 -bottom-2.5 z-10 flex flex-row-reverse gap-1">
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
