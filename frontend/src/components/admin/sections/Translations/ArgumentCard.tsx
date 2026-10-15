import { useMemo } from "react";
import { Badge } from "#/components/ui/badge";
import { type IMessageArgument, pluralCategoriesFor, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { formRows } from "./model";
import type { messages } from "./Translations.messages";

type T = TypedT<typeof messages>;

/**
 * The literal `t()` calls matter: the extractor only sees keys written out in
 * full, so a table lookup would let these fall out of the catalog as
 * "defined but never used".
 */
function kindLabel(t: T, type: IMessageArgument["type"]): string {
    switch (type) {
        case "plain":
            return t("translations.args.kind.plain");
        case "number":
            return t("translations.args.kind.number");
        case "date":
            return t("translations.args.kind.date");
        case "time":
            return t("translations.args.kind.time");
        case "plural":
            return t("translations.args.kind.plural");
        case "selectordinal":
            return t("translations.args.kind.selectordinal");
        case "select":
            return t("translations.args.kind.select");
    }
}

export interface IArgumentCardProps {
    /** A plural, ordinal or choice argument of the English. */
    arg: IMessageArgument;
    /** The same argument as the draft writes it, if it does. */
    written: IMessageArgument | null;
    /** The draft is non-empty, so a missing form is worth flagging. */
    started: boolean;
    locale: string;
    language: string;
}

/** The form table for one branching argument: which wordings the target language needs, and which the draft has. */
export function ArgumentCard({ arg, written, started, locale, language }: IArgumentCardProps): React.ReactElement {
    const t: T = useT("admin");
    const numeric = arg.type === "plural" || arg.type === "selectordinal";
    const rows = useMemo(() => formRows(arg, started ? written : null, locale, started), [arg, written, started, locale]);
    const forms = numeric ? pluralCategoriesFor(locale, arg.type === "selectordinal" ? "selectordinal" : "plural").length : 0;

    return (
        <div className="rounded-[10px] border border-border px-3 py-2.5">
            <div className="flex flex-wrap items-center gap-1.5">
                <span className="font-mono text-[12px]">{`{${arg.name}}`}</span>
                <Badge variant="outline" size="sm">
                    {kindLabel(t, arg.type)}
                </Badge>
            </div>
            <p className="mt-1.5 text-[12px] text-muted-foreground leading-normal">{numeric ? t("translations.args.pluralIntro", { count: t("translations.args.pluralCount", { count: forms }), language }) : t("translations.args.selectIntro")}</p>
            <table className="mt-2 w-full table-auto border-collapse text-left">
                <thead>
                    <tr className="text-[11px] text-muted-foreground">
                        <th className="py-0.5 pr-3 font-medium">{t("translations.args.colForm")}</th>
                        <th className="py-0.5 pr-3 font-medium">{t("translations.args.colApplies")}</th>
                        <th className="py-0.5 font-medium">{t("translations.args.colEnglish")}</th>
                    </tr>
                </thead>
                <tbody>
                    {rows.map((row) => (
                        <tr key={row.key} className="border-border border-t align-baseline">
                            <td className={cn("whitespace-nowrap py-1 pr-3 font-mono text-[11.5px]", row.status === "missing" && "text-warning-foreground")}>{row.key}</td>
                            <td className="py-1 pr-3 text-[12px] text-muted-foreground tabular-nums">{row.key === "other" ? t("translations.args.anythingElse") : row.key.startsWith("=") ? t("translations.args.exact", { value: row.applies }) : row.applies}</td>
                            <td className="py-1 text-[12px] leading-snug">
                                <span className="text-muted-foreground">{row.english}</span>
                                {row.status === "missing" ? (
                                    <Badge variant="warning" size="sm" className="ml-1.5 align-middle">
                                        {t("translations.args.formMissing")}
                                    </Badge>
                                ) : null}
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
