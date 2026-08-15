import { LegalContainer, LegalDivider, SectionHeading } from "frontend";
import { Database, Lock, ShieldIcon } from "lucide-react";

// The page shell both legal routes mount: a full-bleed `<main>` with the ambient
// wash behind it and an 880px-max article column. `ambient` adds the hero glow +
// grid overlay that `PrivacyPage` uses behind its title block; `TermsPage` leaves
// it off. Ported from `src/components/legal/PrivacyPage.tsx`.

const P = ({ children }: { children: React.ReactNode }) => <p className="m-0 mb-4 font-sans text-[16px] text-foreground leading-[1.75] last:mb-0">{children}</p>;

export const WithAmbient = () => (
    <LegalContainer ambient>
        <header className="relative z-1 mb-12 flex flex-col items-center text-center">
            <div className="mb-5.5 inline-flex size-18 items-center justify-center rounded-full bg-primary/10 text-primary">
                <ShieldIcon className="size-10" strokeWidth={1.8} />
            </div>
            <h1 className="m-0 mb-3 font-bold font-sans text-[42px] text-foreground leading-[1.05] tracking-[-0.02em]">Privacy Policy</h1>
            <p className="m-0 max-w-[56ch] font-sans text-[16px] text-muted-foreground leading-[1.6]">What myrtle.moe stores when you link a Yostar account, how long it is kept, and how to have it deleted.</p>
        </header>
        <SectionHeading icon={<Database className="size-5" />} title="What we collect" subtitle="Roster, base layout and pull history — only for accounts you link yourself." />
        <P>Linking an account mirrors the same payload the game client receives: owned operators with their promotion, level, skill and module state, base layout, and the gacha history the server still retains.</P>
    </LegalContainer>
);

export const Plain = () => (
    <LegalContainer>
        <header className="mb-10">
            <h1 className="m-0 mb-3 font-bold font-sans text-[42px] text-foreground leading-[1.05] tracking-[-0.02em]">Terms of Service</h1>
            <p className="m-0 font-sans text-[15px] text-muted-foreground leading-[1.6]">Effective January 12, 2026 · Version 2.0</p>
        </header>
        <SectionHeading icon={<Lock className="size-5" />} title="Acceptance of terms" />
        <P>By using myrtle.moe you agree to these terms. The site is an unofficial, fan-made companion for Arknights and is not affiliated with Hypergryph, Yostar, or any of their subsidiaries.</P>
        <LegalDivider />
        <P>Game data, art and names remain the property of their respective rights holders and are reproduced here for reference only.</P>
    </LegalContainer>
);
