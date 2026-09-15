import { Link, useRouter } from "@tanstack/react-router";
import { RefreshCw, TriangleAlertIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./ChangelogError.messages";
import styles from "./impl/ChangelogPage.module.css";

export function ChangelogError({ error }: { error: unknown }) {
    const t: TypedT<typeof messages> = useT("changelog");
    const router = useRouter();
    const [retrying, setRetrying] = useState(false);
    const message = error instanceof Error ? error.message : t("error.unexpected");

    async function retry() {
        setRetrying(true);
        try {
            await router.invalidate();
        } finally {
            setRetrying(false);
        }
    }

    return (
        <main className="relative overflow-x-clip">
            <div className={styles.pageAmbient} aria-hidden="true" />
            <div className="mx-auto flex w-[min(820px,calc(100%-2rem))] flex-col items-center py-20 text-center sm:py-28">
                <span className="mb-5 inline-flex size-16 items-center justify-center rounded-2xl bg-destructive/10 text-destructive-foreground">
                    <TriangleAlertIcon className="size-8" strokeWidth={1.8} aria-hidden="true" />
                </span>
                <h1 className="m-0 font-extrabold text-[clamp(26px,4vw,34px)] text-foreground leading-[1.1] tracking-[-0.02em]">{t("error.title")}</h1>
                <p className="m-0 mt-3 max-w-[48ch] font-sans text-[15px] text-muted-foreground leading-[1.55]">{t("error.body")}</p>
                <p className="wrap-break-word m-0 mt-4 max-w-[52ch] rounded-lg border border-destructive/25 bg-destructive/8 px-3.5 py-2.5 font-mono text-[12.5px] text-destructive-foreground leading-normal">{message}</p>
                <div className="mt-7 flex flex-wrap items-center justify-center gap-2.5">
                    <Button size="lg" disabled={retrying} onClick={retry}>
                        <RefreshCw className={retrying ? "animate-spin" : undefined} aria-hidden="true" /> {t("error.retry")}
                    </Button>
                    <Button variant="outline" size="lg" render={<Link to="/" />}>
                        {t("error.home")}
                    </Button>
                </div>
            </div>
        </main>
    );
}
