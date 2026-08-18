import type { ReactNode } from "react";
import { baseSkillIcon } from "#/components/operators/detail/impl/assets";
import { colorForTag } from "#/components/operators/detail/impl/description";
import type { IOperatorBaseSkill } from "#/types/operators";

const TOKEN = /<([@$])([\w.]+)>|<\/>/g;

function emphasis(tag: string, content: ReactNode[], key: number): ReactNode {
    return (
        <span key={key} style={{ color: colorForTag(tag) }}>
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
            stack.push({ tag: m[1] + m[2], children: [] });
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

export function BaseSkill({ skill, server }: { skill: IOperatorBaseSkill; server?: "en" | "cn" }) {
    const icon = skill.skillIcon ? baseSkillIcon(skill.skillIcon, server) : "";

    return (
        <div className="flex gap-2">
            {icon && <img alt="" aria-hidden className="mt-px size-5 shrink-0 self-start object-contain" decoding="async" loading="lazy" src={icon} />}
            <div className="flex min-w-0 flex-col gap-0.5">
                <span className="font-semibold text-[11px] text-foreground">{skill.buffName}</span>
                <p className="text-[11px] text-muted-foreground leading-snug">{renderMarkup(skill.description)}</p>
            </div>
        </div>
    );
}
