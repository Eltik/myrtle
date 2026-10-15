import { Link } from "@tanstack/react-router";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./NotFound.messages";

export function NotFound(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");

    return (
        <section className="flex min-h-[calc(100svh-var(--site-header-height))] flex-1 flex-col items-center justify-center gap-7 px-4 py-12">
            <h1 className="m-0 flex select-none items-center justify-center font-black text-[clamp(120px,20vw,220px)] text-foreground leading-[0.8] tracking-[-0.06em]" aria-label="404">
                <span aria-hidden="true">4</span>
                <img src="/home/myrtle_fall.png" alt={t("notFound.imageAlt")} width={300} height={300} className="mx-[-0.04em] block size-[0.95em]" />
                <span aria-hidden="true">4</span>
            </h1>
            <div className="flex max-w-110 flex-col gap-1.5 text-center">
                <p className="m-0 font-semibold text-[18px] text-foreground">{t("notFound.title")}</p>
                <p className="m-0 text-[15px] text-muted-foreground">{t("notFound.description")}</p>
            </div>
            <Button render={<Link to="/" />}>{t("notFound.returnHome")}</Button>
        </section>
    );
}
