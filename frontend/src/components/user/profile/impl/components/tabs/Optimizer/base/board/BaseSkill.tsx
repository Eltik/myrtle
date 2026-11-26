import type { ReactNode } from "react";
import { baseSkillIcon } from "#/components/operators/detail/impl/assets";
import type { IRosterSkill } from "#/lib/base/roster";
import { inkForTag, tagTokenRegex } from "#/lib/gamedata/richtext";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./BaseSkill.messages";

const TOKEN = tagTokenRegex();

function emphasis(tag: string, content: ReactNode[], key: number): ReactNode {
    return (
        <span key={key} style={{ color: inkForTag(tag) }}>
            {content}
        </span>
    );
}

function renderMarkup(text: string): ReactNode[] {
    const stack: { tag: string; children: ReactNode[] }[] = [{ tag: "", children: [] }];
    let last = 0;
    let key = 0;

    TOKEN.lastIndex = 0;
    for (let m = TOKEN.exec(text); m !== null; m = TOKEN.exec(text)) {
        const top = stack[stack.length - 1];
        if (m.index > last) top.children.push(text.slice(last, m.index));
        last = m.index + m[0].length;

        if (m[1]) {
            stack.push({ tag: m[1], children: [] });
            continue;
        }

        if (stack.length === 1) continue;
        const done = stack.pop();
        if (done) stack[stack.length - 1].children.push(emphasis(done.tag, done.children, key++));
    }

    if (last < text.length) stack[stack.length - 1].children.push(text.slice(last));

    while (stack.length > 1) {
        const done = stack.pop();
        if (done) stack[stack.length - 1].children.push(emphasis(done.tag, done.children, key++));
    }
    return stack[0].children;
}

/**
 * `dim` marks a skill that is not why the operator is seated (benched, or a
 * superseded tier). Locked and dimmed skills step their text down to the
 * subtle token and grey the icon; the old 40-60% opacity put the muted text at
 * 1.92 to 2.85:1 on the popover.
 */
export function BaseSkill({ skill, server, dim = false }: { skill: IRosterSkill; server?: "en" | "cn"; dim?: boolean }) {
    const t: TypedT<typeof messages> = useT("user");
    const icon = skill.skillIcon ? baseSkillIcon(skill.skillIcon, server) : "";
    const locked = !skill.unlocked;
    const quiet = locked || dim;

    return (
        <div className="flex gap-2">
            {icon && <img alt="" aria-hidden className={cn("mt-px size-5 shrink-0 self-start object-contain", quiet && "opacity-45 grayscale")} decoding="async" height={20} loading="lazy" src={icon} width={20} />}
            <div className="flex min-w-0 flex-col gap-0.5">
                <span className={cn("flex items-baseline gap-1.5 font-semibold text-[11px]", quiet ? "text-subtle-foreground" : "text-foreground")}>
                    {skill.buffName}
                    {locked && <span className="font-normal text-[9.5px] text-subtle-foreground">{t("profile.base.skill.locked", { elite: skill.unlockElite, level: skill.unlockLevel })}</span>}
                </span>
                <p className={cn("text-[11px] leading-snug", quiet ? "text-subtle-foreground" : "text-muted-foreground")}>{renderMarkup(skill.description)}</p>
            </div>
        </div>
    );
}
